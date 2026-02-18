use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
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

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        self.users
            .get(email)
            .cloned()
            .ok_or(UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        if let Some(user) = self.users.get(email) {
            if user.email == email && user.password == password {
                return Ok(());
            } else {
                return Err(UserStoreError::InvalidCredentials);
            }
        }
        Err(UserStoreError::UserNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let users = HashMap::<String, User>::new();
        let mut user_store = HashmapUserStore { users };
        let user = User::new(
            "john@example.com".to_string(),
            "password123".to_string(),
            false,
        );
        user_store.add_user(user).unwrap();
        assert!(user_store.users.len() == 1);
    }

    #[tokio::test]
    async fn test_get_user() {
        let users = HashMap::<String, User>::new();
        let mut user_store = HashmapUserStore { users };
        let user = User::new(
            "john@example.com".to_string(),
            "password123".to_string(),
            false,
        );
        user_store.add_user(user.clone()).unwrap();
        let retrieved_user = user_store.get_user(&"john@example.com");
        assert_eq!(retrieved_user.unwrap(), user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let valid_user = User::new(
            "john@example.com".to_string(),
            "password123".to_string(),
            false,
        );

        let invalid_password = "wrongpassword";

        let users = HashMap::<String, User>::new();
        let mut user_store = HashmapUserStore { users };
        user_store.add_user(valid_user.clone()).unwrap();
        assert!(user_store
            .validate_user(&valid_user.email, &valid_user.password)
            .is_ok());

        assert_eq!(
            user_store.validate_user(&valid_user.email, invalid_password),
            Err(UserStoreError::InvalidCredentials)
        );
    }
}
