use validator::ValidateEmail;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Email, String> {
        if s.validate_email() {
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
mod tests {

    use super::Email;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck::Gen;
    use quickcheck_macros::quickcheck;
    use rand::SeedableRng;

    #[test]
    fn test_email_parsing() {
        let valid_email = Email::parse(String::from("john@example.com")); //Email(String::from("john@example.com"));
        let invalid_email = Email::parse(String::from("johnexample.com"));

        assert!(valid_email.is_ok());
        assert!(invalid_email.is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let seed: u64 = g.size() as u64;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck]
    fn valid_emails_parse_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(valid_email.0).is_ok()
    }
}
