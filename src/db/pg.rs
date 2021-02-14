use async_trait::async_trait;
use slog::{debug, info, o, Logger};
use snafu::ResultExt;
use sqlx::error::DatabaseError;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgError, PgQueryAs, PgRow};
use sqlx::row::{FromRow, Row};
use sqlx::{PgConnection, PgPool};
use std::convert::TryFrom;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::model;
use super::Db;
use crate::error;

// This should match the information in api.currency_type
impl<'c> FromRow<'c, PgRow<'c>> for model::CurrencyEntity {
    fn from_row(row: &PgRow<'c>) -> Result<Self, sqlx::Error> {
        Ok(model::CurrencyEntity {
            code: row.try_get(0)?,
            name: row.try_get(1)?,
            decimals: row.try_get(2)?,
        })
    }
}

/// Open a connection to a database
pub async fn connect(db_url: &str) -> sqlx::Result<PgPool> {
    let pool = PgPool::new(db_url).await?;
    Ok(pool)
}

impl TryFrom<&PgError> for model::ProvideError {
    type Error = ();

    /// Attempt to convert a Postgres error into a generic ProvideError
    ///
    /// Unexpected cases will be bounced back to the caller for handling
    ///
    /// * [Postgres Error Codes](https://www.postgresql.org/docs/current/errcodes-appendix.html)
    fn try_from(pg_err: &PgError) -> Result<Self, Self::Error> {
        let provider_err = match pg_err.code().unwrap() {
            "23505" => model::ProvideError::UniqueViolation {
                details: pg_err.details().unwrap().to_owned(),
            },
            code if code.starts_with("23") => model::ProvideError::ModelViolation {
                details: pg_err.message().to_owned(),
            },
            _ => return Err(()),
        };

        Ok(provider_err)
    }
}

#[async_trait]
impl Db for PgPool {
    type Conn = PoolConnection<PgConnection>;

    async fn conn(&self) -> Result<Self::Conn, sqlx::Error> {
        self.acquire().await
    }
}

#[async_trait]
impl model::ProvideStock for PgConnection {
    async fn list_currencies(&mut self) -> model::ProvideResult<Vec<model::CurrencyEntity>> {
        let currencies: Vec<model::CurrencyEntity> =
            sqlx::query_as(r#"SELECT * FROM api.list_currencies()"#)
                .fetch_all(self)
                .await?;

        Ok(currencies)
    }
}

pub async fn init_db(conn_str: &str, logger: Logger) -> Result<(), error::Error> {
    info!(logger, "Initializing  DB @ {}", conn_str);
    migration_down(conn_str, &logger).await?;
    migration_up(conn_str, &logger).await?;
    Ok(())
}

pub async fn migration_up(conn_str: &str, logger: &Logger) -> Result<(), error::Error> {
    let clogger = logger.new(o!("database" => String::from(conn_str)));
    debug!(clogger, "Movine Up");
    // This is essentially running 'psql $DATABASE_URL < db/init.sql', and logging the
    // psql output.
    // FIXME This relies on a command psql, which is not desibable.
    // We could alternatively try to use sqlx...
    // There may be a tool for doing migrations.
    let mut cmd = Command::new("movine");
    cmd.env("DATABASE_URL", conn_str);
    cmd.arg("up");
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().context(error::TokioIOError {
        msg: String::from("Failed to execute movine"),
    })?;

    let stdout = child.stdout.take().ok_or(error::Error::MiscError {
        msg: String::from("child did not have a handle to stdout"),
    })?;

    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        // FIXME Need to do something about logging this and returning an error.
        let _status = child.await.expect("child process encountered an error");
        // println!("child status was: {}", status);
    });
    debug!(clogger, "Spawned migration up");

    while let Some(line) = reader.next_line().await.context(error::TokioIOError {
        msg: String::from("Could not read from piped output"),
    })? {
        debug!(clogger, "movine: {}", line);
    }

    Ok(())
}

pub async fn migration_down(conn_str: &str, logger: &Logger) -> Result<(), error::Error> {
    let clogger = logger.new(o!("database" => String::from(conn_str)));
    debug!(clogger, "Movine Down");
    // This is essentially running 'psql $DATABASE_URL < db/init.sql', and logging the
    // psql output.
    // FIXME This relies on a command psql, which is not desibable.
    // We could alternatively try to use sqlx...
    // There may be a tool for doing migrations.
    let mut cmd = Command::new("movine");
    cmd.env("DATABASE_URL", conn_str);
    cmd.arg("down");
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().context(error::TokioIOError {
        msg: String::from("Failed to execute movine"),
    })?;

    let stdout = child.stdout.take().ok_or(error::Error::MiscError {
        msg: String::from("child did not have a handle to stdout"),
    })?;

    let mut reader = BufReader::new(stdout).lines();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async {
        // FIXME Need to do something about logging this and returning an error.
        let _status = child.await.expect("child process encountered an error");
        // println!("child status was: {}", status);
    });
    debug!(clogger, "Spawned migration down");

    while let Some(line) = reader.next_line().await.context(error::TokioIOError {
        msg: String::from("Could not read from piped output"),
    })? {
        debug!(clogger, "movine: {}", line);
    }

    Ok(())
}
