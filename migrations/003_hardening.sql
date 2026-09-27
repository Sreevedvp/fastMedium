-- Stable edit versions and deterministic feed access paths.
ALTER TABLE posts ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
CREATE INDEX IF NOT EXISTS idx_posts_feed_order ON posts(status, published_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_posts_author_order ON posts(author_id, updated_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_expiry ON sessions(expires_at);
