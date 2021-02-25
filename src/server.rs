use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
// use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_warp::{BadRequest, Response};
use clap::ArgMatches;
use http::StatusCode;
use snafu::{ResultExt, Snafu};
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::net::ToSocketAddrs;
use tracing::{info, instrument};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_currency() {
        let mut service = stocks::api::model::MockStockService::new();
        service
            .expect_add_currency()
            .times(1)
            .returning(|code, name, decimals| {
                Ok(stocks::api::model::Currency {
                    code: String::from(code),
                    name: String::from(name),
                    decimals,
                })
            });

        let schema = gql::schema(service);

        let graphql_post = async_graphql_warp::graphql(schema).and_then(
            |(schema, request): (gql::StocksSchema, async_graphql::Request)| async move {
                Ok::<_, Infallible>(Response::from(schema.execute(request).await))
            },
        );

        let query = r#" "mutation addCurrency($currency: CurrencyInput!) { addCurrency(currency: $currency) { code, name, decimals } }" "#;
        let variables = r#" { "code": "EUR", "name": "Euro", "decimals": 2 }"#;
        let body = format!(
            r#"{{ "query": {query}, "variables": {{ "currency": {variables} }} }}"#,
            query = query,
            variables = variables
        );

        let res = warp::test::request()
            .method("POST")
            .body(body)
            .reply(&graphql_post)
            .await;

        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "Hello");
    }
}
