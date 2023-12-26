use unicode_segmentation::UnicodeSegmentation;

pub struct Subscriber {
    pub name: SubscriberName,
    pub email: String,
}
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_invalid_chars = s.chars().any(|g| forbidden_characters.contains(&g));

        println!("name: {}, isempty{}", s, is_empty);
        if is_empty || is_too_long || contains_invalid_chars {
            return Err(format!("{} is not a valid subscriber name", s));
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;

    #[test]
    fn a_empty_name_is_invalid() {
        let name = "".to_string();
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn a_name_that_only_cotains_whitespaces_is_invalid() {
        let name = "    ".to_string();
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        assert!(SubscriberName::parse(name).is_ok());
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_invalid() {
        let name = "a".repeat(257);
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn a_name_that_contains_special_characters_is_invalid() {
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let name = String::from("name");
        for invalid_ch in forbidden_characters {
            let modified_name = format!("{}{}", name, invalid_ch);
            assert!(SubscriberName::parse(modified_name).is_err());
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        assert!(SubscriberName::parse(String::from("Jean Banana")).is_ok());
    }
}
