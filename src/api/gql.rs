use async_graphql::*;
use tracing::info;

use crate::api::model;
use crate::state::State;

pub struct Query;

#[Object]
impl Query {
    async fn list_currencies(&self, context: &Context<'_>) -> FieldResult<Vec<model::Currency>> {
        info!("Request for currencies");
        model::list_currencies(&get_state_from_context(context))
            .await
            .map_err(|e| e.extend())
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
        info!("Request for adding a currency {}", &currency.code);
        model::add_currency(
            &get_state_from_context(context),
            &currency.code,
            &currency.name,
            currency.decimals,
        )
        .await
        .map_err(|e| e.extend())
    }
}

pub type StocksSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn schema(state: State) -> StocksSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(state)
        .finish()
}

pub fn get_state_from_context<'ctx>(context: &'ctx Context) -> &'ctx State {
    context.data::<State>().expect("Can't get state")
}

#[derive(InputObject)]
struct CurrencyInput {
    code: String,
    name: String,
    decimals: u32,
}
