use super::argon;
use super::jwt;
use crate::error;
use crate::settings::Settings;
use slog::{o, Logger};
use snafu::ResultExt;
use sqlx::postgres::PgPool;

// FIXME Move this struct and its implementation to mod.rs

#[derive(Clone, Debug)]
pub struct State {
    pub pool: PgPool,
    pub logger: Logger,
}

impl State {
    pub async fn new(settings: &Settings, logger: &Logger) -> Result<Self, error::Error> {
        let pool = PgPool::builder()
            .max_size(5)
            .build(&settings.database.url)
            .await
            .context(error::DBError {
                msg: String::from("foo"),
            })?;
        // FIXME ping the pool to know quickly if we have a db connection
        let logger = logger.new(
            o!("host" => String::from(&settings.service.host), "port" => settings.service.port, "database" => String::from(&settings.database.url)),
        );

        Ok(Self { pool, logger })
    }
}
