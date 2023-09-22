pub use actix_threadpool::BlockingError;
use diesel::r2d2;
use utils::error::{CommonError, DatabaseError};

#[derive(Debug)]
pub struct DieselRepositoryError(DatabaseError);

impl DieselRepositoryError {
    pub fn into_inner(self) -> DatabaseError {
        self.0
    }
}

impl From<r2d2::Error> for DieselRepositoryError {
    fn from(error: r2d2::Error) -> DieselRepositoryError {
        DieselRepositoryError(DatabaseError {
            message: error.to_string(),
        })
    }
}

impl From<diesel::result::Error> for DieselRepositoryError {
    fn from(error: diesel::result::Error) -> DieselRepositoryError {
        DieselRepositoryError(DatabaseError {
            message: error.to_string(),
        })
    }
}

impl<T: std::fmt::Debug> From<BlockingError<T>> for DieselRepositoryError {
    fn from(error: BlockingError<T>) -> DieselRepositoryError {
        DieselRepositoryError(DatabaseError {
            message: error.to_string(),
        })
    }
}
