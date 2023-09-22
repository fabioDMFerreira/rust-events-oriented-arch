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
use uuid::Uuid;

use crate::models::feed::Feed as RssFeed;
use crate::models::news::News;

#[derive(Clone)]
pub struct Scrapper {
    pub feeds: Vec<RssFeed>,
}

impl Scrapper {
    pub fn new(feeds: Vec<RssFeed>) -> Scrapper {
        Scrapper { feeds }
    }

    pub async fn scrap_all(&mut self, tx: Sender<Vec<News>>) -> Result<(), String> {
        let max_concurrency = 5; // Define the maximum concurrency limit

        let semaphore = Semaphore::new(max_concurrency);
        let mut tasks = FuturesUnordered::new();

        for current_feed in &self.feeds {
            let permit = semaphore
                .acquire()
                .await
                .expect("Semaphore acquisition failed");
            let task = Scrapper::scrap_with_retry(current_feed.clone(), permit);
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
    ) -> Result<(RssFeed, Feed), (RssFeed, String)> {
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

    async fn scrap(rss_feed: &RssFeed) -> Result<Feed, String> {
        let xml = http_request(rss_feed.url.clone())
            .await
            .expect("Failed to send request");

        let feed = parser::parse(xml.reader()).unwrap();

        Ok(feed)
    }
}

async fn http_request(url: String) -> Result<Bytes, String> {
    // Send an HTTP GET request to a URL
    let response = get(url).await.expect("Failed to send request");

    // Check if the request was successful
    if response.status().is_success() {
        // Read the response body as a string
        let body = response
            .bytes()
            .await
            .expect("Failed to read response body");

        return Ok(body);
    }

    Err(format!("Request was not successful: {}", response.status()))
}
