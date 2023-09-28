use crate::news::models::feed::Feed;
use crate::news::schema::subscriptions;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, PartialEq, Associations)]
#[diesel(belongs_to(Feed))]
#[diesel(table_name = subscriptions)]
pub struct Subscription {
    pub feed_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}
