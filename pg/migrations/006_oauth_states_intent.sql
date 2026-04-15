ALTER TABLE oauth_states ADD COLUMN intent TEXT NOT NULL DEFAULT 'login';
ALTER TABLE oauth_states ADD COLUMN link_user_id BIGINT;

CREATE INDEX idx_oauth_states_link_user_id ON oauth_states (link_user_id) WHERE link_user_id IS NOT NULL;
