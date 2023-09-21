use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, PartialEq)]
#[diesel(table_name = users)]
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: String,
}
