use async_trait::async_trait;

use crate::error::CommonError;

#[async_trait]
pub trait Processor: Send + Sync {
    async fn process(&self, message: &str) -> Result<(), CommonError>;
}
