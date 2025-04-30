use axum::Router;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use crate::handlers;

use crate::app_state::AppState;

pub fn openapi_documentation(_state: &AppState) -> Router {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            handlers::signup::signup,
            handlers::login::login,
            handlers::refresh_token::refresh_token,
            handlers::note::get_note,
            handlers::note::get_notes,
            handlers::note::search_notes,
            handlers::note::create_note,
            handlers::note::update_note,
            handlers::note::delete_note,
            handlers::note::share_note,
        ),
        info(
            title = "Speer test",
            description = "API documentation",
            version = "0.1.0",
        ),
        tags(
            (name = "auth", description = "Authentication endpoints"),
            (name = "notes", description = "Note management")
        )
    )]
    struct ApiDoc;

    let html = r##"
    <!doctype html>
    <html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, minimum-scale=1, initial-scale=1, user-scalable=yes">
        <script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
    </head>
    <body>
        <rapi-doc
            render-style="focused"
            load-fonts="false"
            show-header="false"
            allow-spec-file-download="true"
            spec-url="/v1/openapi.json"
            default-schema-tab="model"
            theme="light"
            bg-color="#fafafa"
            nav-bg-color="#2f4467"
            nav-text-color="#a9b7d0"
            nav-hover-bg-color="#333f54"
            nav-hover-text-color="#fff"
            nav-accent-color="#92a6cf"
            primary-color="#5c7096"
            show-method-in-nav-bar="as-colored-block"
            use-path-in-nav-bar="false"
            schema-description-expanded="true"
        >
    </rapi-doc>
    </body>
    </html>
    "##;

    let open_api = ApiDoc::openapi();

    Router::new()
        .merge(RapiDoc::with_openapi("/v1/openapi.json", open_api).path("/").custom_html(html))
}
