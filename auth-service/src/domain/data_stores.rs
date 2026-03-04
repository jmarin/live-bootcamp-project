use crate::domain::{Email, Password, User};
use async_trait::async_trait;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    TokenDoesNotExist,
    UnexpectedError,
}

#[async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

#[async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}
