use actix::MailboxError;
use serde::Serialize;

pub const DATABASE_ERROR_CODE: u32 = 1;
pub const BROKER_ERROR_CODE: u32 = 2;
pub const HTTP_ERROR_CODE: u32 = 3;
pub const AUTH_TOKEN_ENCODING_CODE: u32 = 4;
pub const WS_ERROR_CODE: u32 = 5;
pub const SERIALIZATION_ERROR_CODE: u32 = 6;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct CommonError {
    pub message: String,
    pub code: u32,
}

impl CommonError {
    pub fn new(message: &str) -> Self {
        CommonError {
            message: message.to_string(),
            code: 0,
        }
    }
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

impl DatabaseError {
    pub fn new(message: &str) -> Self {
        DatabaseError {
            message: message.to_string(),
        }
    }
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

impl BrokerError {
    pub fn new(message: &str) -> Self {
        BrokerError {
            message: message.to_string(),
        }
    }
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

impl HttpError {
    pub fn new(message: &str) -> Self {
        HttpError {
            message: message.to_string(),
        }
    }
}

impl From<HttpError> for CommonError {
    fn from(val: HttpError) -> Self {
        CommonError {
            message: val.message,
            code: HTTP_ERROR_CODE,
        }
    }
}

impl From<MailboxError> for CommonError {
    fn from(val: MailboxError) -> Self {
        CommonError {
            message: val.to_string(),
            code: WS_ERROR_CODE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SerializationError {
    pub message: String,
}

impl SerializationError {
    pub fn new(message: &str) -> Self {
        SerializationError {
            message: message.to_string(),
        }
    }
}

impl From<SerializationError> for CommonError {
    fn from(val: SerializationError) -> Self {
        CommonError {
            message: val.message,
            code: SERIALIZATION_ERROR_CODE,
        }
    }
}
