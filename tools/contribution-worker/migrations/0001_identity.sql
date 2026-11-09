CREATE TABLE contributors (id TEXT PRIMARY KEY, google_sub TEXT NOT NULL UNIQUE, name TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE sessions (token_hash TEXT PRIMARY KEY, contributor_id TEXT NOT NULL REFERENCES contributors(id), expires_at INTEGER NOT NULL);
CREATE TABLE login_attempts (id TEXT PRIMARY KEY, state_hash TEXT NOT NULL UNIQUE, desktop_challenge TEXT NOT NULL, oauth_verifier TEXT NOT NULL, nonce TEXT NOT NULL, expires_at INTEGER NOT NULL, status TEXT NOT NULL DEFAULT 'pending', contributor_id TEXT REFERENCES contributors(id));
CREATE TABLE submissions (id TEXT PRIMARY KEY, contributor_id TEXT NOT NULL REFERENCES contributors(id), status TEXT NOT NULL, lease_until INTEGER NOT NULL, url TEXT, created_at INTEGER NOT NULL);
CREATE INDEX sessions_expiry ON sessions(expires_at);
CREATE INDEX submissions_contributor ON submissions(contributor_id, created_at);
CREATE TABLE rate_limits (bucket TEXT PRIMARY KEY, count INTEGER NOT NULL, expires_at INTEGER NOT NULL);
