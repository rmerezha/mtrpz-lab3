use chrono::{DateTime, Utc};
use tokio_pg_mapper::Error;
use tokio_postgres::GenericClient;
use tokio_pg_mapper::FromTokioPostgresRow;
use uuid::Uuid;
use crate::model::db::user_token::UserToken;
use crate::model::types::user_token_type::UserTokenType;

pub async fn create(
    client:  &impl GenericClient,
    created_at: DateTime<Utc>,
    user_id: Uuid,
    token: String,
    token_type: UserTokenType,
    expires_at: DateTime<Utc>) -> Result<UserToken, Error> {
    let row = client.query_one(
        "INSERT INTO user_token (created_at, user_id, token, token_type, expires_at) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        &[&created_at, &user_id, &token, &token_type, &expires_at]).await?;

    UserToken::from_row(row)
}

pub async fn delete_by_token(client:  &impl GenericClient, token: String) -> Result<(), Error> {
    let res = client.execute(
        "DELETE FROM user_token WHERE token = $1",
        &[&token]).await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::from(e)),
    }
}

pub async fn invalidate(client:  &impl GenericClient, id: Uuid) -> Result<(), Error> {
    let res = client.execute(
        "UPDATE user_token SET status = 'INACTIVE' WHERE id = $1",
        &[&id]).await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::from(e)),
    }
}

pub async fn invalidate_by_user_id(client:  &impl GenericClient, user_id: Uuid) -> Result<(), Error> {
    let res = client.execute(
        "UPDATE user_token SET status = 'INACTIVE' WHERE user_id = $1 AND expires_at >= now() AND status = 'ACTIVE'",
        &[&user_id]).await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::from(e)),
    }
}

pub async fn get_by_token<'a>(client: &impl GenericClient, token: &str) -> Result<Option<UserToken>, Error> {
    let row = client
        .query_opt("SELECT * FROM user_token WHERE token = $1", &[&token])
        .await?;

    if let Some(row) = row {
        Ok(Some(UserToken::from_row(row)?))
    }  else {
        Ok(None)
    }
}
