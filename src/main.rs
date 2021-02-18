use clap::{App, Arg, SubCommand};
use snafu::{ResultExt, Snafu};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

mod server;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Command Line Interface Error: {}", msg))]
    CLIError { msg: String },
    #[snafu(display("Server Error: {}", source))]
    ServerError {
        #[snafu(backtrace)]
        source: server::Error,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let matches = App::new("Microservice for stocks")
        .version("0.1")
        .author("Matthieu Paindavoine")
        .arg(
            Arg::with_name("config dir")
                .value_name("DIR")
                .short("c")
                .long("config-dir")
                .help("Config directory"),
        )
        .arg(
            Arg::with_name("settings")
                .value_name("STRING")
                .short("s")
                .long("settings")
                .help("Settings"),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("publish stocks service")
                .version("0.1")
                .author("Matthieu Paindavoine <matt@area403.org>")
                .arg(
                    Arg::with_name("address")
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
                ),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize Database")
                .version("0.1")
                .author("Matthieu Paindavoine <matt@area403.org>"),
        )
        .subcommand(
            SubCommand::with_name("test")
                .about("Test Something")
                .version("0.1")
                .author("Matthieu Paindavoine <matt@area403.org>"),
        )
        .get_matches();

    LogTracer::init().expect("Unable to setup log tracer!");

    let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let bunyan_formatting_layer = BunyanFormattingLayer::new(app_name, non_blocking_writer);
    let subscriber = Registry::default()
        .with(EnvFilter::new("INFO"))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    match matches.subcommand() {
        ("run", Some(_)) => server::run(&matches).await.context(ServerError),
        _ => Err(Error::CLIError {
            msg: String::from("Unrecognized subcommand"),
        }),
    }
}
