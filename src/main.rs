use clap::{App, Arg, SubCommand};

mod init;
mod server;

use stocks::error;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
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

    match matches.subcommand() {
        ("run", Some(_)) => server::run(&matches).await,
        ("init", Some(_)) => init::init(&matches).await,
        // ("test", Some(sm)) => test::test(sm, logger).await,
        _ => Err(error::Error::MiscError {
            msg: String::from("Unrecognized subcommand"),
        }),
    }
}
