use crate::schema::subscriptions;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, PartialEq)]
#[diesel(table_name = subscriptions)]
pub struct Subscription {
    pub feed_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}
