use crate::news::models::feed::Feed;
use crate::news::schema::news;
use crate::serializer::serde_naive_date;
use chrono::NaiveDate;
use diesel::prelude::*;
use feed_rs::model::Entry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Selectable,
    Queryable,
    Insertable,
    PartialEq,
    Identifiable,
    Associations,
)]
#[diesel(belongs_to(Feed))]
#[diesel(table_name = news)]
pub struct News {
    pub id: uuid::Uuid,
    pub author: String,
    pub url: String,
    pub title: String,
    #[serde(with = "serde_naive_date")]
    pub publish_date: Option<NaiveDate>,
    pub feed_id: uuid::Uuid,
}

impl From<Entry> for News {
    fn from(feed_news: Entry) -> Self {
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

        News {
            id: Uuid::new_v4(),
            author,
            url,
            title,
            feed_id: Uuid::new_v4(),
            publish_date: Some(publish_date),
        }
    }
}
