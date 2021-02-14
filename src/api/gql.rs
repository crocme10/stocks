use juniper::{EmptyMutation, EmptySubscription, FieldResult, IntoFieldError, RootNode};
use tracing::info;

use crate::api::model;
use crate::state::State;

#[derive(Debug, Clone)]
pub struct Context {
    pub state: State,
}

impl juniper::Context for Context {}

pub struct Query;

#[juniper::graphql_object(
    Context = Context
)]
impl Query {
    async fn list_currencies(
        &self,
        context: &Context,
    ) -> FieldResult<model::MultiCurrenciesResponseBody> {
        info!("Request for currencies");
        model::list_currencies(context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}
