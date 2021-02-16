use async_trait::async_trait;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgDatabaseError, PgRow, Postgres};
use sqlx::{FromRow, Row};
use sqlx::{PgConnection, PgPool};
use std::convert::TryFrom;

use super::model;
use super::Db;

// This should match the information in api.currency_type
impl<'c> FromRow<'c, PgRow> for model::CurrencyEntity {
    fn from_row(row: &'c PgRow) -> Result<Self, sqlx::Error> {
        Ok(model::CurrencyEntity {
            code: row.try_get(0)?,
            name: row.try_get(1)?,
            decimals: row.try_get(2)?,
        })
    }
}

/// Open a connection to a database
pub async fn connect(db_url: &str) -> sqlx::Result<PgPool> {
    let pool = PgPool::connect(db_url).await?;
    Ok(pool)
}

impl TryFrom<&PgDatabaseError> for model::ProvideError {
    type Error = ();

    /// Attempt to convert a Postgres error into a generic ProvideError
    ///
    /// Unexpected cases will be bounced back to the caller for handling
    ///
    /// * [Postgres Error Codes](https://www.postgresql.org/docs/current/errcodes-appendix.html)
    fn try_from(pg_err: &PgDatabaseError) -> Result<Self, Self::Error> {
        let provider_err = match pg_err.code() {
            "23505" => model::ProvideError::UniqueViolation {
                details: pg_err.detail().unwrap().to_owned(),
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
    type Conn = PoolConnection<Postgres>;

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
