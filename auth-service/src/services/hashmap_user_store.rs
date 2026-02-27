use std::collections::HashMap;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};
use async_trait::async_trait;

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

impl HashmapUserStore {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        self.users
            .iter()
            .find(|(_, existing_user)| {
                //existing_user.email == user.email && existing_user.password == user.password
                *existing_user == &user
            })
            .map(|_| Err(UserStoreError::UserAlreadyExists))
            .unwrap_or_else(|| {
                self.users.insert(user.email.clone(), user);
                Ok(())
            })
    }

    pub fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        self.users
            .get(email)
            .cloned()
            .ok_or(UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                if user.password.eq(password) {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

#[async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        self.add_user(user)
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        self.get_user(email)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        self.validate_user(email, password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let users = HashMap::<Email, User>::new();
        let mut user_store = HashmapUserStore { users };
        let user = User::new(
            Email::parse(String::from("john@example.com")).unwrap(),
            Password::parse("password123".to_string()).unwrap(),
            false,
        );
        user_store.add_user(user).unwrap();
        assert!(user_store.users.len() == 1);
    }

    #[tokio::test]
    async fn test_get_user() {
        let users = HashMap::<Email, User>::new();
        let mut user_store = HashmapUserStore { users };
        let user = User::new(
            Email::parse(String::from("john@example.com")).unwrap(),
            Password::parse("password123".to_string()).unwrap(),
            false,
        );
        user_store.add_user(user.clone()).unwrap();
        let retrieved_user =
            user_store.get_user(&Email::parse(String::from("john@example.com")).unwrap());
        assert_eq!(retrieved_user.unwrap(), user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let valid_user = User::new(
            Email::parse(String::from("john@example.com")).unwrap(),
            Password::parse("password123".to_string()).unwrap(),
            false,
        );

        let invalid_password = Password::parse("wrongpassword".to_string()).unwrap();

        let users = HashMap::<Email, User>::new();
        let mut user_store = HashmapUserStore { users };
        user_store.add_user(valid_user.clone()).unwrap();
        assert!(user_store
            .validate_user(&valid_user.email, &valid_user.password)
            .is_ok());

        assert_eq!(
            user_store.validate_user(&valid_user.email, &invalid_password),
            Err(UserStoreError::InvalidCredentials)
        );
    }
}
