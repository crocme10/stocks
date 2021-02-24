use async_graphql::*;
use async_trait::async_trait;
use snafu::ResultExt;
use sqlx::postgres::PgPool;
use sqlx::Acquire;

use super::error;
use super::model;
use crate::db::model::ProvideStock;

pub struct StockServiceImpl {
    pub pool: PgPool,
}

#[async_trait]
impl model::StockService for StockServiceImpl {
    /// Retrieve all currencies
    async fn list_currencies(&self) -> Result<Vec<model::Currency>, error::Error> {
        async move {
            let pool = &self.pool;

            let mut conn = pool.acquire().await.context(error::DBConnectionError {
                msg: "could not acquire connection",
            })?;

            let mut tx = conn.begin().await.context(error::DBTransactionError {
                msg: "could not initiate transaction",
            })?;

            let entities = tx.list_currencies().await.context(error::DBProvideError {
                msg: "Could not get all them currencies",
            })?;

            let currencies = entities
                .into_iter()
                .map(model::Currency::from)
                .collect::<Vec<_>>();

            tx.commit().await.context(error::DBTransactionError {
                msg: "could not commit transaction",
            })?;

            Ok(currencies)
        }
        .await
    }

    async fn add_currency(
        &self,
        code: &str,
        name: &str,
        decimals: u32,
    ) -> Result<model::Currency, error::Error> {
        async move {
            let pool = &self.pool;

            let mut conn = pool.acquire().await.context(error::DBConnectionError {
                msg: "could not acquire connection",
            })?;

            let mut tx = conn.begin().await.context(error::DBTransactionError {
                msg: "could not initiate transaction",
            })?;

            let entity =
                tx.add_currency(code, name, decimals)
                    .await
                    .context(error::DBProvideError {
                        msg: "Could not get add currency",
                    })?;

            let currency = model::Currency::from(entity);

            tx.commit().await.context(error::DBTransactionError {
                msg: "could not commit transaction",
            })?;

            Ok(currency)
        }
        .await
    }
}
