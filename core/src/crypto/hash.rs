use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::error::AuthError;

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AuthError::Hash(error.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(hash).map_err(|error| AuthError::Hash(error.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let password = "my_secure_password";
        let hash = hash_password(password).expect("hashing should succeed");
        let result = verify_password(password, &hash).expect("verification should succeed");
        assert!(result, "correct password should verify successfully");
    }

    #[test]
    fn wrong_password_returns_false() {
        let password = "my_secure_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).expect("hashing should succeed");
        let result = verify_password(wrong_password, &hash).expect("verification should succeed");
        assert!(!result, "wrong password should not verify");
    }
}
