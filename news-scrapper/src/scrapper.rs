use std::sync::Arc;

use async_trait::async_trait;
use bytes::Buf;
use bytes::Bytes;
use feed_rs::model::Feed;
use feed_rs::parser;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use log::debug;
use log::error;
use mockall::automock;
use tokio::sync::mpsc::Sender;
use tokio::sync::Semaphore;
use utils::error::CommonError;
use utils::error::ASYNC_OPERATIONS_ERROR_CODE;

use utils::error::SERIALIZATION_ERROR_CODE;
use utils::news::models::feed::Feed as RssFeed;
use utils::news::models::news::News;

#[automock]
#[async_trait]
pub trait FeedsScrapper: Send + Sync {
    async fn scrap_all(
        &self,
        feeds: Vec<RssFeed>,
        tx: Sender<Vec<News>>,
    ) -> Result<(), CommonError>;
}

#[automock]
#[async_trait]
pub trait RssFetcher: Send + Sync {
    async fn fetch(&self, fetch_url: String) -> Result<Bytes, CommonError>;
}

#[derive(Clone)]
pub struct RssScrapper {
    fetcher: Arc<dyn RssFetcher>,
}

#[async_trait]
impl FeedsScrapper for RssScrapper {
    async fn scrap_all(
        &self,
        feeds: Vec<RssFeed>,
        tx: Sender<Vec<News>>,
    ) -> Result<(), CommonError> {
        let max_concurrency = 5; // Define the maximum concurrency limit

        let semaphore = Semaphore::new(max_concurrency);
        let mut tasks = FuturesUnordered::new();

        for current_feed in &feeds {
            let permit = semaphore.acquire().await.map_err(|e| CommonError {
                message: format!("Semaphore acquisition failed: {}", e),
                code: ASYNC_OPERATIONS_ERROR_CODE,
            })?;
            let task = self.scrap_with_retry(current_feed.clone(), permit);
            tasks.push(task);
        }

        while let Some(result) = tasks.next().await {
            match result {
                Ok((rss_feed, feed)) => {
                    debug!("Got new entries for feed {}", rss_feed.title);
                    let news = feed
                        .entries
                        .into_iter()
                        .map(|feed_news| {
                            let mut news_entry: News = feed_news.into();
                            news_entry.feed_id = rss_feed.id;

                            news_entry
                        })
                        .collect();

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
}

impl RssScrapper {
    pub fn new(fetcher: Arc<dyn RssFetcher>) -> Self {
        RssScrapper { fetcher }
    }

    async fn scrap_with_retry(
        &self,
        rss_feed: RssFeed,
        permit: tokio::sync::SemaphorePermit<'_>,
    ) -> Result<(RssFeed, Feed), (RssFeed, CommonError)> {
        let mut retry_count = 0;

        loop {
            let result = self.scrap(rss_feed.url.to_string()).await;

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

    async fn scrap(&self, feed_url: String) -> Result<Feed, CommonError> {
        let xml = self.fetcher.fetch(feed_url).await?;

        parser::parse(xml.reader()).map_err(|err| CommonError {
            message: err.to_string(),
            code: SERIALIZATION_ERROR_CODE,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio::sync::Semaphore;
    use tokio::task;
    use utils::news::models::feed::Feed as RssFeed;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_scrap_success() {
        let feed_url = "https://example.com/feed.xml";

        // Create an instance of the Fetcher mock
        let mut fetcher = MockRssFetcher::new();

        fetcher.expect_fetch().returning(|_| Ok(r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:googleplay="http://www.google.com/schemas/play-podcasts/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:media="http://search.yahoo.com/mrss/" xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel><title>Crime Junkie</title></channel></rss>"#.into()));

        let fetcher_wrapped = Arc::new(fetcher);

        // Create an instance of RssScrapper and set the fetcher
        let scrapper = RssScrapper {
            fetcher: fetcher_wrapped.clone(),
        };

        // Call the scrap method with the feed_url
        let result = scrapper.scrap(feed_url.to_string()).await;

        // Assert that the result is Ok
        assert!(result.is_ok());

        // Assert that the parser result is equal to the feed in the scrap result
        assert_eq!(
            result.unwrap().title.unwrap().content,
            "Crime Junkie".to_string()
        );
    }

    #[tokio::test]
    async fn test_scrap_error() {
        let feed_url = "https://example.com/feed.xml";

        // Create an instance of the Fetcher mock
        let mut fetcher = MockRssFetcher::new();

        fetcher
            .expect_fetch()
            .returning(|_| Ok(r#"randomresponse"#.into()));

        let fetcher = Arc::new(fetcher);

        // Create an instance of RssScrapper and set the fetcher
        let scrapper = RssScrapper::new(fetcher);

        // Call the scrap method with the feed_url
        let result = scrapper.scrap(feed_url.to_string()).await;

        // Assert that the result is Ok
        assert!(result.is_err());

        let err = result.err().unwrap();

        // Assert that the parser result is equal to the feed in the scrap result
        assert_eq!(
            err.message,
            "unable to parse feed: no root element".to_string()
        );
        assert_eq!(err.code, SERIALIZATION_ERROR_CODE);
    }

    #[tokio::test]
    async fn test_scrap_retry_success_after_first_fail() {
        // Create an instance of the Fetcher mock
        let mut fetcher = MockRssFetcher::new();

        fetcher
            .expect_fetch()
            .returning(|_| {
                Err(CommonError {
                    message: "timeout".to_string(),
                    code: 1,
                })
            })
            .once();
        fetcher.expect_fetch().returning(|_| Ok(r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:googleplay="http://www.google.com/schemas/play-podcasts/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:media="http://search.yahoo.com/mrss/" xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel><title>Crime Junkie</title></channel></rss>"#.into()));

        let fetcher = Arc::new(fetcher);

        // Create an instance of RssScrapper and set the fetcher
        let scrapper = RssScrapper::new(fetcher);

        // Call the scrap method with the feed_url
        let semaphore = Semaphore::new(1);
        let permit = semaphore.acquire().await.unwrap();
        let rss_feed = RssFeed {
            id: Uuid::new_v4(),
            url: "https://example.com/rss".to_string(),
            author: "".to_string(),
            title: "".to_string(),
        };
        let result = scrapper.scrap_with_retry(rss_feed, permit).await;

        // Assert that the result is Ok
        assert!(result.is_ok());

        let rss_feed = result.clone().unwrap().0.clone();
        let xml_feed = result.unwrap().1.clone();
        assert_eq!(rss_feed.url, "https://example.com/rss".to_string());
        assert_eq!(xml_feed.title.unwrap().content, "Crime Junkie".to_string());
    }

    #[tokio::test]
    async fn test_scrap_retry_success() {
        // Create an instance of the Fetcher mock
        let mut fetcher = MockRssFetcher::new();

        fetcher.expect_fetch().returning(|_| Ok(r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:googleplay="http://www.google.com/schemas/play-podcasts/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:media="http://search.yahoo.com/mrss/" xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel><title>Crime Junkie</title></channel></rss>"#.into()));

        let fetcher = Arc::new(fetcher);

        // Create an instance of RssScrapper and set the fetcher
        let scrapper = RssScrapper::new(fetcher);

        // Call the scrap method with the feed_url
        let semaphore = Semaphore::new(1);
        let permit = semaphore.acquire().await.unwrap();
        let rss_feed = RssFeed {
            id: Uuid::new_v4(),
            url: "https://example.com/rss".to_string(),
            author: "".to_string(),
            title: "".to_string(),
        };
        let result = scrapper.scrap_with_retry(rss_feed, permit).await;

        // Assert that the result is Ok
        assert!(result.is_ok());

        let rss_feed = result.clone().unwrap().0.clone();
        let xml_feed = result.unwrap().1.clone();
        assert_eq!(rss_feed.url, "https://example.com/rss".to_string());
        assert_eq!(xml_feed.title.unwrap().content, "Crime Junkie".to_string());
    }

    #[tokio::test]
    async fn test_scrap_retry_error() {
        // Create an instance of the Fetcher mock
        let mut fetcher = MockRssFetcher::new();

        fetcher.expect_fetch().returning(|_| {
            Err(CommonError {
                message: "timeout".to_string(),
                code: 1,
            })
        });

        let fetcher = Arc::new(fetcher);

        // Create an instance of RssScrapper and set the fetcher
        let scrapper = RssScrapper::new(fetcher);

        // Call the scrap method with the feed_url
        let semaphore = Semaphore::new(1);
        let permit = semaphore.acquire().await.unwrap();
        let rss_feed = RssFeed {
            id: Uuid::new_v4(),
            url: "https://example.com/rss".to_string(),
            author: "".to_string(),
            title: "".to_string(),
        };
        let result = scrapper.scrap_with_retry(rss_feed, permit).await;

        // Assert that the result is Ok
        assert!(result.is_err());

        assert_eq!(result.err().unwrap().1.message, "timeout".to_string());
    }

    #[tokio::test]
    async fn test_scrap_all() {
        let feeds = vec![
            RssFeed {
                id: Uuid::new_v4(),
                url: "https://example.com/rss1".to_string(),
                author: "".to_string(),
                title: "".to_string(),
            },
            RssFeed {
                id: Uuid::new_v4(),
                url: "https://example.com/rss2".to_string(),
                author: "".to_string(),
                title: "".to_string(),
            },
        ];

        let (tx, mut rx) = mpsc::channel::<Vec<News>>(10);

        // Create an instance of the Fetcher mock
        let mut fetcher = MockRssFetcher::new();

        fetcher.expect_fetch().returning(|_| Ok(r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:googleplay="http://www.google.com/schemas/play-podcasts/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:media="http://search.yahoo.com/mrss/" xmlns:podcast="https://podcastindex.org/namespace/1.0">
            <channel>
                <title>Crime Junkie</title>
                <item>
                    <title>MURDERED: Deanna Cook</title>
                </item>
                <item>
                    <title>MYSTERIOUS DEATH OF: Morgan Patten</title>
                </item>
            </channel>
        </rss>"#.into())).once();
        fetcher.expect_fetch().returning(|_| Ok(r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:googleplay="http://www.google.com/schemas/play-podcasts/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:media="http://search.yahoo.com/mrss/" xmlns:podcast="https://podcastindex.org/namespace/1.0">
        <channel>
            <title>Crime Junkie</title>
            <item>
                <title>MURDERED: Deanna Cook</title>
            </item>
        </channel>
    </rss>"#.into()));

        let fetcher = Arc::new(fetcher);

        // Create an instance of RssScrapper and set the fetcher
        let scrapper = RssScrapper::new(fetcher);

        // Call the function under test
        task::spawn(async move {
            let result = scrapper.scrap_all(feeds, tx).await;
            assert!(result.is_ok());
        });

        // Assert the expected number of news received
        let mut total_news = 0;
        while let Some(news) = rx.recv().await {
            total_news += news.len();
        }

        assert_eq!(total_news, 3 /*expected number of news received*/);
    }

    #[tokio::test]
    async fn test_scrap_all_one_feed_not_synced() {
        let feeds = vec![
            RssFeed {
                id: Uuid::new_v4(),
                url: "https://example.com/rss1".to_string(),
                author: "".to_string(),
                title: "".to_string(),
            },
            RssFeed {
                id: Uuid::new_v4(),
                url: "https://example.com/rss2".to_string(),
                author: "".to_string(),
                title: "".to_string(),
            },
        ];

        let (tx, mut rx) = mpsc::channel::<Vec<News>>(10);

        // Create an instance of the Fetcher mock
        let mut fetcher = MockRssFetcher::new();

        fetcher.expect_fetch().returning(|_| Ok(r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:googleplay="http://www.google.com/schemas/play-podcasts/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:media="http://search.yahoo.com/mrss/" xmlns:podcast="https://podcastindex.org/namespace/1.0">
            <channel>
                <title>Crime Junkie</title>
                <item>
                    <title>MURDERED: Deanna Cook</title>
                </item>
                <item>
                    <title>MYSTERIOUS DEATH OF: Morgan Patten</title>
                </item>
            </channel>
        </rss>"#.into())).once();
        fetcher.expect_fetch().returning(|_| {
            Err(CommonError {
                message: "timeout".to_string(),
                code: 0,
            })
        });

        let fetcher = Arc::new(fetcher);

        // Create an instance of RssScrapper and set the fetcher
        let scrapper = RssScrapper::new(fetcher);

        // Call the function under test
        task::spawn(async move {
            let result = scrapper.scrap_all(feeds, tx).await;
            assert!(result.is_ok());
        });

        // Assert the expected number of news received
        let mut total_news = 0;
        while let Some(news) = rx.recv().await {
            total_news += news.len();
        }

        assert_eq!(total_news, 2 /*expected number of news received*/);
    }
}
