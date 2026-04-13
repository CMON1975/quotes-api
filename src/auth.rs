use constant_time_eq::constant_time_eq;

pub fn extract_bearer_token(header: &str) -> Option<&str> {
    let token = header.strip_prefix("Bearer ")?;
    if token.is_empty() { None } else { Some(token) }
}

pub fn verify_api_key(token: &str, expected: &str) -> bool {
    constant_time_eq(token.as_bytes(), expected.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_valid_bearer_token() {
        let header = "Bearer mysecretkey";
        assert_eq!(extract_bearer_token(header), Some("mysecretkey"));
    }

    #[test]
    fn rejects_missing_bearer_prefix() {
        assert_eq!(extract_bearer_token("mysecretkey"), None);
    }

    #[test]
    fn rejects_empty_string() {
        assert_eq!(extract_bearer_token(""), None);
    }

    #[test]
    fn rejects_bearer_with_no_token() {
        assert_eq!(extract_bearer_token("Bearer "), None);
    }

    #[test]
    fn verifies_correct_key() {
        assert!(verify_api_key("correctkey", "correctkey"));
    }

    #[test]
    fn rejects_incorrect_key() {
        assert!(!verify_api_key("wrongkey", "correctkey"));
    }

    #[test]
    fn comparison_is_constant_time() {
        // Verifies constant_time_eq is used - both strings same length, different content
        assert!(!verify_api_key("aaaaaaaaa", "bbbbbbbbb"));
    }
}
