use async_graphql::extensions::TracingConfig;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use clap::ArgMatches;
use http::StatusCode;
use snafu::{ResultExt, Snafu};
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::net::ToSocketAddrs;
use tracing::{info, instrument, span, Level};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};
use uuid::Uuid;
use warp::{http::Response as HttpResponse, Filter, Rejection};

use stocks::api::gql;
use stocks::settings::Settings;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not get database pool: {}", source))]
    DBConnectionError { source: sqlx::Error },
    #[snafu(display("Could not generate settings: {}", source))]
    SettingsError {
        #[snafu(backtrace)]
        source: stocks::settings::Error,
    },
    #[snafu(display("Could not generate state: {}", source))]
    StateError {
        #[snafu(backtrace)]
        source: stocks::state::Error,
    },
    #[snafu(display("Socket Addr Error {}", source))]
    SockAddrError { source: std::io::Error },
    #[snafu(display("Addr Resolution Error {}", msg))]
    AddrResolutionError { msg: String },
}

#[allow(clippy::needless_lifetimes)]
pub async fn run<'a>(matches: &ArgMatches<'a>) -> Result<(), Error> {
    let settings = Settings::new(matches).context(SettingsError)?;
    LogTracer::init().expect("Unable to setup log tracer!");

    // following code mostly from https://betterprogramming.pub/production-grade-logging-in-rust-applications-2c7fffd108a6
    let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();

    let file_appender = tracing_appender::rolling::daily(&settings.logging.path, "stocks.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    let bunyan_formatting_layer = BunyanFormattingLayer::new(app_name, non_blocking);
    let subscriber = Registry::default()
        .with(EnvFilter::new("INFO"))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    run_server(settings).await
}

#[instrument]
pub async fn run_server(settings: Settings) -> Result<(), Error> {
    let pool = PgPool::connect(&settings.database.url)
        .await
        .context(DBConnectionError)?;
    let service = Box::new(stocks::api::imp::StockServiceImpl { pool });

    let schema = gql::schema(service);

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (gql::StocksSchema, async_graphql::Request)| async move {
            let request_id = Uuid::new_v4();
            let root_span = span!(parent: None, Level::INFO, "graphql request", %request_id);
            let request = request.data(TracingConfig::default().parent_span(root_span));
            Ok::<_, Infallible>(async_graphql_warp::Response::from(
                schema.execute(request).await,
            ))
        },
    );

    let graphql_playground = warp::path("playground").and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    });

    let routes = graphql_playground
        .or(graphql_post)
        .recover(|err: Rejection| async move {
            if let Some(async_graphql_warp::BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        });

    let host = settings.service.host;
    let port = settings.service.port;
    let addr = (host.as_str(), port);
    let addr = addr
        .to_socket_addrs()
        .context(SockAddrError)?
        .next()
        .ok_or(Error::AddrResolutionError {
            msg: String::from("Cannot resolve addr"),
        })?;

    info!("Serving stocks on {}", addr);
    warp::serve(routes).run(addr).await;

    Ok(())
}
