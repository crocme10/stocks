// use juniper::GraphQLObject;
use async_graphql::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
// use snafu::ResultExt;
// use sqlx::Connection;

use super::error;
use crate::db::model as db;
// use crate::db::model::ProvideStock;
// use crate::state::State;

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

#[mockall::automock]
#[async_trait]
pub trait StockService {
    async fn list_currencies(&self) -> Result<Vec<Currency>, error::Error>;
}
