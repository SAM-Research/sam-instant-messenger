use sqlx::Error as SqlxError;
use std::{error::Error, panic::AssertUnwindSafe};

use derive_more::{Display, Error};

type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug, Display, Error)]
#[display("DatabaseError({})", _0.0)]
pub struct DatabaseError(AssertUnwindSafe<BoxDynError>);

impl From<BoxDynError> for DatabaseError {
    fn from(value: BoxDynError) -> Self {
        Self(AssertUnwindSafe(value))
    }
}

impl From<SqlxError> for DatabaseError {
    fn from(value: SqlxError) -> Self {
        Self(AssertUnwindSafe(Box::new(value)))
    }
}
