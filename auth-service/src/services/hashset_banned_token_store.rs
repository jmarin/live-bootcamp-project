use std::collections::HashSet;

use async_trait::async_trait;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default, PartialEq)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

impl HashsetBannedTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: HashSet::new(),
        }
    }
}

#[async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        if self.tokens.insert(token.to_string()) {
            Ok(())
        } else {
            Err(BannedTokenStoreError::UnexpectedError)
        }
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token() {
        let mut token_store = HashsetBannedTokenStore::new();
        let token = "token";
        let response = token_store.add_token(token).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_contains_token() {
        let mut token_store = HashsetBannedTokenStore::new();
        let token = "token";
        let response = token_store.add_token(token).await;
        assert!(response.is_ok());

        let response = token_store.contains_token(token).await;
        assert!(response.is_ok());
        assert_eq!(response, Ok(true));

        let response = token_store.contains_token("random_token").await;
        assert!(response.is_ok());
        assert_eq!(response, Ok(false));
    }
}
