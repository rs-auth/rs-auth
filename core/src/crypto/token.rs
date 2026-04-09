use rand::{Rng, distributions::Alphanumeric};
use sha2::{Digest, Sha256};

pub fn generate_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn verify_token(raw: &str, stored_hash: &str) -> bool {
    hash_token(raw) == stored_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_has_correct_length() {
        let token = generate_token(32);
        assert_eq!(token.len(), 32, "token should have exactly 32 characters");
    }

    #[test]
    fn generate_token_is_alphanumeric() {
        let token = generate_token(32);
        assert!(
            token.chars().all(|c| c.is_alphanumeric()),
            "all characters should be alphanumeric"
        );
    }

    #[test]
    fn hash_token_is_deterministic() {
        let token = "test_token_123";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        assert_eq!(
            hash1, hash2,
            "hashing the same token should produce the same result"
        );
    }

    #[test]
    fn verify_token_roundtrip() {
        let token = generate_token(32);
        let hash = hash_token(&token);
        assert!(
            verify_token(&token, &hash),
            "verify_token should return true for correct token"
        );
    }

    #[test]
    fn verify_token_wrong_hash() {
        let token = generate_token(32);
        let wrong_hash = hash_token("different_token");
        assert!(
            !verify_token(&token, &wrong_hash),
            "verify_token should return false for wrong hash"
        );
    }
}
