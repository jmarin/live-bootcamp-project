use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::UserStore;

// Using a type alias to improve readability!
// By wrapping HashmapUserStore  in Tokio's RwLock smart pointer the user store can be safely mutated across threads,
// and by wrapping RwLock<HashmapUserStore> in an Arc smart pointer the underlying data can be shared across threads while maintaining a single source of truth!
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}
