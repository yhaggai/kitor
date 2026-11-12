ALTER TABLE submissions ADD COLUMN email_opt_in INTEGER NOT NULL DEFAULT 0;
ALTER TABLE submissions ADD COLUMN email_consent_at INTEGER;
ALTER TABLE contributors ADD COLUMN email_unsubscribed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE contributors ADD COLUMN unsubscribe_token TEXT;
CREATE UNIQUE INDEX contributor_unsubscribe_token ON contributors(unsubscribe_token);
