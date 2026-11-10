ALTER TABLE contributors ADD COLUMN email TEXT;
ALTER TABLE submissions ADD COLUMN edits_json TEXT;
ALTER TABLE submissions ADD COLUMN merge_sha TEXT;
ALTER TABLE submissions ADD COLUMN release_checked_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE submissions ADD COLUMN released_tag TEXT;
CREATE TABLE patch_releases(tag TEXT PRIMARY KEY, created_at INTEGER NOT NULL);
CREATE TABLE notifications(
 id TEXT PRIMARY KEY, submission_id TEXT NOT NULL REFERENCES submissions(id),
 kind TEXT NOT NULL, subject TEXT NOT NULL, message TEXT NOT NULL,
 status TEXT NOT NULL DEFAULT 'pending', recipient TEXT, sender TEXT,
 first_attempt INTEGER, next_attempt INTEGER NOT NULL DEFAULT 0,
 lease_until INTEGER NOT NULL DEFAULT 0, attempts INTEGER NOT NULL DEFAULT 0,
 provider_id TEXT, sent_at INTEGER
);
CREATE INDEX notifications_pending ON notifications(status,next_attempt,lease_until);
CREATE TABLE notification_budget(day TEXT PRIMARY KEY, count INTEGER NOT NULL);
CREATE TABLE release_checks(submission_id TEXT NOT NULL, tag TEXT NOT NULL, checked_at INTEGER NOT NULL, PRIMARY KEY(submission_id,tag));
