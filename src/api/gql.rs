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
        let service: &imp::StockServiceImpl = get_service_from_context(context)?;
        service.list_currencies().await.map_err(|e| e.extend())
    }
}

pub type StocksSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn schema<A>(service: A) -> StocksSchema
where
    A: model::StockService + Any + Send + Sync,
{
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(service)
        .finish()
}

pub fn get_service_from_context<'ctx, A>(
    context: &'ctx Context,
) -> Result<&'ctx A, async_graphql::Error>
where
    A: model::StockService + Any + Send + Sync,
{
    context.data::<A>()
}

#[derive(InputObject)]
struct CurrencyInput {
    code: String,
    name: String,
    decimals: u32,
}
