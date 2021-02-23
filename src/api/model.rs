// use juniper::GraphQLObject;
use async_graphql::*;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use sqlx::Connection;

use super::error;
use crate::db::model as db;
use crate::db::model::ProvideStock;
use crate::state::State;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub code: String,
    pub name: String,
    pub decimals: i32,
}

#[Object]
impl Currency {
    async fn code(&self) -> &String {
        &self.code
    }

    async fn name(&self) -> &String {
        &self.name
    }

    async fn decimals(&self) -> &i32 {
        &self.decimals
    }
}

impl From<db::CurrencyEntity> for Currency {
    fn from(entity: db::CurrencyEntity) -> Self {
        let db::CurrencyEntity {
            code,
            name,
            decimals,
            ..
        } = entity;

        Currency {
            code,
            name,
            decimals,
        }
    }
}

/// Retrieve all currencies
pub async fn list_currencies(state: &State) -> Result<Vec<Currency>, error::Error> {
    async move {
        let pool = &state.pool;

        let mut conn = pool.acquire().await.context(error::DBConnectionError {
            msg: "could not acquire connection",
        })?;

        let mut tx = conn.begin().await.context(error::DBTransactionError {
            msg: "could not initiate transaction",
        })?;

        let entities = tx.list_currencies().await.context(error::DBProvideError {
            msg: "Could not get all them currencies",
        })?;

        let currencies = entities.into_iter().map(Currency::from).collect::<Vec<_>>();

        tx.commit().await.context(error::DBTransactionError {
            msg: "could not commit transaction",
        })?;

        Ok(currencies)
    }
    .await
}

pub async fn add_currency(
    state: &State,
    code: &str,
    name: &str,
    decimals: u32,
) -> Result<Currency, error::Error> {
    async move {
        let pool = &state.pool;

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
                    msg: "Could not add currency",
                })?;

        let currency = Currency::from(entity);

        tx.commit().await.context(error::DBTransactionError {
            msg: "could not commit transaction",
        })?;

        Ok(currency)
    }
    .await
}
