use bytes::{Buf, Bytes};
use feed_rs::model::Feed;
use feed_rs::parser;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use log::debug;
use log::error;
use reqwest::get;
use tokio::sync::mpsc::Sender;
use tokio::sync::Semaphore;
use utils::error::CommonError;
use utils::error::HttpError;
use uuid::Uuid;

use crate::models::feed::Feed as RssFeed;
use crate::models::news::News;

#[derive(Clone)]
pub struct Scrapper {}

impl Scrapper {
    pub async fn scrap_all(feeds: Vec<RssFeed>, tx: Sender<Vec<News>>) -> Result<(), String> {
        let max_concurrency = 5; // Define the maximum concurrency limit

        let semaphore = Semaphore::new(max_concurrency);
        let mut tasks = FuturesUnordered::new();

        for current_feed in &feeds {
            let permit = semaphore
                .acquire()
                .await
                .expect("Semaphore acquisition failed");
            let task = Self::scrap_with_retry(current_feed.clone(), permit);
            tasks.push(task);
        }

        while let Some(result) = tasks.next().await {
            match result {
                Ok((rss_feed, feed)) => {
                    debug!("Got new entries for feed {}", rss_feed.title);
                    let mut news = Vec::new();

                    for feed_news in feed.entries {
                        let mut author = "".to_string();
                        let mut url = "".to_string();
                        let mut title = "".to_string();
                        let mut publish_date = chrono::Utc::now().naive_local().date();

                        if !feed_news.authors.is_empty() {
                            author = feed_news.authors[0].name.clone();
                        }

                        if let Some(source) = feed_news.source {
                            url = source;
                        }

                        if let Some(news_title) = feed_news.title {
                            title = news_title.content.to_string();
                        }

                        if let Some(date) = feed_news.published {
                            publish_date = date.naive_local().date();
                        }

                        news.push(News {
                            id: Uuid::new_v4(),
                            author,
                            url,
                            title,
                            feed_id: rss_feed.id,
                            publish_date: Some(publish_date),
                        })
                    }

                    if let Err(err) = tx.send(news).await {
                        error!(
                            "could not process entries from feed {}: {}",
                            rss_feed.title, err
                        )
                    }
                }
                Err((feed, err)) => {
                    error!("Failed getting feed {}: {}", feed.title, err);
                }
            }
        }

        Ok(())
    }

    async fn scrap_with_retry(
        rss_feed: RssFeed,
        permit: tokio::sync::SemaphorePermit<'_>,
    ) -> Result<(RssFeed, Feed), (RssFeed, CommonError)> {
        let mut retry_count = 0;

        loop {
            let result = Scrapper::scrap(&rss_feed).await;

            if let Ok(feed) = result {
                drop(permit); // Release the semaphore permit
                return Ok((rss_feed, feed));
            }

            retry_count += 1;

            if retry_count >= 3 {
                drop(permit); // Release the semaphore permit
                return Err((rss_feed, result.unwrap_err()));
            }
        }
    }

    async fn scrap(rss_feed: &RssFeed) -> Result<Feed, CommonError> {
        let xml = http_request(rss_feed.url.clone()).await?;

        let feed = parser::parse(xml.reader()).unwrap();

        Ok(feed)
    }
}

async fn http_request(url: String) -> Result<Bytes, HttpError> {
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
