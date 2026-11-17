//! Desktop-only translation drafts and GitHub API submissions. No clone or HTTP server.

pub mod csv_text;
pub mod github;
pub mod identity;
use csv_text::{apply, hash, rows, Catalog, Row};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;

pub const REPO: &str = "yhaggai/kitor";
pub const MAX_CATALOG_BYTES: usize = 24_000_000;

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
    pub context: String,
    pub variant: Option<&'static str>,
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

fn path(root: &Path, config: &Catalog) -> PathBuf {
    root.join("translation-drafts")
        .join(format!("{}-he.json", config.id))
}
fn read(root: &Path, config: &Catalog) -> Result<Draft, String> {
    let file = path(root, config);
    if std::fs::metadata(&file)
        .map_err(|_| "No saved draft")?
        .len()
        > 32_000_000
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
    if data.source.len() > MAX_CATALOG_BYTES {
        return Err("Translation catalog exceeds size limit".into());
    }
    Ok(data)
}
fn write(root: &Path, config: &Catalog, draft: &Draft) -> Result<(), String> {
    use std::io::Write;
    let file = path(root, config);
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

pub async fn open(root: &Path, config: &Catalog) -> Result<EditorData, String> {
    let draft = if path(root, config).exists() {
        read(root, config)?
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
        let source = github::raw(&config.path, &sha, None).await?;
        rows(config, &source)?;
        let draft = Draft {
            schema: 1,
            base_sha: sha,
            base_branch: branch,
            source,
            edits: BTreeMap::new(),
            revision: 0,
        };
        write(root, config, &draft)?;
        draft
    };
    Ok(EditorData {
        rows: rows(config, &draft.source)?,
        base_sha: draft.base_sha,
        base_branch: draft.base_branch,
        edits: draft.edits,
        revision: draft.revision,
    })
}

pub fn save(
    root: &Path,
    config: &Catalog,
    revision: u64,
    mut edits: BTreeMap<String, String>,
) -> Result<u64, String> {
    let mut draft = read(root, config)?;
    if draft.revision != revision {
        return Err("Draft changed in another window. Reopen the editor before saving.".into());
    }
    apply(config, &draft.source, &edits, false)?;
    let originals: BTreeMap<_, _> = rows(config, &draft.source)?
        .into_iter()
        .map(|r| (r.key, r.hebrew))
        .collect();
    edits.retain(|key, value| originals.get(key) != Some(value));
    draft.edits = edits;
    draft.revision += 1;
    write(root, config, &draft)?;
    Ok(draft.revision)
}

pub fn preview(root: &Path, config: &Catalog) -> Result<Preview, String> {
    let draft = read(root, config)?;
    let csv = apply(config, &draft.source, &draft.edits, true)?;
    let changes = rows(config, &draft.source)?
        .into_iter()
        .filter_map(|row| {
            draft
                .edits
                .get(&row.key)
                .filter(|value| **value != row.hebrew)
                .map(|value| Change {
                    key: row.key.clone(),
                    english: row.english,
                    context: row.context,
                    variant: row.variant,
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
    config: &Catalog,
    revision: u64,
    digest: &str,
    token: &str,
    email_opt_in: bool,
) -> Result<Submission, String> {
    let draft = read(root, config)?;
    if draft.revision != revision {
        return Err("The draft changed. Review your corrections again.".into());
    }
    let csv = apply(config, &draft.source, &draft.edits, true)?;
    if hash(csv.as_bytes()) != digest {
        return Err("The preview is outdated. Review again.".into());
    }
    if csv == draft.source {
        return Err("There are no corrections to submit".into());
    }
    let input = serde_json::json!({
        "catalog_id": config.id,
        "base_sha": draft.base_sha,
        "digest": digest,
        "edits": draft.edits,
        "email_opt_in": email_opt_in,
    });
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not initialize bot connection")?
        .post(format!("{}/submissions", identity::endpoint()?))
        .bearer_auth(token)
        .header("X-Kitor-Client", "desktop-poc")
        .json(&input)
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
        let config = csv_text::catalog("drifter").unwrap();
        let key = rows(config, &source).unwrap()[0].key.clone();
        write(
            root.path(),
            config,
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
                config,
                0,
                BTreeMap::from([(key.clone(), "תיקון".into())])
            )
            .unwrap(),
            1
        );
        assert_eq!(read(root.path(), config).unwrap().edits[&key], "תיקון");
        assert!(save(root.path(), config, 0, BTreeMap::new()).is_err());
        assert_eq!(preview(root.path(), config).unwrap().changes.len(), 1);
    }
}

#[derive(Serialize, Deserialize)]
pub struct BotStatus {
    pub configured: bool,
    #[serde(default)]
    pub catalogs: Vec<String>,
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
