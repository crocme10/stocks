// use juniper::{graphql_value, FieldError, IntoFieldError};
// use async_graphql::{ErrorExtensions, FieldError, FieldResult, Object, ResultExt};
use async_graphql::{ErrorExtensions, FieldError};
use snafu::Snafu;

use crate::db::model::ProvideError;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("DB Error: {} - {}", msg, source))]
    #[snafu(visibility(pub))]
    DBConnectionError { msg: String, source: sqlx::Error },

    #[snafu(display("DB Provide Error: {} - {}", msg, source))]
    #[snafu(visibility(pub))]
    DBTransactionError { msg: String, source: sqlx::Error },

    #[snafu(display("DB Provide Error: {} - {}", msg, source))]
    #[snafu(visibility(pub))]
    DBProvideError { msg: String, source: ProvideError },
}

impl ErrorExtensions for Error {
    // lets define our base extensions
    fn extend(&self) -> FieldError {
        self.extend_with(|err, e| match err {
            Error::DBConnectionError { msg, .. } => e.set("reason", msg.clone()),
            Error::DBTransactionError { msg, .. } => e.set("reason", msg.clone()),
            Error::DBProvideError { msg, .. } => e.set("reason", msg.clone()),
        })
    }
}

// impl IntoFieldError for Error {
//     fn into_field_error(self) -> FieldError {
//         match self {
//             err @ Error::DBConnectionError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Database Connection Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//             err @ Error::DBTransactionError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Database Transaction Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//             err @ Error::DBProvideError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Database Provide Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//         }
//     }
// }
