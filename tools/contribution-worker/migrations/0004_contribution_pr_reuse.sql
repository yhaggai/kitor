CREATE TABLE contribution_leases (
  contributor_id TEXT PRIMARY KEY REFERENCES contributors(id),
  owner TEXT NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE INDEX submissions_pr_url ON submissions(url);
