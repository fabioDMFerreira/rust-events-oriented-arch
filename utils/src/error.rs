use serde::Serialize;

const DATABASE_ERROR_CODE: u32 = 1;
const BROKER_ERROR_CODE: u32 = 2;
const HTTP_ERROR_CODE: u32 = 3;

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
pub struct DatabaseError {
    pub message: String,
}

impl From<DatabaseError> for CommonError {
    fn from(val: DatabaseError) -> Self {
        Self {
            message: val.message,
            code: DATABASE_ERROR_CODE,
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
            code: BROKER_ERROR_CODE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub message: String,
}

impl From<HttpError> for CommonError {
    fn from(val: HttpError) -> Self {
        CommonError {
            message: val.message,
            code: HTTP_ERROR_CODE,
        }
    }
}
