use std::sync::Arc;

use crate::db::PgPool;
use crate::error::DatabaseError;
use diesel::prelude::*;
use mockall::automock;
use uuid::Uuid;

use crate::news::models::news::News;
use crate::news::schema::{feeds, news, subscriptions};

#[automock]
pub trait NewsRepository: Send + Sync {
    fn list(&self) -> Result<Vec<News>, DatabaseError>;
    fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<News>, DatabaseError>;
    fn find_by_id(&self, news_id: Uuid) -> Result<Option<News>, DatabaseError>;
    fn find_by_fields(
        &self,
        title: Option<String>,
        feed_id: Option<Uuid>,
    ) -> Result<Option<News>, DatabaseError>;
    fn create(&self, news: &News) -> Result<News, DatabaseError>;
    fn delete(&self, news_id: Uuid) -> Result<usize, DatabaseError>;
}

pub struct NewsDieselRepository {
    pool: Arc<PgPool>,
}

impl NewsDieselRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        NewsDieselRepository { pool }
    }
}

impl NewsRepository for NewsDieselRepository {
    fn create(&self, news: &News) -> Result<News, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        diesel::insert_into(news::table)
            .values(news)
            .get_result(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn find_by_id(&self, news_id: Uuid) -> Result<Option<News>, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        news::table
            .filter(news::id.eq(news_id))
            .first(&mut conn)
            .optional()
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn find_by_fields(
        &self,
        title: Option<String>,
        feed_id: Option<Uuid>,
    ) -> Result<Option<News>, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        let mut query = news::table.into_boxed();

        if let Some(title) = title {
            query = query.filter(news::title.eq(title));
        }

        if let Some(feed_id) = feed_id {
            query = query.filter(news::feed_id.eq(feed_id));
        }

        query
            .first(&mut conn)
            .optional()
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn list(&self) -> Result<Vec<News>, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        news::table
            .load::<News>(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<News>, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        news::table
            .inner_join(feeds::table.inner_join(subscriptions::table))
            .select(News::as_select())
            .filter(subscriptions::user_id.eq(user_id))
            .order(news::publish_date.desc())
            .load::<News>(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }

    fn delete(&self, news_id: Uuid) -> Result<usize, DatabaseError> {
        let mut conn = self.pool.get().unwrap();

        diesel::delete(news::table.find(news_id))
            .execute(&mut conn)
            .map_err(|err| DatabaseError {
                message: err.to_string(),
            })
    }
}
