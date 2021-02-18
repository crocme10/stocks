use snafu::{ResultExt, Snafu};
use sqlx::postgres::PgPool;
use tracing::info;

use crate::settings::Settings;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Database Connection Error: {} [{}]", msg, source))]
    DBConnectionError { msg: String, source: sqlx::Error },
    #[snafu(display("Database Version Error: {} [{}]", msg, source))]
    DBVersionError { msg: String, source: sqlx::Error },
}

#[derive(Clone, Debug)]
pub struct State {
    pub pool: PgPool,
    pub settings: Settings,
}

impl State {
    pub async fn new(settings: &Settings) -> Result<Self, Error> {
        let pool = PgPool::connect(&settings.database.url)
            .await
            .context(DBConnectionError {
                msg: String::from("foo"),
            })?;

        let row: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&pool)
            .await
            .context(DBVersionError {
                msg: format!(
                    "Could not test database version for {}",
                    &settings.database.url,
                ),
            })?;

        info!("db version: {:?}", row.0);

        Ok(Self {
            pool,
            settings: settings.clone(),
        })
    }
}
