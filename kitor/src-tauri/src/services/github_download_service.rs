//! Optional private-repository access. Credentials never leave the GitHub API host.
use reqwest::{Client, Response};
use std::time::Duration;

const REPO: &str = "yhaggai/kitor";
fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new("click.kitor.github", "patch-download-token")
        .map_err(|_| "Could not open the system credential store".into())
}
fn token() -> Result<Option<String>, String> {
    match entry()?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Could not read the GitHub token from the system credential store".into()),
    }
}
pub fn configured() -> Result<bool, String> {
    Ok(token()?.is_some())
}
pub fn remove() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Could not remove the GitHub token".into()),
    }
}
fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Kitor-patch-updater")
        .connect_timeout(Duration::from_secs(20))
        .read_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not initialize GitHub downloads".into())
}
fn api_url(path: &str) -> Result<String, String> {
    let prefix = format!("/repos/{REPO}");
    if path != prefix && !path.starts_with(&(prefix + "/")) {
        return Err("GitHub request is outside the configured repository".into());
    }
    if path.contains(['\r', '\n', '#', '\\']) || path.contains("..") {
        return Err("Invalid GitHub API path".into());
    }
    Ok(format!("https://api.github.com{path}"))
}
async fn api_get(path: &str, accept: &str, credential: Option<&str>) -> Result<Response, String> {
    let mut request = client()?
        .get(api_url(path)?)
        .header("Accept", accept)
        .header("X-GitHub-Api-Version", "2022-11-28");
    if accept != "application/octet-stream" {
        request = request.timeout(Duration::from_secs(20));
    }
    if let Some(value) = credential {
        request = request.bearer_auth(value);
    }
    request
        .send()
        .await
        .map_err(|_| "Could not reach GitHub. Check your connection.".into())
}
fn checked(response: Response) -> Result<Response, String> {
    if response.status().is_success() {
        return Ok(response);
    }
    Err(match response.status().as_u16() {
        401 => "GitHub token is invalid or expired. Replace it in Settings.".into(),
        403 => "GitHub denied access or its rate limit was reached. Check token permissions and try again later.".into(),
        404 => "GitHub resource was not found or is private. Check the release and repository access in Settings.".into(),
        status => format!("GitHub download failed: HTTP {status}"),
    })
}
pub async fn save(value: String) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 1024
        || value
            .bytes()
            .any(|c| c.is_ascii_whitespace() || c.is_ascii_control())
    {
        return Err("Enter a valid GitHub personal access token".into());
    }
    // Verify access to repository contents before replacing an existing credential.
    checked(
        api_get(
            &format!("/repos/{REPO}/contents/kitor/patches/registry.json?ref=v1"),
            "application/vnd.github+json",
            Some(value),
        )
        .await?,
    )?;
    entry()?
        .set_password(value)
        .map_err(|_| "Could not save the GitHub token in the system credential store".into())
}
pub async fn registry() -> Result<Response, String> {
    let credential = token()?;
    checked(
        api_get(
            &format!("/repos/{REPO}/contents/registry.json?ref=patch-registry"),
            "application/vnd.github.raw+json",
            credential.as_deref(),
        )
        .await?,
    )
}
fn asset_parts(value: &str) -> Option<(String, String)> {
    let url = url::Url::parse(value).ok()?;
    if url.scheme() != "https"
        || url.host_str()? != "github.com"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    let parts: Vec<_> = url.path_segments()?.collect();
    if parts.len() != 6
        || format!("{}/{}", parts[0], parts[1]) != REPO
        || parts[2] != "releases"
        || parts[3] != "download"
        || parts.iter().any(|p| {
            p.is_empty()
                || !p
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        })
    {
        return None;
    }
    Some((parts[4].into(), parts[5].into()))
}
fn trusted_asset_redirect(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && url.host_str().is_some_and(|host| {
                host == "release-assets.githubusercontent.com"
                    || host == "objects.githubusercontent.com"
            })
    })
}
#[derive(serde::Deserialize)]
struct Release {
    assets: Vec<Asset>,
}
#[derive(serde::Deserialize)]
struct Asset {
    id: u64,
    name: String,
}
pub async fn download(url: &str) -> Result<Option<Response>, String> {
    let Some((tag, filename)) = asset_parts(url) else {
        return Ok(None);
    };
    let credential = token()?;
    let release: Release = checked(
        api_get(
            &format!("/repos/{REPO}/releases/tags/{tag}"),
            "application/vnd.github+json",
            credential.as_deref(),
        )
        .await?,
    )?
    .json()
    .await
    .map_err(|_| "Invalid GitHub release metadata")?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == filename)
        .ok_or("Patch asset is missing from the GitHub release")?;
    let mut response = api_get(
        &format!("/repos/{REPO}/releases/assets/{}", asset.id),
        "application/octet-stream",
        credential.as_deref(),
    )
    .await?;
    for _ in 0..5 {
        if !response.status().is_redirection() {
            return checked(response).map(Some);
        }
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or("GitHub asset redirect is missing")?;
        if !trusted_asset_redirect(location) {
            return Err("Untrusted GitHub asset redirect".into());
        }
        // A fresh client/request intentionally has no Authorization header.
        response = client()?
            .get(location)
            .send()
            .await
            .map_err(|_| "Could not download the GitHub release asset")?;
    }
    Err("Too many GitHub asset redirects".into())
}

#[cfg(test)]
mod tests;
