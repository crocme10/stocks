use async_graphql::*;
use std::any::Any;
use tracing::info;

use crate::api::imp;
use crate::api::model;
use crate::api::model::StockService;

pub struct Query;

#[Object]
impl Query {
    async fn list_currencies(&self, context: &Context<'_>) -> FieldResult<Vec<model::Currency>> {
        info!("Request for currencies");
        let service = get_service_from_context(context)?;

        service.list_currencies().await.map_err(|e| e.extend())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn add_currency(
        &self,
        context: &Context<'_>,
        currency: CurrencyInput,
    ) -> FieldResult<model::Currency> {
        info!("Request for adding a currency");
        //let service: &imp::StockServiceImpl = get_service_from_context(context)?;
        let service = get_service_from_context(context)?;
        service
            .add_currency(&currency.code, &currency.name, currency.decimals)
            .await
            .map_err(|e| e.extend())
    }
}

pub type StocksSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn schema(service: Box<dyn StockService + Send + Sync>) -> StocksSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(service)
        .finish()
}

pub fn get_service_from_context<'ctx>(
    context: &'ctx Context,
) -> Result<&'ctx Box<dyn StockService + Send + Sync>, async_graphql::Error>
where
{
    context.data::<Box<dyn StockService + Send + Sync>>()
}

#[derive(InputObject)]
struct CurrencyInput {
    code: String,
    name: String,
    decimals: i32,
}
