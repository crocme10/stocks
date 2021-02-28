use async_graphql::extensions::TracingConfig;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use clap::ArgMatches;
use http::StatusCode;
use snafu::{ResultExt, Snafu};
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::net::ToSocketAddrs;
use tracing::{info, instrument, span, Level};
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
            let root_span = span!(parent: None, Level::INFO, "span root");
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
