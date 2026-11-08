use super::{csv_text::hash, Draft, Submission, CSV_PATH, MANIFEST_PATH, REPO};
use reqwest::{Client, Method};
use serde_json::{json, Value};
use std::time::Duration;

pub fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Kitor-translation-contributions")
        .timeout(Duration::from_secs(40))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not initialize GitHub connection".into())
}
pub fn encode(value: &str) -> String {
    percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC).to_string()
}
pub fn string(value: &Value, field: &str) -> Result<String, String> {
    value[field]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("GitHub response omitted {field}"))
}
pub async fn bounded(mut response: reqwest::Response) -> Result<String, String> {
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "GitHub returned HTTP {}. Check sign-in, permissions, or rate limits and retry.",
            status.as_u16()
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "GitHub response was interrupted")?
    {
        if bytes.len() + chunk.len() > 8_000_000 {
            return Err("GitHub response exceeds size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| "GitHub response was not UTF-8".into())
}
pub async fn api(path: &str, body: Option<Value>, token: Option<&str>) -> Result<Value, String> {
    let mut request = client()?
        .request(
            if body.is_some() {
                Method::POST
            } else {
                Method::GET
            },
            format!("https://api.github.com/{path}"),
        )
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let text = bounded(request.send().await.map_err(|_| "Could not reach GitHub")?).await?;
    serde_json::from_str(&text).map_err(|_| "Invalid GitHub response".into())
}
pub async fn raw(path: &str, sha: &str, token: Option<&str>) -> Result<String, String> {
    let mut request = client()?
        .get(format!(
            "https://api.github.com/repos/{REPO}/contents/{path}?ref={}",
            encode(sha)
        ))
        .header("Accept", "application/vnd.github.raw+json");
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    bounded(
        request
            .send()
            .await
            .map_err(|_| "Could not load translation source from GitHub")?,
    )
    .await
}

pub async fn submit(draft: &Draft, csv: String, token: &str) -> Result<Submission, String> {
    let token = Some(token);
    let user = api("user", None, token).await?;
    let login = string(&user, "login")?;
    if !login
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || c == b'-')
    {
        return Err("Invalid GitHub login".into());
    }
    let setup = repository_setup(token.unwrap()).await?;
    let target = setup.repository.ok_or(setup.message)?;
    let owner = target
        .split('/')
        .next()
        .ok_or("Invalid target repository")?
        .to_owned();
    let branch = format!(
        "codex/drifter-{}",
        &hash(format!("{}{}", draft.base_sha, csv).as_bytes())[..20]
    );
    let head = format!("{owner}:{branch}");
    let existing = api(
        &format!(
            "repos/{REPO}/pulls?state=all&head={}&base={}",
            encode(&head),
            encode(&draft.base_branch)
        ),
        None,
        token,
    )
    .await?;
    if let Some(pr) = existing.as_array().and_then(|a| a.first()) {
        return Ok(Submission {
            url: string(pr, "html_url")?,
            existing: true,
        });
    }
    let endpoint = format!("repos/{target}");
    // A newly created fork can take a moment to become readable. User can retry safely.
    let base = api(
        &format!("{endpoint}/git/commits/{}", draft.base_sha),
        None,
        token,
    )
    .await
    .map_err(|e| format!("{e} If a fork was just created, wait a few seconds and submit again."))?;
    let references = api(
        &format!("{endpoint}/git/matching-refs/heads/{}", encode(&branch)),
        None,
        token,
    )
    .await?;
    let reference = references.as_array().and_then(|a| {
        a.iter()
            .find(|r| r["ref"].as_str() == Some(&format!("refs/heads/{branch}")))
    });
    let source_digest = hash(csv.as_bytes());
    if let Some(reference) = reference {
        let sha = reference["object"]["sha"]
            .as_str()
            .ok_or("Branch SHA missing")?;
        let csv_response = client()?
            .get(format!(
                "https://api.github.com/{endpoint}/contents/{CSV_PATH}?ref={}",
                encode(sha)
            ))
            .header("Accept", "application/vnd.github.raw+json")
            .bearer_auth(token.unwrap())
            .send()
            .await
            .map_err(|_| "Could not verify existing branch")?;
        if bounded(csv_response).await? != csv {
            return Err("Submission branch has different CSV content".into());
        }
        let manifest_response = client()?
            .get(format!(
                "https://api.github.com/{endpoint}/contents/{MANIFEST_PATH}?ref={}",
                encode(sha)
            ))
            .header("Accept", "application/vnd.github.raw+json")
            .bearer_auth(token.unwrap())
            .send()
            .await
            .map_err(|_| "Could not verify existing branch")?;
        let manifest: Value = serde_json::from_str(&bounded(manifest_response).await?)
            .map_err(|_| "Invalid branch manifest")?;
        let found = manifest["files"].as_array().and_then(|a| {
            a.iter()
                .find(|f| f["source"]["path"] == "payload/translation.csv")
        });
        if found.and_then(|f| f["source"]["sha256"].as_str()) != Some(&source_digest) {
            return Err("Submission branch already exists with different content".into());
        }
    } else {
        let mut manifest: Value =
            serde_json::from_str(&raw(MANIFEST_PATH, &draft.base_sha, token).await?)
                .map_err(|_| "Invalid patch manifest")?;
        let file = manifest["files"]
            .as_array_mut()
            .and_then(|a| {
                a.iter_mut()
                    .find(|f| f["source"]["path"] == "payload/translation.csv")
            })
            .ok_or("Translation payload not in manifest")?;
        file["source"]["sha256"] = json!(source_digest);
        let manifest_text = serde_json::to_string_pretty(&manifest).unwrap() + "\n";
        let mut tree = Vec::new();
        for (path, text) in [(CSV_PATH, csv), (MANIFEST_PATH, manifest_text)] {
            let blob = api(
                &format!("{endpoint}/git/blobs"),
                Some(json!({"content":text,"encoding":"utf-8"})),
                token,
            )
            .await?;
            tree.push(
                json!({"path":path,"mode":"100644","type":"blob","sha":string(&blob,"sha")?}),
            );
        }
        let created = api(
            &format!("{endpoint}/git/trees"),
            Some(json!({"base_tree":base["tree"]["sha"],"tree":tree})),
            token,
        )
        .await?;
        let commit = api(&format!("{endpoint}/git/commits"),Some(json!({"message":"POC: Drifter translation corrections from Kitor","tree":string(&created,"sha")?,"parents":[draft.base_sha]})),token).await?;
        api(
            &format!("{endpoint}/git/refs"),
            Some(json!({"ref":format!("refs/heads/{branch}"),"sha":string(&commit,"sha")?})),
            token,
        )
        .await?;
    }
    let pr = api(&format!("repos/{REPO}/pulls"),Some(json!({
        "head":head,"base":draft.base_branch,"draft":true,
        "title":format!("[POC — do not merge] Drifter: {} correction(s) from Kitor",draft.edits.len()),
        "body":format!("Submitted by @{login} from Kitor’s desktop translation editor.\n\nBase: `{}`. Only edited CSV records and the payload checksum change.\n\n**POC: do not merge or publish.** Release version is unchanged. No gameplay validation or package build was run by submission. Hebrew remains in Drifter’s existing display order.",draft.base_sha)
    })),token).await?;
    Ok(Submission {
        url: string(&pr, "html_url")?,
        existing: false,
    })
}

#[derive(serde::Serialize)]
pub struct RepositorySetup {
    pub repository: Option<String>,
    pub message: String,
    pub fork_url: String,
    pub install_url: String,
}
fn selected_installation(installation: &Value, login: &str) -> bool {
    installation["app_slug"] == "kitor-translation-contributions"
        && installation["account"]["login"].as_str() == Some(login)
        && installation["repository_selection"] == "selected"
        && installation["suspended_at"].is_null()
        && installation["permissions"]["contents"] == "write"
        && installation["permissions"]["pull_requests"] == "write"
}
pub async fn repository_setup(token: &str) -> Result<RepositorySetup, String> {
    let token = Some(token);
    let user = api("user", None, token).await?;
    let login = string(&user, "login")?;
    let mut setup = RepositorySetup {
        repository: None,
        message: "Create your Kitor fork, then install Kitor Translation Contributions with Only select repositories and select just that fork. Return here and check access.".into(),
        fork_url: format!("https://github.com/{REPO}/fork"),
        install_url: "https://github.com/apps/kitor-translation-contributions/installations/new".into(),
    };
    for page in 1..=10 {
        let response = api(
            &format!("user/installations?per_page=100&page={page}"),
            None,
            token,
        )
        .await?;
        let installations = response["installations"]
            .as_array()
            .ok_or("Invalid installation response")?;
        for installation in installations {
            if installation["app_slug"] != "kitor-translation-contributions"
                || installation["account"]["login"].as_str() != Some(&login)
            {
                continue;
            }
            let id = installation["id"]
                .as_u64()
                .ok_or("Installation ID missing")?;
            setup.install_url = format!("https://github.com/settings/installations/{id}");
            if !selected_installation(installation, &login) {
                setup.message = "Choose Only select repositories and grant Contents and Pull requests read/write for just your Kitor repository. All repositories access is not accepted by this editor.".into();
                return Ok(setup);
            }
            let repositories = api(
                &format!("user/installations/{id}/repositories?per_page=100"),
                None,
                token,
            )
            .await?;
            if repositories["total_count"].as_u64() != Some(1) {
                setup.message = "Select exactly one repository for this app: your Kitor fork (or yhaggai/kitor if you own it).".into();
                return Ok(setup);
            }
            let repository = repositories["repositories"]
                .as_array()
                .and_then(|r| r.first())
                .ok_or("Selected repository missing")?;
            let name = string(repository, "full_name")?;
            let verified = api(&format!("repos/{name}"), None, token).await?;
            let own_repo = verified["owner"]["login"].as_str() == Some(&login);
            let kitor = name == REPO
                || (verified["fork"] == true && verified["parent"]["full_name"] == REPO);
            if own_repo && kitor && verified["permissions"]["push"] == true {
                setup.message = format!("Ready to submit through {name}");
                setup.repository = Some(name);
            }
            return Ok(setup);
        }
        if installations.len() < 100 {
            return Ok(setup);
        }
    }
    Err("Too many GitHub installations to check. Contact the Kitor maintainer.".into())
}

#[cfg(test)]
mod permission_tests {
    use super::*;
    #[test]
    fn requires_selected_repository_and_correct_app_and_permissions() {
        let valid = json!({"app_slug":"kitor-translation-contributions","account":{"login":"contributor"},"repository_selection":"selected","permissions":{"contents":"write","pull_requests":"write"},"suspended_at":null});
        assert!(selected_installation(&valid, "contributor"));
        assert!(!selected_installation(&valid, "other-user"));
        for (field, value) in [
            ("repository_selection", json!("all")),
            ("app_slug", json!("another-app")),
            ("suspended_at", json!("2026-01-01")),
            (
                "permissions",
                json!({"contents":"read","pull_requests":"write"}),
            ),
        ] {
            let mut invalid = valid.clone();
            invalid[field] = value;
            assert!(!selected_installation(&invalid, "contributor"));
        }
    }
}
