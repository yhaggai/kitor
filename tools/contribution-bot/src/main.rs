//! Local-only submission service. GitHub credentials never enter Kitor or its webview.
use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
#[path = "../../../kitor/src-tauri/src/services/contributions/csv_text.rs"]
mod csv_text;
mod github;
const REPO: &str = "yhaggai/kitor";
const CSV_PATH: &str = "patches-workspace/drifter/he/payload/translation.csv";
const MANIFEST_PATH: &str = "patches-workspace/drifter/he/patch.json";
struct Draft {
    base_sha: String,
    base_branch: String,
    edits: BTreeMap<String, String>,
}
#[derive(Serialize)]
struct Submission {
    url: String,
    existing: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    base_sha: String,
    digest: String,
    edits: BTreeMap<String, String>,
}
struct App {
    key: Option<jsonwebtoken::EncodingKey>,
    gate: Mutex<()>,
}
type ApiError = (StatusCode, Json<Value>);
fn error(code: StatusCode, message: impl Into<String>) -> ApiError {
    (code, Json(json!({"error":message.into()})))
}
fn validate(request: &Request) -> Result<(), String> {
    if request.base_sha.len() != 40 || !request.base_sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid base commit".into());
    }
    if request.digest.len() != 64 || !request.digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid preview digest".into());
    }
    if request.edits.is_empty() || request.edits.len() > 200 {
        return Err("Submit between 1 and 200 corrections".into());
    }
    for (key, value) in &request.edits {
        if key.len() != 64
            || !key.bytes().all(|b| b.is_ascii_hexdigit())
            || value.trim().is_empty()
            || value.len() > 16000
            || value.contains('\0')
        {
            return Err("Invalid correction".into());
        }
    }
    Ok(())
}
async fn installation_token(key: &jsonwebtoken::EncodingKey) -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Invalid clock")?
        .as_secs();
    let jwt = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        &json!({"iat":now.saturating_sub(60),"exp":now+540,"iss":"5666467"}),
        key,
    )
    .map_err(|_| "Could not sign bot authentication")?;
    let installation = github::api(&format!("repos/{REPO}/installation"), None, Some(&jwt)).await?;
    if installation["app_id"].as_u64() != Some(5666467) {
        return Err("Unexpected GitHub App installation".into());
    }
    let id = installation["id"]
        .as_u64()
        .ok_or("Bot installation missing")?;
    let result=github::api(&format!("app/installations/{id}/access_tokens"),Some(json!({"repositories":["kitor"],"permissions":{"contents":"write","pull_requests":"write"}})),Some(&jwt)).await?;
    github::string(&result, "token")
}
async fn health(State(app): State<Arc<App>>) -> Json<Value> {
    Json(
        json!({"service":"kitor-contribution-bot","configured":app.key.is_some(),"local_only":true}),
    )
}
async fn submit(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    Json(request): Json<Request>,
) -> Result<Json<Submission>, ApiError> {
    // No CORS: websites cannot drive the local bot using a browser session.
    if headers.contains_key("origin")
        || headers.get("x-kitor-client").and_then(|h| h.to_str().ok()) != Some("desktop-poc")
    {
        return Err(error(StatusCode::FORBIDDEN, "Use the Kitor desktop editor"));
    }
    validate(&request).map_err(|e| error(StatusCode::BAD_REQUEST, e))?;
    let _guard = app.gate.try_lock().map_err(|_| {
        error(
            StatusCode::TOO_MANY_REQUESTS,
            "Another submission is running. Retry shortly.",
        )
    })?;
    let key = app.key.as_ref().ok_or_else(|| {
        error(
            StatusCode::SERVICE_UNAVAILABLE,
            "The local bot needs its GitHub App private key and repository installation.",
        )
    })?;
    let result = submit_validated(&request, key)
        .await
        .map_err(|e| error(StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(result))
}
async fn submit_validated(
    request: &Request,
    key: &jsonwebtoken::EncodingKey,
) -> Result<Submission, String> {
    let token = installation_token(key).await?;
    let repo = github::api(&format!("repos/{REPO}"), None, Some(&token)).await?;
    let branch = github::string(&repo, "default_branch")?;
    let comparison = github::api(
        &format!(
            "repos/{REPO}/compare/{}...{}",
            request.base_sha,
            github::encode(&branch)
        ),
        None,
        Some(&token),
    )
    .await?;
    if !matches!(comparison["status"].as_str(), Some("ahead" | "identical")) {
        return Err("Draft base is not on the current upstream branch".into());
    }
    let source = github::raw(CSV_PATH, &request.base_sha, Some(&token)).await?;
    let csv = csv_text::apply(&source, &request.edits)?;
    if csv == source {
        return Err("No corrections to submit".into());
    }
    if csv_text::hash(csv.as_bytes()) != request.digest {
        return Err("The submitted preview differs from the validated corrections".into());
    }
    let draft = Draft {
        base_sha: request.base_sha.clone(),
        base_branch: branch,
        edits: request.edits.clone(),
    };
    github::submit(&draft, csv, &token).await
}
fn router(app: Arc<App>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/submissions", post(submit))
        .layer(DefaultBodyLimit::max(1_000_000))
        .with_state(app)
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = match std::env::var("KITOR_BOT_PRIVATE_KEY_FILE") {
        Ok(path) => Some(jsonwebtoken::EncodingKey::from_rsa_pem(&std::fs::read(
            path,
        )?)?),
        Err(_) => None,
    };
    let app = Arc::new(App {
        key,
        gate: Mutex::new(()),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:47831").await?;
    eprintln!(
        "Kitor contribution bot listening on http://127.0.0.1:47831 (local only; configured={})",
        app.key.is_some()
    );
    axum::serve(listener, router(app)).await?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request as HttpRequest};
    use tower::ServiceExt;
    fn sample() -> Request {
        Request {
            base_sha: "a".repeat(40),
            digest: "b".repeat(64),
            edits: BTreeMap::from([("c".repeat(64), "תיקון".into())]),
        }
    }
    #[test]
    fn rejects_bad_inputs() {
        let mut r = sample();
        assert!(validate(&r).is_ok());
        r.base_sha = "../../main".into();
        assert!(validate(&r).is_err());
        let mut r = sample();
        r.edits.clear();
        assert!(validate(&r).is_err());
        let mut r = sample();
        r.edits.values_mut().for_each(|v| *v = "\0".into());
        assert!(validate(&r).is_err());
    }
    #[tokio::test]
    async fn local_endpoint_rejects_browser_and_unconfigured_submissions() {
        let request = || {
            HttpRequest::builder()
                .method("POST")
                .uri("/submissions")
                .header("content-type", "application/json")
        };
        let payload=json!({"base_sha":"a".repeat(40),"digest":"b".repeat(64),"edits":{"c".repeat(64):"תיקון"}}).to_string();
        let app = || {
            router(Arc::new(App {
                key: None,
                gate: Mutex::new(()),
            }))
        };
        let response = app()
            .oneshot(
                request()
                    .header("origin", "https://example.org")
                    .header("x-kitor-client", "desktop-poc")
                    .body(Body::from(payload.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let response = app()
            .oneshot(
                request()
                    .header("x-kitor-client", "desktop-poc")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
