//! Desktop-only translation drafts and GitHub API submissions. No clone or HTTP server.
pub mod auth;
pub mod csv_text;
pub mod github;
use csv_text::{apply, hash, rows, Row};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;

pub const REPO: &str = "yhaggai/kitor";
pub const CSV_PATH: &str = "patches-workspace/drifter/he/payload/translation.csv";
pub const MANIFEST_PATH: &str = "patches-workspace/drifter/he/patch.json";

#[derive(Default)]
pub struct Contributions {
    pub gate: Mutex<()>,
    pub device: Mutex<Option<auth::Device>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Draft {
    schema: u32,
    pub base_sha: String,
    pub base_branch: String,
    source: String,
    pub edits: BTreeMap<String, String>,
    pub revision: u64,
}
#[derive(Serialize)]
pub struct EditorData {
    pub base_sha: String,
    pub base_branch: String,
    pub rows: Vec<Row>,
    pub edits: BTreeMap<String, String>,
    pub revision: u64,
}
#[derive(Serialize)]
pub struct Change {
    pub key: String,
    pub english: String,
    pub before: String,
    pub after: String,
}
#[derive(Serialize)]
pub struct Preview {
    pub digest: String,
    pub revision: u64,
    pub changes: Vec<Change>,
}
#[derive(Serialize)]
pub struct Submission {
    pub url: String,
    pub existing: bool,
}

fn path(root: &Path) -> PathBuf {
    root.join("translation-drafts/drifter-he.json")
}
fn read(root: &Path) -> Result<Draft, String> {
    let file = path(root);
    if std::fs::metadata(&file)
        .map_err(|_| "No saved draft")?
        .len()
        > 8_000_000
    {
        return Err("Draft exceeds size limit".into());
    }
    let data: Draft =
        serde_json::from_slice(&std::fs::read(file).map_err(|_| "Could not read draft")?)
            .map_err(|_| "Saved draft is damaged; it has not been overwritten")?;
    if data.schema != 1
        || data.base_sha.len() != 40
        || !data.base_sha.bytes().all(|b| b.is_ascii_hexdigit())
        || data.base_branch.is_empty()
    {
        return Err("Unsupported draft; it has not been overwritten".into());
    }
    apply(&data.source, &data.edits)?;
    Ok(data)
}
fn write(root: &Path, draft: &Draft) -> Result<(), String> {
    use std::io::Write;
    let file = path(root);
    let dir = file.parent().unwrap();
    std::fs::create_dir_all(dir).map_err(|_| "Could not create draft directory")?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(dir).map_err(|_| "Could not create draft file")?;
    temporary
        .write_all(&serde_json::to_vec(draft).map_err(|_| "Could not encode draft")?)
        .map_err(|_| "Could not write draft")?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|_| "Could not flush draft")?;
    temporary
        .persist(file)
        .map_err(|_| "Could not save draft atomically")?;
    Ok(())
}

pub async fn open(root: &Path) -> Result<EditorData, String> {
    let draft = if path(root).exists() {
        read(root)?
    } else {
        let repo = github::api(&format!("repos/{REPO}"), None, None).await?;
        let branch = github::string(&repo, "default_branch")?;
        let commit = github::api(
            &format!("repos/{REPO}/commits/{}", github::encode(&branch)),
            None,
            None,
        )
        .await?;
        let sha = github::string(&commit, "sha")?;
        let source = github::raw(CSV_PATH, &sha, None).await?;
        rows(&source)?;
        let draft = Draft {
            schema: 1,
            base_sha: sha,
            base_branch: branch,
            source,
            edits: BTreeMap::new(),
            revision: 0,
        };
        write(root, &draft)?;
        draft
    };
    Ok(EditorData {
        rows: rows(&draft.source)?,
        base_sha: draft.base_sha,
        base_branch: draft.base_branch,
        edits: draft.edits,
        revision: draft.revision,
    })
}

pub fn save(
    root: &Path,
    revision: u64,
    mut edits: BTreeMap<String, String>,
) -> Result<u64, String> {
    let mut draft = read(root)?;
    if draft.revision != revision {
        return Err("Draft changed in another window. Reopen the editor before saving.".into());
    }
    apply(&draft.source, &edits)?;
    let originals: BTreeMap<_, _> = rows(&draft.source)?
        .into_iter()
        .map(|r| (r.key, r.hebrew))
        .collect();
    edits.retain(|key, value| originals.get(key) != Some(value));
    draft.edits = edits;
    draft.revision += 1;
    write(root, &draft)?;
    Ok(draft.revision)
}

pub fn preview(root: &Path) -> Result<Preview, String> {
    let draft = read(root)?;
    let csv = apply(&draft.source, &draft.edits)?;
    let changes = rows(&draft.source)?
        .into_iter()
        .filter_map(|row| {
            draft
                .edits
                .get(&row.key)
                .filter(|value| **value != row.hebrew)
                .map(|value| Change {
                    key: row.key.clone(),
                    english: row.english,
                    before: row.hebrew,
                    after: value.clone(),
                })
        })
        .collect();
    Ok(Preview {
        digest: hash(csv.as_bytes()),
        revision: draft.revision,
        changes,
    })
}

pub async fn submit(root: &Path, revision: u64, digest: &str) -> Result<Submission, String> {
    let draft = read(root)?;
    if draft.revision != revision {
        return Err("The draft changed. Review your corrections again.".into());
    }
    let csv = apply(&draft.source, &draft.edits)?;
    if hash(csv.as_bytes()) != digest {
        return Err("The preview is outdated. Review again.".into());
    }
    if csv == draft.source {
        return Err("There are no corrections to submit".into());
    }
    let token = auth::token()
        .await?
        .ok_or("Sign in to GitHub before submitting")?;
    // Never trust a locally modified snapshot to overwrite upstream source silently.
    if github::raw(CSV_PATH, &draft.base_sha, Some(&token)).await? != draft.source {
        return Err(
            "Saved source differs from GitHub. Keep your draft and contact the maintainer.".into(),
        );
    }
    github::submit(&draft, csv, &token).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drafts_survive_restart_and_reject_stale_writes() {
        let root = tempfile::tempdir().unwrap();
        let source =
            include_str!("../../../../../patches-workspace/drifter/he/payload/translation.csv")
                .to_owned();
        let key = rows(&source).unwrap()[0].key.clone();
        write(
            root.path(),
            &Draft {
                schema: 1,
                base_sha: "a".repeat(40),
                base_branch: "v1".into(),
                source,
                edits: BTreeMap::new(),
                revision: 0,
            },
        )
        .unwrap();
        assert_eq!(
            save(
                root.path(),
                0,
                BTreeMap::from([(key.clone(), "תיקון".into())])
            )
            .unwrap(),
            1
        );
        assert_eq!(read(root.path()).unwrap().edits[&key], "תיקון");
        assert!(save(root.path(), 0, BTreeMap::new()).is_err());
        assert_eq!(preview(root.path()).unwrap().changes.len(), 1);
    }
}
