//! Desktop-only translation drafts and GitHub API submissions. No clone or HTTP server.

pub mod csv_text;
pub mod github;
pub mod identity;
use csv_text::{apply, hash, rows, Row};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;

pub const REPO: &str = "yhaggai/kitor";
pub const CSV_PATH: &str = "patches-workspace/drifter/he/payload/translation.csv";

#[derive(Default)]
pub struct Contributions {
    pub gate: Mutex<()>,
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
#[derive(Serialize, Deserialize)]
pub struct Submission {
    pub url: String,
    pub existing: bool,
    #[serde(default)]
    pub updated: bool,
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

pub async fn submit(
    root: &Path,
    revision: u64,
    digest: &str,
    token: &str,
    email_opt_in: bool,
) -> Result<Submission, String> {
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
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not initialize bot connection")?
        .post(format!("{}/submissions", identity::endpoint()?))
        .bearer_auth(token)
        .header("X-Kitor-Client", "desktop-poc")
        .json(&serde_json::json!({"base_sha":draft.base_sha,"digest":digest,"edits":draft.edits,"email_opt_in":email_opt_in}))
        .send()
        .await
        .map_err(|_| {
            "The local contribution bot is unavailable. Start it and retry; your draft is saved."
        })?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|_| "Could not read the bot response")?;
    if !status.is_success() {
        let value: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
        return Err(value["error"]
            .as_str()
            .unwrap_or("The bot could not submit your corrections")
            .to_owned());
    }
    let result: Submission = serde_json::from_str(&text).map_err(|_| "Invalid bot response")?;
    let suffix = result
        .url
        .strip_prefix("https://github.com/yhaggai/kitor/pull/")
        .ok_or("Unexpected PR link from bot")?;
    if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Invalid PR link from bot".into());
    }
    Ok(result)
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

#[derive(Serialize, Deserialize)]
pub struct BotStatus {
    pub configured: bool,
}
pub async fn bot_status() -> Result<BotStatus, String> {
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not initialize bot connection")?
        .get(format!("{}/health", identity::endpoint()?))
        .send()
        .await
        .map_err(|_| {
            "Start the local contribution bot to submit. Your draft stays saved on this computer."
        })?;
    response
        .json::<BotStatus>()
        .await
        .map_err(|_| "Invalid local bot response".into())
}
