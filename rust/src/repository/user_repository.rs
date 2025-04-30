use anyhow::Error;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::{Client, GenericClient};
use uuid::Uuid;
use crate::model::db::user::User;

pub async fn create_user(client: &impl GenericClient, username: &str, password_hash: &str) -> Result<User, Error> {
    let row = client.query_one(
        "INSERT INTO users (username, password_hash)
            VALUES ($1, $2)
            RETURNING *",
        &[&username, &password_hash]).await?;

    Ok(User::from_row(row)?)
}


pub async fn find_user_by_username(client: &impl GenericClient, username: &str) -> Result<Option<User>, Error> {
    let row = client.query_opt("SELECT * FROM users WHERE username = $1", &[&username]).await?;

    match row {
        Some(row) => Ok(Some(User::from_row(row)?)),
        _ => Ok(None),
    }
}

pub async fn find_user_by_id(client: &Client, id: &Uuid) -> Result<Option<User>, Error> {
    let row = client.query_opt("SELECT * FROM users WHERE id = $1", &[&id]).await?;

    match row {
        Some(row) => Ok(Some(User::from_row(row)?)),
        _ => Ok(None),
    }
}
