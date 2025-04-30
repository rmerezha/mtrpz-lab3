use anyhow::Context;
use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use chrono::Utc;
use headers::Cookie;
use tokio_postgres::GenericClient;
use crate::app_state::AppState;
use crate::error::Error;
use crate::model::db::user::User;
use crate::model::types::user_token_status::UserTokenStatus;
use crate::model::types::user_token_type::UserTokenType;
use crate::repository::token_repository::get_by_token;
use crate::repository::user_repository::find_user_by_id;

pub async fn inject_user_data(
    State(app_state): State<AppState>,
    cookie: Option<TypedHeader<Cookie>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, Error> {
    let mut session_token: Option<String> = None;

    if let Some(cookie) = cookie {
        if let Some(s) = cookie.get("session_token") {
            session_token = Some(s.to_string());
        }
    }

    if session_token.is_none() {
        let auth_header = request.headers().get("authorization");

        if let Some(s) = auth_header {
            session_token = Some(s.to_str().unwrap_or_default().to_string())
        }
    }

    if let Some(session_token) = session_token {
        let _cfg = app_state.cfg.clone();
        let pool = app_state.db_pool();
        let db_client = pool.get().await.context("Can't get pool connection")?;
        let user_session = get_by_token(db_client.client(), session_token.as_str()).await;

        if let Ok(Some(user_session)) = user_session {
            if user_session.expires_at > Utc::now()
                && user_session.status == UserTokenStatus::ACTIVE
                && user_session.token_type == UserTokenType::SESSION {
                let user: Result<Option<User>, _> = find_user_by_id(db_client.client(), &user_session.user_id).await;

                if let Ok(Some(user)) = user {
                    request.extensions_mut().insert(Some(user));
                }
            }
        }
    }

    Ok(next.run(request).await)
}

pub async fn check_auth(request: Request<Body>, next: Next) -> Result<impl IntoResponse, Error> {
    if request
        .extensions()
        .get::<Option<User>>()
        .ok_or("check_auth: extensions have no UserData").unwrap()
        .is_some()
    {
        Ok(next.run(request).await)
    } else {
        Err(Error::Unauthorized)
    }
}
