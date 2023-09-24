use crate::schema::feeds;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, PartialEq, Identifiable)]
#[diesel(table_name = feeds)]
pub struct Feed {
    pub id: uuid::Uuid,
    pub author: String,
    pub title: String,
    pub url: String,
}
