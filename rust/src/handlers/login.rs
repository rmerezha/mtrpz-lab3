use crate::app_state::AppState;
use crate::error::Error;
use crate::model::types::user_token_type::UserTokenType;
use crate::model::user::LoginRequest;
use crate::model::user_session::UserSession;
use crate::repository::token_repository;
use crate::repository::user_repository::find_user_by_username;
use crate::tools::token;
use anyhow::Context;
use axum::extract::State;
use axum::{extract::Json, response::IntoResponse};
use chrono::{Duration, Utc};
use tokio_postgres::GenericClient;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    description = "Login request",
    request_body(
        content = LoginRequest, description = "Login", content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Login response", body = UserSession),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, Error> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(Error::BadRequest);
    }

    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;

    if let Some(user) = find_user_by_username(client.client(), &payload.username).await? {
        let Ok(true) = bcrypt::verify(&payload.password, &user.password_hash) else {
            return Err(Error::Unauthorized);
        };

        let now = Utc::now();
        let session_token = token::generate();
        let refresh_token = token::generate();
        let cfg = app_state.cfg.clone();
        let expires_at = now + Duration::seconds(cfg.session_lifetime);

        let session_token = token_repository::create(
            client.client(),
            now,
            user.id,
            session_token,
            UserTokenType::SESSION,
            expires_at,
        )
        .await?;

        let refresh_token = token_repository::create(
            client.client(),
            now,
            user.id,
            refresh_token,
            UserTokenType::REFRESH,
            now + Duration::seconds(cfg.refresh_token_lifetime),
        )
        .await?;

        let headers = axum::response::AppendHeaders([(
            axum::http::header::SET_COOKIE,
            "session_token=".to_owned()
                + &*session_token.token
                + "; path=/; httponly; secure; samesite=strict",
        )]);

        Ok((
            headers,
            Json(UserSession::new(session_token, refresh_token)),
        ))
    } else {
        Err(Error::BadRequest)
    }
}
