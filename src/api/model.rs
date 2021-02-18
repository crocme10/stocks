use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use sqlx::Connection;
use std::convert::TryFrom;

use super::error;
use crate::api::gql::Context;
use crate::db::model as db;
use crate::db::model::ProvideStock;

#[derive(Debug, PartialEq, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub code: String,
    pub name: String,
    pub decimals: i32,
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

#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct SingleCurrencyResponseBody {
    pub currency: Option<Currency>,
}

impl From<Currency> for SingleCurrencyResponseBody {
    fn from(currency: Currency) -> Self {
        Self {
            currency: Some(currency),
        }
    }
}

/// The response body for multiple documents
#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct MultiCurrenciesResponseBody {
    pub currencies: Vec<Currency>,
    pub currencies_count: i32,
}

impl From<Vec<Currency>> for MultiCurrenciesResponseBody {
    fn from(currencies: Vec<Currency>) -> Self {
        let currencies_count = i32::try_from(currencies.len()).unwrap();
        Self {
            currencies,
            currencies_count,
        }
    }
}

/// Retrieve all currencies
pub async fn list_currencies(
    context: &Context,
) -> Result<MultiCurrenciesResponseBody, error::Error> {
    async move {
        let pool = &context.state.pool;

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

        Ok(MultiCurrenciesResponseBody::from(currencies))
    }
    .await
}
