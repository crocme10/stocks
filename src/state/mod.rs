use crate::error;
use crate::settings::Settings;
use slog::{info, o, Drain, Logger};
use snafu::ResultExt;
use sqlx::postgres::PgPool;
use sqlx::postgres::PgQueryAs;

#[derive(Clone, Debug)]
pub struct State {
    pub pool: PgPool,
    pub logger: Logger,
    pub settings: Settings,
}

impl State {
    pub async fn new(settings: &Settings) -> Result<Self, error::Error> {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, o!());

        let pool = PgPool::builder()
            .max_size(5)
            .build(&settings.database.url)
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

        info!(logger, "db version: {:?}", row.0);

        Ok(Self {
            pool,
            logger,
            settings: settings.clone(),
        })
    }
}
