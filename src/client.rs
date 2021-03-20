use clap::{App, Arg, ArgMatches, SubCommand};
use snafu::{ResultExt, Snafu};

use stocks::api::gql;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Command Line Interface Error: {}", msg))]
    CLIError { msg: String },
    #[snafu(display("Decimals Value Error: {} [{}]", msg, source))]
    DecimalsParseError {
        msg: String,
        source: std::num::ParseIntError,
    },
    #[snafu(display("Reqwest Error: {} [{}]", msg, source))]
    ReqwestError { msg: String, source: reqwest::Error },
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Error> {
    let matches = App::new("Client App for stocks")
        .version(VERSION)
        .author("Matthieu Paindavoine")
        .arg(
            Arg::with_name("host")
                .value_name("HOST")
                .short("h")
                .long("host")
                .help("Address serving this server"),
        )
        .arg(
            Arg::with_name("port")
                .value_name("PORT")
                .short("p")
                .long("port")
                .help("Port"),
        )
        .subcommand(
            SubCommand::with_name("add").subcommand(
                SubCommand::with_name("currency")
                    .arg(Arg::with_name("code").index(1))
                    .arg(Arg::with_name("name").index(2))
                    .arg(Arg::with_name("decimals").index(3)),
            ),
        )
        .subcommand(SubCommand::with_name("list").subcommand(SubCommand::with_name("currencies")))
        .get_matches();

    let host = matches.value_of("host").unwrap_or("localhost");
    let port = matches.value_of("port").unwrap_or("6060");

    let url = format!("http://{}:{}/graphql", host, port);
    match matches.subcommand() {
        ("add", Some(matches)) => add_cmd(&url, &matches).await,
        ("list", Some(matches)) => list_cmd(&url, &matches).await,
        _ => Err(Error::CLIError {
            msg: String::from("Unrecognized subcommand"),
        }),
    }
}

#[allow(clippy::needless_lifetimes)]
pub async fn add_cmd<'a>(url: &str, matches: &ArgMatches<'a>) -> Result<(), Error> {
    match matches.subcommand() {
        ("currency", Some(matches)) => add_currency_cmd(url, &matches).await,
        _ => Err(Error::CLIError {
            msg: String::from("Unrecognized add subcommand"),
        }),
    }
}

#[allow(clippy::needless_lifetimes)]
pub async fn list_cmd<'a>(url: &str, matches: &ArgMatches<'a>) -> Result<(), Error> {
    match matches.subcommand() {
        ("currencies", Some(_)) => list_currencies(url).await,
        _ => Err(Error::CLIError {
            msg: String::from("Unrecognized list subcommand"),
        }),
    }
}

#[allow(clippy::needless_lifetimes)]
pub async fn add_currency_cmd<'a>(url: &str, matches: &ArgMatches<'a>) -> Result<(), Error> {
    let code = matches.value_of("code").ok_or(Error::CLIError {
        msg: String::from("Missing Currency Code"),
    })?;
    let name = matches.value_of("name").ok_or(Error::CLIError {
        msg: String::from("Missing Currency Name"),
    })?;
    let decimals = matches.value_of("decimals").ok_or(Error::CLIError {
        msg: String::from("Missing Currency Decimals"),
    })?;

    let decimals = decimals.parse::<i32>().context(DecimalsParseError {
        msg: String::from("Could not parse decimals value"),
    })?;

    let currency = gql::CurrencyInput {
        code: String::from(code),
        name: String::from(name),
        decimals,
    };

    add_currency(url, currency).await
}

pub async fn add_currency(url: &str, currency: gql::CurrencyInput) -> Result<(), Error> {
    let query = r#" "mutation addCurrency($currency: CurrencyInput!) { addCurrency(currency: $currency) { code, name, decimals } }" "#;
    let variables = format!(
        r#" {{ "code": "{code}", "name": "{name}", "decimals": {decimals} }}"#,
        code = currency.code,
        name = currency.name,
        decimals = currency.decimals
    );
    let body = format!(
        r#"{{ "query": {query}, "variables": {{ "currency": {variables} }} }}"#,
        query = query,
        variables = variables
    );

    let client = reqwest::Client::new();
    let _ = client
        .post(url)
        .body(body)
        .send()
        .await
        .context(ReqwestError {
            msg: String::from("req err"),
        })?;

    Ok(())
}

pub async fn list_currencies(url: &str) -> Result<(), Error> {
    let query = r#" "query listCurrencies() { code, name, decimals }" "#;
    let body = format!(r#"{{ "query": {query} }}"#, query = query);

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .body(body)
        .send()
        .await
        .context(ReqwestError {
            msg: String::from("req err"),
        })?;

    println!("Resp: {:?}", resp);

    Ok(())
}
