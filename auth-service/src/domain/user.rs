use crate::domain::{Email, HashedPassword};

// The User struct should contain 3 fields: email (String), password (String) and requires_2fa (boolean)
#[derive(Debug, PartialEq, Clone, sqlx::FromRow)]
pub struct User {
    pub email: Email,
    pub password: HashedPassword,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: Email, password: HashedPassword, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
        }
    }
}
