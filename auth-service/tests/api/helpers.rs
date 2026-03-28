use std::{str::FromStr, sync::Arc};

use auth_service::{Application, app_state::{AppState, BannedTokenStoreType, TwoFACodeStoreType}, get_postgres_pool, get_redis_client, services::{data_stores::{HashsetBannedTokenStore, PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore, hashmap_two_fa_code_store::HashmapTwoFACodeStore}, mock_email_client::MockEmailClient}, utils::constants::{DATABASE_URL, DEFAULT_REDIS_HOSTNAME, test::APP_ADDRESS}};
use reqwest::cookie::Jar;
use sqlx::{Connection, Executor, PgConnection, PgPool, postgres::{PgConnectOptions, PgPoolOptions}};
use tokio::sync::RwLock;
use uuid::Uuid;

//TestApp is test helper that is responsible for configuring/launching the auth service and providing methods for sending HTTP requests to the auth service.
pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub db_name: String,
    pub clean_up_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (pg_pool, db_name) = configure_postgresql().await;
         
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(Arc::new(RwLock::new(configure_redis())))));
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(Arc::new(RwLock::new(configure_redis())))));
        let email_client = Arc::new(MockEmailClient::default());
        let app_state = AppState { user_store, banned_token_store: banned_token_store.clone(), two_fa_code_store: two_fa_code_store.clone(), email_client };

        let app = Application::build(app_state, APP_ADDRESS)
            .await
            .expect("Failed to build application");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default()); 

        let http_client = 
         // Create a Reqwest http client instance
            reqwest::Client::builder()
                .cookie_provider(cookie_jar.clone())
                .build()
                .expect("Failed to build HTTP client");


        // Create new 'TestApp' instance and return it
        Self {address, cookie_jar, http_client, banned_token_store, two_fa_code_store, db_name, clean_up_called: false}
    }



    pub async fn clean_up(&mut self) {
        if self.clean_up_called {
            return;
        }

        delete_database(&self.db_name).await;
        self.clean_up_called = true;
    }

    

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }
 

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response 
    where 
      Body: serde::Serialize
     {
        self.http_client
            .post(format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
     } 

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
     where Body: serde::Serialize
     {
        self.http_client
          .post(format!("{}/login", &self.address))
          .json(body)
          .send()
          .await
          .expect("Failed to execute request")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response where Body: serde::Serialize {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response where Body: serde::Serialize {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn get_random_email() -> String {
        format!("{}@example.com", Uuid::new_v4())
    } 
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("TestApp::clean_up hasn't been called before dropping TestApp")
        }
    }
}

async fn configure_postgresql() -> (PgPool, String) {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    let pool = get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!");

    (pool, db_name)
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Migrate database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

fn configure_redis() -> redis::Connection {
    let redis_hostname = DEFAULT_REDIS_HOSTNAME.to_owned();

    get_redis_client(redis_hostname)
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}