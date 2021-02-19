use async_graphql::*;
use tracing::info;

use crate::api::model;
use crate::state::State;

pub struct Query;

#[Object]
impl Query {
    async fn list_currencies(
        &self,
        context: &Context<'_>,
    ) -> FieldResult<model::MultiCurrenciesResponseBody> {
        info!("Request for currencies");
        model::list_currencies(&get_state_from_context(context))
            .await
            .map_err(|e| e.extend())
    }
}

pub type StocksSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn schema(state: State) -> StocksSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(state)
        .finish()
}

pub fn get_state_from_context<'ctx>(context: &'ctx Context) -> &'ctx State {
    context.data::<State>().expect("Can't get state")
}
