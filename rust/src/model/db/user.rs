use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio_pg_mapper_derive::PostgresMapper;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone,  PostgresMapper)]
#[pg_mapper(table = "users")]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}
