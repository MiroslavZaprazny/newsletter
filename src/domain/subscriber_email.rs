use validator::validate_email;

#[derive(Debug)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Self, String> {
        if !validate_email(&s) {
            return Err(format!("{} is not a valid email", s));
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Email;

    #[test]
    fn test_valid_email() {
        let email = String::from("testingg@gmail.com");
        assert!(Email::parse(email).is_ok());
    }

    #[test]
    fn test_empty_string_is_invalid() {
        let email = String::from(" ");
        assert!(Email::parse(email).is_err());
    }

    #[test]
    fn test_invalid_email_is_rejected() {
        let email = String::from("gmail.rs");
        assert!(Email::parse(email).is_err());
    }
}
