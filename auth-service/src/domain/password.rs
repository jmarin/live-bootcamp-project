#[derive(Debug, PartialEq, Clone)]
pub struct Password(String);

impl Password {
    pub fn parse(s: String) -> Result<Password, String> {
        if s.len() >= 8 {
            Ok(Password(s))
        } else {
            Err("Password must be at least 8 characters long".to_string())
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[cfg(test)]

mod tests {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn test_valid_password_preserves_input(s: String) -> bool {
        // If a password is valid, it should preserve the original string
        if s.len() >= 8 {
            if let Ok(password) = Password::parse(s.clone()) {
                password.as_ref() == s.as_str()
            } else {
                false
            }
        } else {
            true // Skip invalid passwords
        }
    }

    #[quickcheck]
    fn test_password_parsing_is_idempotent(s: String) -> bool {
        // Parsing a valid password string twice should give the same result
        let first = Password::parse(s.clone());
        let second = Password::parse(s);
        first == second
    }

    #[quickcheck]
    fn test_password_clone_equals_original(s: String) -> bool {
        // A cloned password should equal the original
        if s.len() >= 8 {
            if let Ok(password) = Password::parse(s) {
                let cloned = password.clone();
                password == cloned
            } else {
                false
            }
        } else {
            true
        }
    }

    #[quickcheck]
    fn test_password_min_length_boundary(_extra: String) -> bool {
        // Test that 8 characters is valid, 7 is not
        let mut base = "1234567".to_string();
        let invalid = Password::parse(base.clone());

        base.push('8');
        let valid = Password::parse(base);

        invalid.is_err() && valid.is_ok()
    }

    #[quickcheck]
    fn test_password_error_message_consistency(s: String) -> bool {
        // All invalid passwords should return the same error message
        if s.len() < 8 {
            if let Err(msg) = Password::parse(s) {
                msg == "Password must be at least 8 characters long"
            } else {
                false
            }
        } else {
            true
        }
    }

    #[quickcheck]
    fn test_password_as_ref_gives_str(s: String) -> bool {
        // as_ref should return a string slice with the same content
        if s.len() >= 8 {
            if let Ok(password) = Password::parse(s.clone()) {
                let as_ref: &str = password.as_ref();
                as_ref == s.as_str() && as_ref.len() == s.len()
            } else {
                false
            }
        } else {
            true
        }
    }

    #[quickcheck]
    fn test_password_accepts_unicode(c: char) -> bool {
        // Passwords should accept unicode characters
        let mut password_str = "password".to_string();
        password_str.push(c);

        if password_str.len() >= 8 {
            Password::parse(password_str).is_ok()
        } else {
            true
        }
    }
}
