use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
// use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_warp::{BadRequest, Response};
use clap::ArgMatches;
use http::StatusCode;
use snafu::{ResultExt, Snafu};
use std::convert::Infallible;
use std::net::ToSocketAddrs;
use tracing::{debug, info, instrument};
use warp::{http::Response as HttpResponse, Filter, Rejection};

use stocks::api::gql;
use stocks::settings::Settings;
use stocks::state::State;

#[derive(Debug, Snafu)]
pub enum Error {
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
    let state = State::new(&settings).await.context(StateError)?;
    run_server(state).await
}

#[instrument]
pub async fn run_server(state: State) -> Result<(), Error> {
    // We keep a copy of the logger before the context takes ownership of it.
    debug!("Entering server");
    let state1 = state.clone();
    // let qm_state1 = warp::any().map(move || gql::Context {
    //     state: state1.clone(),
    // });

    let schema = gql::schema(state1);

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (gql::StocksSchema, async_graphql::Request)| async move {
            Ok::<_, Infallible>(Response::from(schema.execute(request).await))
        },
    );

    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    });

    let routes = graphql_playground
        .or(graphql_post)
        .recover(|err: Rejection| async move {
            if let Some(BadRequest(err)) = err.find() {
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

    let host = state.settings.service.host;
    let port = state.settings.service.port;
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
