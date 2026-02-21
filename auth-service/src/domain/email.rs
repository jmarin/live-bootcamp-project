#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Email, String> {
        if s.contains("@") {
            Ok(Email(s))
        } else {
            Err("Invalid email format".to_string())
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[cfg(test)]
#[test]
fn test_email_parsing() {
    let valid_email = Email::parse(String::from("john@example.com")); //Email(String::from("john@example.com"));
    let invalid_email = Email::parse(String::from("johnexample.com"));

    assert!(valid_email.is_ok());
    assert!(invalid_email.is_err());
}
