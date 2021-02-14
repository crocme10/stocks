use juniper::{EmptyMutation, EmptySubscription, FieldResult, IntoFieldError, RootNode};
use slog::info;
// use uuid::Uuid;

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
    /// Returns a list of currencies
    async fn list_currencies(
        &self,
        context: &Context,
    ) -> FieldResult<model::MultiCurrenciesResponseBody> {
        info!(context.state.logger, "Request for currencies");
        model::list_currencies(context)
            .await
            .map_err(IntoFieldError::into_field_error)
    }
}

// pub struct Mutation;
//
// #[juniper::graphql_object(
//     Context = Context
// )]
// impl Mutation {
//     async fn create_or_update_document(
//         &self,
//         doc: model::DocumentRequestBody,
//         context: &Context,
//     ) -> FieldResult<model::SingleDocResponseBody> {
//         info!(
//             context.state.logger,
//             "Request for document update with id {}", doc.doc.id
//         );
//         model::create_or_update_document(doc, context)
//             .await
//             .map_err(IntoFieldError::into_field_error)
//     }
// }
//
type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}
