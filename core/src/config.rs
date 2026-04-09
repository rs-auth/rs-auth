use time::Duration;

/// Top-level authentication configuration.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Secret used for cookie signing. Must be kept private.
    pub secret: String,
    /// Session time-to-live. Default: 30 days.
    pub session_ttl: Duration,
    /// Verification token time-to-live. Default: 1 hour.
    pub verification_ttl: Duration,
    /// Password reset token time-to-live. Default: 1 hour.
    pub reset_ttl: Duration,
    /// Length of generated tokens in bytes. Default: 32.
    pub token_length: usize,
    /// Email behavior configuration.
    pub email: EmailConfig,
    /// HTTP cookie configuration.
    pub cookie: CookieConfig,
    /// OAuth provider configuration.
    pub oauth: OAuthConfig,
}

/// Email behavior configuration.
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// Whether to send a verification email on signup. Default: true.
    pub send_verification_on_signup: bool,
    /// Whether to require email verification before allowing login. Default: false.
    pub require_verification_to_login: bool,
    /// Whether to automatically sign in the user after signup. Default: true.
    pub auto_sign_in_after_signup: bool,
    /// Whether to automatically sign in the user after email verification. Default: false.
    pub auto_sign_in_after_verification: bool,
}

/// HTTP cookie configuration.
#[derive(Debug, Clone)]
pub struct CookieConfig {
    /// Cookie name. Default: "rs_auth_session".
    pub name: String,
    /// Whether the cookie is HTTP-only. Default: true.
    pub http_only: bool,
    /// Whether the cookie requires HTTPS. Default: true.
    pub secure: bool,
    /// SameSite attribute. Default: Lax.
    pub same_site: SameSite,
    /// Cookie path. Default: "/".
    pub path: String,
    /// Cookie domain. Default: None.
    pub domain: Option<String>,
}

/// SameSite cookie attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    /// Strict SameSite policy.
    Strict,
    /// Lax SameSite policy.
    Lax,
    /// No SameSite policy.
    None,
}

/// OAuth provider configuration.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// List of configured OAuth providers.
    pub providers: Vec<OAuthProviderEntry>,
    /// Whether to allow implicit account linking when OAuth email matches existing user. Default: true.
    pub allow_implicit_account_linking: bool,
    /// URL to redirect to after successful OAuth login. Default: None.
    pub success_redirect: Option<String>,
    /// URL to redirect to after OAuth error. Default: None.
    pub error_redirect: Option<String>,
}

/// OAuth provider entry.
#[derive(Debug, Clone)]
pub struct OAuthProviderEntry {
    /// Provider identifier (e.g., "google", "github").
    pub provider_id: String,
    /// OAuth client ID.
    pub client_id: String,
    /// OAuth client secret. Must be kept private.
    pub client_secret: String,
    /// OAuth redirect URL.
    pub redirect_url: String,
    /// Override authorization URL, primarily useful for testing or self-hosted providers.
    pub auth_url: Option<String>,
    /// Override token URL, primarily useful for testing or self-hosted providers.
    pub token_url: Option<String>,
    /// Override userinfo URL, primarily useful for testing or self-hosted providers.
    pub userinfo_url: Option<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            providers: vec![],
            allow_implicit_account_linking: true,
            success_redirect: None,
            error_redirect: None,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            secret: String::new(),
            session_ttl: Duration::days(30),
            verification_ttl: Duration::hours(1),
            reset_ttl: Duration::hours(1),
            token_length: 32,
            email: EmailConfig::default(),
            cookie: CookieConfig::default(),
            oauth: OAuthConfig::default(),
        }
    }
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            send_verification_on_signup: true,
            require_verification_to_login: false,
            auto_sign_in_after_signup: true,
            auto_sign_in_after_verification: false,
        }
    }
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            name: "rs_auth_session".to_string(),
            http_only: true,
            secure: true,
            same_site: SameSite::Lax,
            path: "/".to_string(),
            domain: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_values() {
        let config = AuthConfig::default();

        assert_eq!(
            config.session_ttl,
            Duration::days(30),
            "session_ttl should be 30 days"
        );
        assert_eq!(config.token_length, 32, "token_length should be 32");
        assert_eq!(
            config.cookie.name, "rs_auth_session",
            "cookie name should be 'rs_auth_session'"
        );
        assert_eq!(
            config.verification_ttl,
            Duration::hours(1),
            "verification_ttl should be 1 hour"
        );
        assert_eq!(
            config.reset_ttl,
            Duration::hours(1),
            "reset_ttl should be 1 hour"
        );
        assert!(config.cookie.http_only, "cookie should be http_only");
        assert!(config.cookie.secure, "cookie should be secure");
        assert_eq!(
            config.cookie.same_site,
            SameSite::Lax,
            "cookie same_site should be Lax"
        );
        assert_eq!(config.cookie.path, "/", "cookie path should be '/'");
        assert_eq!(config.cookie.domain, None, "cookie domain should be None");
    }
}
