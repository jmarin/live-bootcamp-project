use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, EmailClient, TwoFACodeStore, UserStore};

// Using a type alias to improve readability!
// By wrapping HashmapUserStore  in Tokio's RwLock smart pointer the user store can be safely mutated across threads,
// and by wrapping RwLock<HashmapUserStore> in an Arc smart pointer the underlying data can be shared across threads while maintaining a single source of truth!
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>;
pub type EmailClientType = Arc<dyn EmailClient + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        banned_token_store: BannedTokenStoreType,
        two_fa_code_store: TwoFACodeStoreType,
        email_client: EmailClientType,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }
}
