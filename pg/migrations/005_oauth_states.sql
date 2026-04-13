CREATE TABLE oauth_states (
    id BIGSERIAL PRIMARY KEY,
    provider_id TEXT NOT NULL,
    csrf_state TEXT NOT NULL,
    pkce_verifier TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_oauth_states_csrf_state ON oauth_states (csrf_state);
CREATE INDEX idx_oauth_states_expires_at ON oauth_states (expires_at);
