use crate::schema::news;
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utils::serializer::serde_naive_date;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, PartialEq)]
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
