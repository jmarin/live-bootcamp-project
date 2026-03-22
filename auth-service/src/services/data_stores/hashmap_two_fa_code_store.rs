use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

impl HashmapTwoFACodeStore {
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes
            .remove(email)
            .map(|_| ())
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .cloned()
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore},
        services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore,
    };

    #[tokio::test]
    async fn test_add_and_get_code() {
        let email = Email::parse("john@example.com".to_owned()).expect("Email could not be parsed");
        let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string())
            .expect("Login Attempt Id could not be parsed");
        let code = TwoFACode::parse("012345".to_owned()).expect("Could not parse Two FA Code");
        let mut code_store = HashmapTwoFACodeStore::new();
        let _ = code_store
            .add_code(email.clone(), login_attempt_id, code.clone())
            .await;
        let result_code = code_store.get_code(&email).await;
        result_code
            .map(|c| assert_eq!(c.1, code))
            .expect("Test failed");
    }

    #[tokio::test]
    async fn test_remove_code() {
        let email = Email::parse("john@example.com".to_owned()).expect("Email could not be parsed");
        let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string())
            .expect("Login Attempt Id could not be parsed");
        let code = TwoFACode::parse("012345".to_owned()).expect("Could not parse Two FA Code");
        let mut code_store = HashmapTwoFACodeStore::new();
        let _ = code_store
            .add_code(email.clone(), login_attempt_id, code.clone())
            .await;
        let result_code = code_store.get_code(&email).await;
        result_code
            .map(|c| assert_eq!(c.1, code))
            .expect("Test failed");
        let _ = code_store.remove_code(&email).await;

        let result_code = code_store.get_code(&email).await;
        assert!(result_code.is_err());
    }
}
