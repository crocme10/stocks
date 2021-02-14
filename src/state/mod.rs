use crate::error;
use crate::settings::Settings;
use snafu::ResultExt;
use sqlx::postgres::PgPool;
//use sqlx::postgres::{PgRow, Postgres};
use tracing::info;

#[derive(Clone, Debug)]
pub struct State {
    pub pool: PgPool,
    pub settings: Settings,
}

impl State {
    pub async fn new(settings: &Settings) -> Result<Self, error::Error> {
        let pool = PgPool::connect(&settings.database.url)
            .await
            .context(error::DBError {
                msg: String::from("foo"),
            })?;

        let row: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&pool)
            .await
            .context(error::DBError {
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
