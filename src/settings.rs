use clap::ArgMatches;
use config::{Config, Environment, File};
use serde::Deserialize;
use snafu::ResultExt;
use snafu::Snafu;
use std::convert::TryFrom;
use std::env;
use std::path::Path;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Arg Match Error: {}", msg))]
    ArgMatchError { msg: String },
    #[snafu(display("Arg Missing Error: {}", msg))]
    ArgMissingError { msg: String },
    #[snafu(display("Env Var Missing Error: {} [{}]", msg, source))]
    EnvVarMissingError { msg: String, source: env::VarError },
    #[snafu(display("Config Merge Error: {} [{}]", msg, source))]
    ConfigMergeError {
        msg: String,
        source: config::ConfigError,
    },
    #[snafu(display("Config Extract Error: {} [{}]", msg, source))]
    ConfigExtractError {
        msg: String,
        source: config::ConfigError,
    },
    #[snafu(display("Config Value Error: {} [{}]", msg, source))]
    ConfigValueError {
        msg: String,
        source: std::num::TryFromIntError,
    },
    #[snafu(display("Config Value Error: {} [{}]", msg, source))]
    ConfigParseError {
        msg: String,
        source: std::num::ParseIntError,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logging {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub testing: bool,
    pub mode: String,
    pub logging: Logging,
    pub database: Database,
    pub service: Service,
}

// TODO Parameterize the config directory

impl Settings {
    pub fn new<'a, T: Into<Option<&'a ArgMatches<'a>>>>(matches: T) -> Result<Self, Error> {
        let matches = matches.into().ok_or(Error::ArgMatchError {
            msg: String::from("no matches"),
        })?;

        let config_dir = matches
            .value_of("config dir")
            .ok_or(Error::ArgMissingError {
                msg: String::from("no config dir"),
            })?;

        let config_dir = Path::new(config_dir);

        let mut s = Config::new();

        let default_path = config_dir.join("default").with_extension("toml");

        // Start off by merging in the "default" configuration file
        s.merge(File::from(default_path))
            .context(ConfigMergeError {
                msg: String::from("Could not merge default configuration"),
            })?;

        // Add in the current environment file
        // Note that this file is _optional_
        let settings = matches.value_of("settings").ok_or(Error::ArgMissingError {
            msg: String::from("no settings"),
        })?;

        let settings = env::var("RUN_MODE").unwrap_or_else(|_| String::from(settings));

        let settings_path = config_dir.join(&settings).with_extension("toml");

        s.merge(File::from(settings_path).required(true))
            .context(ConfigMergeError {
                msg: format!("Could not merge {} configuration", settings),
            })?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        s.merge(File::with_name("config/local").required(false))
            .context(ConfigMergeError {
                msg: String::from("Could not merge local configuration"),
            })?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("app"))
            .context(ConfigMergeError {
                msg: String::from("Could not merge configuration from environment variables"),
            })?;

        // Now we take care of the database.url, which can be had from environment variables.
        let key = match settings.as_str() {
            "testing" => "DATABASE_TEST_URL",
            _ => "DATABASE_URL",
        };

        let db_url = env::var(key).context(EnvVarMissingError {
            msg: format!("Could not get env var {}", key),
        })?;

        s.set("database.url", db_url).context(ConfigExtractError {
            msg: String::from("Could not set database url from environment variable"),
        })?;

        // For the port, the value by default is the one in the configuration file. But it
        // gets overwritten by the environment variable STOCKS_GRAPHQL_PORT.
        let default_port = s.get_int("service.port").context(ConfigExtractError {
            msg: String::from("Could not get default port"),
        })?;

        // config crate support i64, not u16
        let default_port = u16::try_from(default_port).context(ConfigValueError {
            //map_err(|err| error::Error::MiscError {
            msg: String::from("Could not get u16 port"),
        })?;

        // We retrieve the port, first as a string, which must be turned into an u16, because
        // thats a correct port number. But then it should be cast into an i64 in the config,
        // because config doesn't trade in u16s.
        let port = env::var("STOCKS_GRAPHQL_PORT").unwrap_or_else(|_| format!("{}", default_port));

        let port = port.parse::<u16>().context(ConfigParseError {
            msg: String::from("Could not parse into a valid port number"),
        })?;

        let port = i64::try_from(port).unwrap(); // infaillible

        s.set("service.port", port).context(ConfigExtractError {
            msg: String::from("Could not set service port"),
        })?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into().context(ConfigMergeError {
            msg: String::from("Could not generate settings from configuration"),
        })
    }
}
