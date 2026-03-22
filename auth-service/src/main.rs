use std::sync::Arc;

use auth_service::app_state::AppState;

use auth_service::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use auth_service::services::data_stores::{
    HashmapUserStore, HashsetBannedTokenStore, PostgresUserStore,
};
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::utils::constants::prod::APP_ADDRESS;
use auth_service::utils::constants::DATABASE_URL;
use auth_service::{get_postgres_pool, Application};

use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::new()));
    let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::new()));
    let email_client = Arc::new(MockEmailClient::default());
    let app_state = AppState {
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    };

    let app = Application::build(app_state, APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}
