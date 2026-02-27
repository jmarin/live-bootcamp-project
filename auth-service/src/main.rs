use std::sync::Arc;

use auth_service::app_state::AppState;

use auth_service::utils::constants::prod::APP_ADDRESS;
use auth_service::{services::hashmap_user_store::HashmapUserStore, Application};

use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = Arc::new(RwLock::new(HashmapUserStore::new()));
    let app_state = AppState { user_store };

    let app = Application::build(app_state, APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
