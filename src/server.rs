use clap::ArgMatches;
use juniper_warp::playground_filter;
use snafu::ResultExt;
use std::net::ToSocketAddrs;
use tracing::{debug, info, instrument};

use warp::{self, Filter};

use stocks::api::gql;
use stocks::error;
use stocks::settings::Settings;
use stocks::state::State;

#[allow(clippy::needless_lifetimes)]
pub async fn run<'a>(matches: &ArgMatches<'a>) -> Result<(), error::Error> {
    let settings = Settings::new(matches)?;
    let state = State::new(&settings).await?;
    run_server(state).await
}

#[instrument]
pub async fn run_server(state: State) -> Result<(), error::Error> {
    // We keep a copy of the logger before the context takes ownership of it.
    debug!("Entering server");
    let state1 = state.clone();
    let qm_state1 = warp::any().map(move || gql::Context {
        state: state1.clone(),
    });

    let qm_schema = gql::schema();
    let graphql = warp::post()
        .and(warp::path("graphql"))
        .and(juniper_warp::make_graphql_filter(
            qm_schema,
            qm_state1.boxed(),
        ));

    let playground = warp::get()
        .and(warp::path("playground"))
        .and(playground_filter("/graphql", Some("/subscriptions")));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["content-type", "authorization"])
        .allow_any_origin()
        .build();

    let log = warp::log("journal::graphql");

    let routes = playground.or(graphql).with(cors).with(log);

    let host = state.settings.service.host;
    let port = state.settings.service.port;
    let addr = (host.as_str(), port);
    let addr = addr
        .to_socket_addrs()
        .context(error::IOError {
            msg: String::from("To Sock Addr"),
        })?
        .next()
        .ok_or(error::Error::MiscError {
            msg: String::from("Cannot resolve addr"),
        })?;

    info!("Serving stocks on {}", addr);
    warp::serve(routes).run(addr).await;

    Ok(())
}
