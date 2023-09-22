pub use actix_threadpool::BlockingError;
use diesel::r2d2;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct CommonError {
    pub message: String,
    pub code: u32,
}

impl std::fmt::Display for CommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}, Code: {}", self.message, self.code)
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryError {
    pub message: String,
}

impl From<RepositoryError> for CommonError {
    fn from(val: RepositoryError) -> Self {
        Self {
            message: val.message,
            code: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrokerError {
    pub message: String,
}

impl From<BrokerError> for CommonError {
    fn from(val: BrokerError) -> Self {
        CommonError {
            message: val.message,
            code: 2,
        }
    }
}

#[derive(Debug)]
pub struct DieselRepositoryError(RepositoryError);

impl DieselRepositoryError {
    pub fn into_inner(self) -> RepositoryError {
        self.0
    }
}

impl From<r2d2::Error> for DieselRepositoryError {
    fn from(error: r2d2::Error) -> DieselRepositoryError {
        DieselRepositoryError(RepositoryError {
            message: error.to_string(),
        })
    }
}

impl From<diesel::result::Error> for DieselRepositoryError {
    fn from(error: diesel::result::Error) -> DieselRepositoryError {
        DieselRepositoryError(RepositoryError {
            message: error.to_string(),
        })
    }
}

impl<T: std::fmt::Debug> From<BlockingError<T>> for DieselRepositoryError {
    fn from(error: BlockingError<T>) -> DieselRepositoryError {
        DieselRepositoryError(RepositoryError {
            message: error.to_string(),
        })
    }
}
