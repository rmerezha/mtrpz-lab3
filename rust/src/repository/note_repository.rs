use crate::model::db::note::Note;
use crate::tools::utils;
use tokio_pg_mapper::Error;
use tokio_postgres::GenericClient;
use uuid::Uuid;

pub async fn create_note(
    client: &impl GenericClient,
    user_id: Uuid,
    title: &str,
    content: &str,
) -> Result<Note, Error> {
    let row = client
        .query_one(
            "
            INSERT INTO notes (user_id, title, content)
            VALUES ($1, $2, $3)
            RETURNING *",
            &[&user_id, &title, &content],
        )
        .await?;

    Ok(Note::from_row(row)?)
}

pub async fn get_note(
    client: &impl GenericClient,
    id: &Uuid,
    user_id: &Uuid,
) -> Result<Option<Note>, Error> {
    let row = client
        .query_opt("SELECT * FROM notes WHERE id = $1 AND (user_id = $2 OR EXISTS (SELECT 1 FROM shared_notes WHERE note_id = $1 AND shared_with = $2))",
                   &[id, user_id])
        .await?;
    match row {
        Some(row) => Ok(Some(Note::from_row(row)?)),
        None => Ok(None),
    }
}

pub async fn list_notes_by_user_id(
    client: &impl GenericClient,
    user_id: &Uuid,
) -> Result<Vec<Note>, Error> {
    let rows = client
        .query(
            "SELECT * FROM notes WHERE user_id = $1 OR id IN (SELECT note_id FROM shared_notes WHERE shared_with = $1)",
            &[user_id],
        )
        .await?;
    utils::from_rows(rows)
}

pub async fn search_notes(
    client: &impl GenericClient,
    user_id: &Uuid,
    q: String,
) -> Result<Vec<Note>, Error> {
    let rows = client
        .query(
            "SELECT n.*
            FROM notes n
            WHERE
                (n.user_id = $1 OR EXISTS (SELECT 1 FROM shared_notes s WHERE s.note_id = n.id AND s.shared_with = $1))
            AND
                to_tsvector('english', n.title || ' ' || n.content) @@ phraseto_tsquery('english', $2)
            ORDER BY
                ts_rank(to_tsvector('english', n.title || ' ' || n.content), phraseto_tsquery('english', $2)) DESC",
            &[user_id, &q])
        .await?;
    utils::from_rows(rows)
}

pub async fn update_note(client: &impl GenericClient, note: &mut Note) -> Result<Note, Error> {
    let row = client
        .query_one(
            "
        UPDATE notes SET
          title = $2,
          content = $3
        WHERE
           id = $1
        RETURNING *
    ",
            &[&note.id, &note.title, &note.content],
        )
        .await?;

    Ok(Note::from_row(row)?)
}

pub async fn delete_note(client: &impl GenericClient, id: Uuid) -> Result<bool, Error> {
    let result = client
        .execute(
            "
        DELETE FROM notes
        WHERE id = $1
    ",
            &[&id],
        )
        .await?;

    Ok(result > 0)
}

pub async fn share_note(
    client: &impl GenericClient,
    id: Uuid,
    target_user_id: Uuid,
) -> Result<(), Error> {
    client
        .execute(
            "
        INSERT INTO shared_notes (note_id, shared_with)
        VALUES ($1, $2)
        ON CONFLICT (note_id, shared_with) DO NOTHING",
            &[&id, &target_user_id],
        )
        .await?;

    Ok(())
}
