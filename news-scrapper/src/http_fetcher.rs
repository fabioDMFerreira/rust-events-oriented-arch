use async_trait::async_trait;
use bytes::Bytes;
use reqwest::get;
use utils::error::{CommonError, HttpError};

use crate::scrapper::RssFetcher;

pub struct HttpFetcher {}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpFetcher {
    pub fn new() -> Self {
        HttpFetcher {}
    }
}

#[async_trait]
impl RssFetcher for HttpFetcher {
    async fn fetch(&self, fetch_url: String) -> Result<Bytes, CommonError> {
        http_request(fetch_url).await.map_err(|e| e.into())
    }
}

pub async fn http_request(url: String) -> Result<Bytes, HttpError> {
    // Send an HTTP GET request to a URL
    let response = get(url).await.map_err(|v| HttpError {
        message: format!("failed to send request: {}", v),
    })?;

    // Check if the request was successful
    if response.status().is_success() {
        // Read the response body as a string
        let body = response.bytes().await.map_err(|v| HttpError {
            message: format!("failed to read response body: {}", v),
        })?;

        return Ok(body);
    }

    Err(HttpError {
        message: format!("Request was not successful: {}", response.status().as_str()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_request_success() {
        let mut server = mockito::Server::new();

        // Arrange
        let expected_body = "Hello, World!";
        let _m = server
            .mock("GET", "/")
            .with_body(expected_body)
            .with_status(200)
            .create();

        // Act
        let result = http_request(server.url().to_string()).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_body.as_bytes().to_vec());
    }

    #[tokio::test]
    async fn test_http_request_failure() {
        let mut server = mockito::Server::new();

        // Arrange
        let _m = server.mock("GET", "/").with_status(500).create();

        // Act
        let result = http_request(server.url().to_string()).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            format!("Request was not successful: {}", 500)
        );
    }

    #[tokio::test]
    async fn test_http_request_error() {
        // Arrange
        let url = "invalid url";

        // Act
        let result = http_request(url.to_string()).await;

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("failed to send request:"));
    }
}
