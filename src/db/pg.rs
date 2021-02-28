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

    async fn add_currency(
        &mut self,
        code: &str,
        name: &str,
        decimals: i32,
    ) -> model::ProvideResult<model::CurrencyEntity> {
        let currency: model::CurrencyEntity = sqlx::query_as(
            r#"SELECT * FROM api.add_currency($1::CHAR(3), $2::VARCHAR(255), $3::INTEGER)"#,
        )
        .bind(&code)
        .bind(&name)
        .bind(&decimals)
        .fetch_one(self)
        .await?;
        Ok(currency)
    }

    async fn find_currency(
        &mut self,
        code: &str,
    ) -> model::ProvideResult<Option<model::CurrencyEntity>> {
        let currency: Option<model::CurrencyEntity> =
            sqlx::query_as(r#"SELECT * FROM api.find_currency_by_code($1::CHAR(3))"#)
                .bind(&code)
                .fetch_optional(self)
                .await?;
        Ok(currency)
    }
}

#[cfg(test)]
mod tests {
    use super::model::ProvideStock;
    use crate::utils::get_database_url;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    #[tokio::test]
    async fn test_add_and_find_currency() {
        let url = get_database_url();
        let pool = PgPoolOptions::new()
            .connect_timeout(Duration::new(2, 0))
            .connect(&url)
            .await
            .expect("Database connection");
        let mut conn = pool.acquire().await.expect("connection");

        let _currency = conn
            .add_currency("EUR", "Euro", 2)
            .await
            .expect("add currency");

        let currency = conn.find_currency("EUR").await.expect("find currency");

        assert_eq!(currency.unwrap().name, "Euro");
    }
}
