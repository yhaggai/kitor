ALTER TABLE submissions ADD COLUMN catalog_id TEXT NOT NULL DEFAULT 'drifter';
CREATE INDEX submissions_contributor_catalog ON submissions(contributor_id, catalog_id, status);
