use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ValidString(String);

impl ValidString {
    pub fn parse(s: String) -> Result<ValidString, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));
        if is_empty_or_whitespace || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use crate::domain::valid_string::ValidString;

    #[test]
    fn a_normal_string_is_ok() {
        let name = "I am a normal rust string! 😄".to_string();
        assert_ok!(ValidString::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(ValidString::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(ValidString::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(ValidString::parse(name));
        }
    }
}
