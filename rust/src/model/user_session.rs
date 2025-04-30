use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::model::db::user_token::UserToken;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserSession {
    pub token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

impl UserSession {
    pub fn new(session_token: UserToken, refresh_token: UserToken) -> Self {
        UserSession {
            user_id: session_token.user_id,
            token: session_token.token,
            refresh_token: refresh_token.token,
            expires_at: session_token.expires_at,
        }
    }
}
