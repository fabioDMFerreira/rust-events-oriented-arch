use actix_web::error::BlockingError;
use diesel::r2d2;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommonError {
    pub message: String,
    pub code: u32,
}

impl std::fmt::Display for CommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}, Code: {}", self.message, self.code)
    }
}

#[derive(Debug)]
pub struct RepositoryError {
    pub message: String,
}

impl Into<CommonError> for RepositoryError {
    fn into(self) -> CommonError {
        CommonError {
            message: self.message,
            code: 1,
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

impl From<BlockingError> for DieselRepositoryError {
    fn from(error: BlockingError) -> DieselRepositoryError {
        DieselRepositoryError(RepositoryError {
            message: error.to_string(),
        })
    }
}
