use axum::{
    middleware,
    routing::post,
    Extension, Router,
};
use axum::routing::get;
use deadpool_postgres::Pool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace;
use tracing::Level;

use crate::app_state::AppState;
use crate::config::Config;
use crate::handlers::login::login;
use crate::handlers::signup::signup;
use crate::middleware::auth::{check_auth, inject_user_data};
use crate::openapi_docs::openapi_documentation;
use tower_http::trace::TraceLayer;
use crate::handlers::note::{create_note, delete_note, get_note, get_notes, search_notes, share_note, update_note};
use crate::handlers::refresh_token::refresh_token;
use crate::model::db::user::User;

fn public_router(app_state: AppState) -> Router {
    let user_data: Option<User> = None;

    Router::new()
        .route("/api/auth/refresh", post(refresh_token))
        .route("/api/notes", get(get_notes).post(create_note))
        .route(
            "/api/notes/{id}",
                get(get_note)
                .put(update_note)
                .delete(delete_note),
        )
        .route("/api/notes/{id}/share", post(share_note))
        .route("/api/notes/search", get(search_notes))
        .layer(CorsLayer::new().allow_origin(Any))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            check_auth,
        ))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            inject_user_data,
        ))
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .with_state(app_state)
        .layer(Extension(user_data))
}

pub fn router(cfg: Config, connection_pool: Pool) -> Result<Router, anyhow::Error> {
    let app_state = AppState {
        cfg,
        connection_pool,
    };

    Ok(Router::new()
        .merge(openapi_documentation(&app_state))
        .merge(public_router(app_state.clone())))
}

pub fn enable_http_tracing(router: Router) -> Router {
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    )
}
