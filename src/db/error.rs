use juniper::{graphql_value, FieldError, IntoFieldError};
use snafu::Snafu;

use crate::db::model::ProvideError;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("DB Error: {} - {}", msg, source))]
    #[snafu(visibility(pub))]
    DBError { msg: String, source: sqlx::Error },

    #[snafu(display("DB Provide Error: {} - {}", msg, source))]
    #[snafu(visibility(pub))]
    DBProvideError { msg: String, source: ProvideError },

    #[snafu(display("Reqwest Error: {} - {}", msg, source))]
    #[snafu(visibility(pub))]
    ReqwestError { msg: String, source: reqwest::Error },
}

// impl IntoFieldError for Error {
//     fn into_field_error(self) -> FieldError {
//         match self {
//             err @ Error::Environment { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Environment Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//
//             err @ Error::ConfigError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new("Config Error", graphql_value!({ "internal_error": errmsg }))
//             }
//
//             err @ Error::EnvVarError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Environment Variable Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//
//             err @ Error::MiscError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Miscellaneous Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//
//             err @ Error::TokioIOError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Tokio IO Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//
//             err @ Error::IOError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new("IO Error", graphql_value!({ "internal_error": errmsg }))
//             }
//
//             err @ Error::JSONError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new("JSON Error", graphql_value!({ "internal_error": errmsg }))
//             }
//
//             err @ Error::DBError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new("DB Error", graphql_value!({ "internal_error": errmsg }))
//             }
//
//             err @ Error::DBProvideError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Provide Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//
//             err @ Error::ReqwestError { .. } => {
//                 let errmsg = format!("{}", err);
//                 FieldError::new(
//                     "Reqwest Error",
//                     graphql_value!({ "internal_error": errmsg }),
//                 )
//             }
//         }
//     }
// }
