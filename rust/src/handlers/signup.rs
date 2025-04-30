use anyhow::Context;
use axum::extract::State;
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};
use bcrypt::DEFAULT_COST;
use tokio_postgres::GenericClient;

use crate::app_state::AppState;
use crate::error::Error;
use crate::model::user::SignupRequest;
use crate::repository::user_repository::{create_user, find_user_by_username};

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    tag = "auth",
    description = "Sign up request",
    request_body(
        content = SignupRequest, content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Account created, empty response"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn signup(
    State(app_state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, Error> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(Error::BadRequest);
    }

    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;

    if let Ok(Some(_)) = find_user_by_username(client.client(), &payload.username).await {
        return Err(Error::BadRequest);
    }

    let hash_result = bcrypt::hash(payload.password, DEFAULT_COST);

    if let Ok(password_hash) = hash_result {
        let _ = create_user(client.client(), &payload.username, &password_hash).await?;

        Ok(StatusCode::CREATED)
    } else {
        Err(Error::BadRequest)
    }
}
