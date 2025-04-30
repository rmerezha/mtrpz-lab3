use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;


#[derive(Deserialize, ToSchema)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ShareNoteRequest {
    pub target_user_id: Uuid,
}

#[derive(Deserialize, IntoParams)]
pub struct SearchQuery {
    pub q: String,
}
