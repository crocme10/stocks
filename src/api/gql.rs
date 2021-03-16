use async_graphql::extensions::Tracing;
use async_graphql::*;
use tracing::{info, instrument};
// use uuid::Uuid;

use crate::api::model::{self, StockService};

pub struct Query;

#[Object]
impl Query {
    async fn list_currencies(&self, context: &Context<'_>) -> FieldResult<Vec<model::Currency>> {
        // let request_id = Uuid::new_v4();
        // let request_span = tracing::info_span!(
        //     "Request currencies list",
        //     %request_id);
        // let _request_span_guard = request_span.enter();
        let service = get_service_from_context(context)?;
        service.list_currencies().await.map_err(|e| e.extend())
    }
    #[instrument(skip(self, context))]
    async fn find_currency(
        &self,
        context: &Context<'_>,
        code: String,
    ) -> FieldResult<Option<model::Currency>> {
        // let request_id = Uuid::new_v4();
        // let request_span = tracing::info_span!(
        //     "Request currency search",
        //     %request_id,
        //     code = %code);
        // let _request_span_guard = request_span.enter();
        let service = get_service_from_context(context)?;
        service.find_currency(&code).await.map_err(|e| e.extend())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    #[instrument(skip(self, context))]
    async fn add_currency(
        &self,
        context: &Context<'_>,
        currency: CurrencyInput,
    ) -> FieldResult<model::Currency> {
        //info!("Request for adding a currency");
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
        .extension(Tracing::default())
        .data(service)
        .finish()
}

#[allow(clippy::borrowed_box)]
pub fn get_service_from_context<'ctx>(
    context: &'ctx Context,
) -> Result<&'ctx Box<dyn StockService + Send + Sync>, async_graphql::Error>
where
{
    context.data::<Box<dyn StockService + Send + Sync>>()
}

#[derive(Debug, InputObject)]
struct CurrencyInput {
    code: String,
    name: String,
    decimals: i32,
}

#[cfg(test)]
mod tests {
    use super::model;
    use super::*;
    use serde_json::Value;
    use std::convert::Infallible;
    use warp::Filter;

    // TODO How to create a function to return graphql_post, so we don't repeat it.
    #[tokio::test]
    async fn test_add_currency() {
        let mut service = model::MockStockService::new();
        service
            .expect_add_currency()
            .times(1)
            .returning(|code, name, decimals| {
                Ok(model::Currency {
                    code: String::from(code),
                    name: String::from(name),
                    decimals,
                })
            });

        let schema = schema(Box::new(service));

        let graphql_post = async_graphql_warp::graphql(schema).and_then(
            |(schema, request): (StocksSchema, async_graphql::Request)| async move {
                Ok::<_, Infallible>(async_graphql_warp::Response::from(
                    schema.execute(request).await,
                ))
            },
        );
        let query = r#" "mutation addCurrency($currency: CurrencyInput!) { addCurrency(currency: $currency) { code, name, decimals } }" "#;
        let variables = r#" { "code": "EUR", "name": "Euro", "decimals": 2 }"#;
        let body = format!(
            r#"{{ "query": {query}, "variables": {{ "currency": {variables} }} }}"#,
            query = query,
            variables = variables
        );

        let resp = warp::test::request()
            .method("POST")
            .body(body)
            .reply(&graphql_post)
            .await;

        assert_eq!(resp.status(), 200);
        let data = resp.into_body();
        let v: Value = serde_json::from_slice(&data).expect("json");
        let c: model::Currency =
            serde_json::from_value(v["data"]["addCurrency"].to_owned()).expect("currency");
        assert_eq!(c.code, "EUR");
    }
}
