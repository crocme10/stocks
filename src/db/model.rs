use async_trait::async_trait;
use snafu::Snafu;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct CurrencyEntity {
    pub code: String,
    pub name: String,
    pub decimals: i32,
}

#[mockall::automock]
#[async_trait]
pub trait ProvideStock {
    async fn list_currencies(&mut self) -> ProvideResult<Vec<CurrencyEntity>>;

    async fn add_currency(
        &mut self,
        code: &str,
        name: &str,
        decimals: i32,
    ) -> ProvideResult<CurrencyEntity>;

    async fn find_currency(&mut self, code: &str) -> ProvideResult<Option<CurrencyEntity>>;
}

pub type ProvideResult<T> = Result<T, ProvideError>;

/// An error returned by a provider
#[derive(Debug, Snafu)]
pub enum ProvideError {
    /// The requested entity does not exist
    #[snafu(display("Entity does not exist"))]
    #[snafu(visibility(pub))]
    NotFound,

    /// The operation violates a uniqueness constraint
    #[snafu(display("Operation violates uniqueness constraint: {}", details))]
    #[snafu(visibility(pub))]
    UniqueViolation { details: String },

    /// The requested operation violates the data model
    #[snafu(display("Operation violates model: {}", details))]
    #[snafu(visibility(pub))]
    ModelViolation { details: String },

    /// The requested operation violates the data model
    #[snafu(display("UnHandled Error: {}", source))]
    #[snafu(visibility(pub))]
    UnHandledError { source: sqlx::Error },
}

impl From<sqlx::Error> for ProvideError {
    /// Convert a SQLx error into a provider error
    ///
    /// For Database errors we attempt to downcast
    ///
    /// FIXME(RFC): I have no idea if this is sane
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ProvideError::NotFound,
            sqlx::Error::Database(db_err) => {
                if let Some(pg_err) = db_err.try_downcast_ref::<sqlx::postgres::PgDatabaseError>() {
                    if let Ok(provide_err) = ProvideError::try_from(pg_err) {
                        provide_err
                    } else {
                        ProvideError::UnHandledError {
                            source: sqlx::Error::Database(db_err),
                        }
                    }
                } else {
                    ProvideError::UnHandledError {
                        source: sqlx::Error::Database(db_err),
                    }
                }
            }
            _ => ProvideError::UnHandledError { source: e },
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use mockall::predicate::*;

    #[tokio::test]
    async fn test_mocked_add() {}
}
