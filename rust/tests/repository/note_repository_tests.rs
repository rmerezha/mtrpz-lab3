use std::env;

use deadpool_postgres::Pool;
use speer_test::config::AppConfig;
use speer_test::db::init_db_pool;
use speer_test::model::db::user::User;
use speer_test::repository;
use tokio_postgres::GenericClient;

async fn get_pool() -> Pool {
    unsafe {
        env::set_var("APP_DB_HOST", "localhost");
        env::set_var("APP_DB_PORT", "15432");
    }

    let app_config = AppConfig::new();
    let cfg = app_config.config;

    init_db_pool(&cfg).await
}

#[tokio::test]
async fn test_create_note() {
    let pool = get_pool().await;
    let mut client = pool.get().await.unwrap();
    let transaction = client.transaction().await.unwrap();

    let user = create_user(transaction.client(), "user1").await.unwrap();
    let _note = repository::note_repository::create_note(transaction.client(), user.id, "title", "content").await.unwrap();

    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn test_search_note() {
    let pool = get_pool().await;
    let mut client = pool.get().await.unwrap();
    let transaction = client.transaction().await.unwrap();

    let user = create_user(transaction.client(), "user1").await.unwrap();
    let _note = repository::note_repository::create_note(transaction.client(), user.id, "title", "content").await.unwrap();
    let _note = repository::note_repository::create_note(transaction.client(), user.id, "New York", "City description").await.unwrap();
    let _note = repository::note_repository::create_note(transaction.client(), user.id, "US cities", "Washington, New York").await.unwrap();
    let notes = repository::note_repository::search_notes(transaction.client(), &user.id, "new york".to_string()).await.unwrap();

    assert_eq!(notes.len(), 2);
    transaction.rollback().await.unwrap();
}

async fn create_user(client: &impl GenericClient, username: &str) -> Result<User, anyhow::Error> {
    repository::user_repository::create_user(client, username, "").await
}
