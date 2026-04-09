use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::{Cookie, Key, SameSite as AxumSameSite};
use rs_auth_core::config::{CookieConfig, SameSite};
use time::Duration;

/// Extract the session token from a signed cookie jar.
pub fn get_session_token(jar: &SignedCookieJar, config: &CookieConfig) -> Option<String> {
    jar.get(&config.name)
        .map(|cookie| cookie.value().to_string())
}

/// Add a session cookie to the jar with the given token and max age.
pub fn set_session_cookie(
    jar: SignedCookieJar,
    token: &str,
    config: &CookieConfig,
    max_age: Duration,
) -> SignedCookieJar {
    let same_site = match config.same_site {
        SameSite::Strict => AxumSameSite::Strict,
        SameSite::Lax => AxumSameSite::Lax,
        SameSite::None => AxumSameSite::None,
    };

    let mut cookie = Cookie::new(config.name.clone(), token.to_string());
    cookie.set_http_only(config.http_only);
    cookie.set_secure(config.secure);
    cookie.set_same_site(same_site);
    cookie.set_path(config.path.clone());
    cookie.set_max_age(max_age);

    if let Some(domain) = &config.domain {
        cookie.set_domain(domain.clone());
    }

    jar.add(cookie)
}

/// Remove the session cookie from the jar.
pub fn remove_session_cookie(jar: SignedCookieJar, config: &CookieConfig) -> SignedCookieJar {
    let mut cookie = Cookie::new(config.name.clone(), String::new());
    cookie.set_path(config.path.clone());
    if let Some(domain) = &config.domain {
        cookie.set_domain(domain.clone());
    }
    jar.remove(cookie)
}

/// Create a cookie signing key from a secret string.
pub fn make_cookie_key(secret: &str) -> Key {
    let mut bytes = [0u8; 64];
    let secret_bytes = secret.as_bytes();
    let len = secret_bytes.len().min(64);
    bytes[..len].copy_from_slice(&secret_bytes[..len]);
    Key::from(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    #[test]
    fn set_and_get_session_cookie_roundtrip() {
        let key =
            make_cookie_key("test_secret_key_that_is_at_least_64_bytes_long_for_cookie_signing");
        let jar = SignedCookieJar::new(key);
        let config = CookieConfig::default();
        let token = "test_session_token_12345";
        let max_age = Duration::days(30);

        let jar = set_session_cookie(jar, token, &config, max_age);
        let retrieved_token = get_session_token(&jar, &config);

        assert_eq!(
            retrieved_token,
            Some(token.to_string()),
            "retrieved token should match the set token"
        );
    }
}
