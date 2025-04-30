use crate::app_state::AppState;
use crate::error::Error;
use crate::model::types::user_token_status::UserTokenStatus;
use crate::model::types::user_token_type::UserTokenType;
use crate::model::user::RefreshTokenRequest;
use crate::model::user_session::UserSession;
use crate::repository::token_repository;
use crate::tools::token;
use anyhow::Context;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{Duration, Utc};
use tokio_postgres::GenericClient;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "auth",
    description = "Refresh token",
    request_body(
        content = RefreshTokenRequest, description = "Refresh token", content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Refresh token", body = UserSession),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    ),
)]
pub async fn refresh_token(
    State(app_state): State<AppState>,
    Json(refresh_token_params): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, Error> {
    let refresh_token_param = refresh_token_params.refresh_token;
    let pool = app_state.db_pool();
    let db_client = pool.get().await.context("Can't get pool connection")?;
    let client = db_client.client();
    let refresh_token =
        token_repository::get_by_token(client, refresh_token_param.as_str()).await?;
    let user_id: Uuid;

    if let Some(refresh_token) = refresh_token {
        if refresh_token.status == UserTokenStatus::INACTIVE {
            token_repository::invalidate_by_user_id(client, refresh_token.user_id).await?;
            return Err(Error::Unauthorized);
        } else if refresh_token.expires_at > Utc::now()
            && refresh_token.token_type == UserTokenType::REFRESH
            && refresh_token.status == UserTokenStatus::ACTIVE
        {
            token_repository::invalidate(client, refresh_token.id).await?;
            user_id = refresh_token.user_id;
        } else {
            return Err(Error::Unauthorized);
        }
    } else {
        return Err(Error::Unauthorized);
    }

    let now = Utc::now();
    let cfg = app_state.cfg.clone();

    let session_token = token_repository::create(
        client,
        now,
        user_id,
        token::generate(),
        UserTokenType::SESSION,
        now + Duration::seconds(cfg.session_lifetime),
    )
    .await?;

    let refresh_token = token_repository::create(
        client,
        now,
        user_id,
        token::generate(),
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
}
