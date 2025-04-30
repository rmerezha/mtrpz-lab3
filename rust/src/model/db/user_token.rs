extern crate tokio_pg_mapper;
extern crate tokio_pg_mapper_derive;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::model::types::user_token_status::UserTokenStatus;
use crate::model::types::user_token_type::UserTokenType;

#[derive(Serialize, Deserialize, Default, PostgresMapper, ToSchema, Clone)]
#[pg_mapper(table = "user_token")]
pub struct UserToken {
    pub id: Uuid,
    pub created_at: chrono::DateTime<Utc>,
    pub user_id: Uuid,
    pub token: String,
    pub token_type: UserTokenType,
    pub status: UserTokenStatus,
    pub expires_at: chrono::DateTime<Utc>,
}
