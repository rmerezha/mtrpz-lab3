use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::extract::{Extension, Json, Query};
use tokio_postgres::GenericClient;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::error::Error;
use crate::model::db::note::Note;
use crate::model::db::user::User;
use crate::model::note::{CreateNoteRequest, SearchQuery, ShareNoteRequest, UpdateNoteRequest};
use crate::repository;
use crate::repository::note_repository;

#[utoipa::path(
    get,
    path = "/api/notes",
    tag = "notes",
    description = "List of all notes for current user",
    responses(
        (status = 200, description = "List of notes for current user", body = Vec<Note>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn get_notes(
    Extension(user_data): Extension<Option<User>>,
    State(app_state): State<AppState>,
) -> Result<Json<Vec<Note>>, Error> {
    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;

    if let Some(user) = user_data {
        let notes = note_repository::list_notes_by_user_id(client.client(), &user.id).await?;

        Ok(Json(notes))
    } else {
        Err(Error::BadRequest)
    }
}

#[utoipa::path(
    get,
    path = "/api/notes/{id}",
    tag = "notes",
    description = "Get notes by id",
    responses(
        (status = 200, description = "Note for current user", body = Note),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn get_note(
    Extension(user_data): Extension<Option<User>>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Note>, Error> {
    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;
    let Some(user) = user_data else {
        return Err(Error::Unauthorized);
    };

    let Some(note) = note_repository::get_note(client.client(), &id, &user.id).await? else {
        return Err(Error::Forbidden);
    };

    Ok(Json(note))
}

#[utoipa::path(
    get,
    path = "/api/notes/search",
    tag = "notes",
    description = "Search for notes",
    params(
        SearchQuery,
    ),
    responses(
        (status = 200, description = "Found notes", body = Vec<Note>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn search_notes(
    Extension(user_data): Extension<Option<User>>,
    State(app_state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<Note>>, Error> {
    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;
    let Some(user) = user_data else {
        return Err(Error::Unauthorized);
    };

    if params.q.len() < 3 {
        return Ok(Json(vec![]));
    }

    let res = repository::note_repository::search_notes(client.client(), &user.id, params.q).await?;
    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/notes",
    tag = "notes",
    description = "Create new note",
    request_body(
        content = CreateNoteRequest, description = "Create Note", content_type = " application/json"
    ),
    responses(
        (status = 200, description = "Empty response", body = Note),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn create_note(
    Extension(user_data): Extension<Option<User>>,
    State(app_state): State<AppState>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<Json<Note>, Error> {
    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;
    let Some(user) = user_data else {
        return Err(Error::Unauthorized);
    };

    let note = repository::note_repository::create_note(client.client(), user.id, &payload.title, &payload.content).await?;
    Ok(Json(note))
}

#[utoipa::path(
    put,
    path = "/api/notes/{id}",
    tag = "notes",
    description = "Update note",
    request_body(
        content = UpdateNoteRequest, description = "Update note", content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Updated note", body = Note),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn update_note(
    Path(id): Path<Uuid>,
    Extension(user_data): Extension<Option<User>>,
    State(app_state): State<AppState>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<Note>, Error> {
    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;
    let mut updated = false;

    let Some(user) = user_data else {
        return Err(Error::Unauthorized);
    };

    let Some(mut note) = note_repository::get_note(client.client(), &id, &user.id).await? else {
        return Err(Error::Forbidden);
    };

    if note.user_id != user.id {
        //Only owner can update its notes
        return Err(Error::Forbidden);
    }

    if let Some(title) = payload.title {
        if title != note.title {
            note.title = title;
            updated = true;
        }
    }

    if let Some(content) = payload.content {
        if content != note.content {
            note.content = content;
            updated = true;
        }
    }

    if updated {
        note = repository::note_repository::update_note(client.client(), &mut note).await?;
    }

    Ok(Json(note))
}

#[utoipa::path(
    delete,
    path = "/api/notes/{id}",
    tag = "notes",
    description = "Delete note by id",
    responses(
        (status = 200, description = "Empty response"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn delete_note(
    Path(id): Path<Uuid>,
    Extension(user_data): Extension<Option<User>>,
    State(app_state): State<AppState>,
) -> Result<(), Error> {
    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;

    let Some(user) = user_data else {
        return Err(Error::Unauthorized);
    };

    let Some(note) = note_repository::get_note(client.client(), &id, &user.id).await? else {
        return Ok(());
    };

    if note.user_id != user.id {
        //only owner can delete its notes
        return Err(Error::Forbidden);
    }

    note_repository::delete_note(client.client(), id).await?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/notes/{id}",
    tag = "notes",
    description = "Share note with user",
    request_body(
        content = ShareNoteRequest,
    ),
    responses(
        (status = 200, description = "Empty response"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server error"),
    )
)]
pub async fn share_note(
    Path(id): Path<Uuid>,
    Extension(user_data): Extension<Option<User>>,
    State(app_state): State<AppState>,
    Json(payload): Json<ShareNoteRequest>,
) -> Result<(), Error> {
    let pool = app_state.db_pool();
    let client = pool.get().await.context("Can't get pool connection")?;

    let Some(user) = user_data else {
        return Err(Error::Unauthorized);
    };

    let Some(note) = note_repository::get_note(client.client(), &id, &user.id).await? else {
        return Err(Error::Forbidden);
    };

    if note.user_id != user.id {
        //only owner can share its notes
        return Err(Error::Forbidden);
    }

    note_repository::share_note(client.client(), id, payload.target_user_id).await?;
    Ok(())
}
