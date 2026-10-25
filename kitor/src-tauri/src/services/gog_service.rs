use sea_orm::{entity::*, query::*, DatabaseConnection};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use base64::{Engine as _, engine::general_purpose};
use crate::models::gog_auth;

const CLIENT_ID: &str = "46899977096215655";
const CLIENT_SECRET: &str = "9d85c43b1482497dbbce61f6e4aa173a433796eeae2ca8c5f6129f2dc4de46d9";
const AUTH_URL: &str = "https://auth.gog.com/auth";
const TOKEN_URL: &str = "https://auth.gog.com/token";
const REDIRECT_URI: &str = "https://embed.gog.com/on_login_success?origin=client";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GogGame {
    pub id: i64,
    pub title: String,
    pub cover_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfflineInstaller {
    pub id: String,
    pub name: String,
    pub path: String,
}

pub fn build_auth_url() -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&layout=client2",
        AUTH_URL, CLIENT_ID, REDIRECT_URI
    )
}

pub async fn exchange_code_for_token(code: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", REDIRECT_URI),
    ];
    
    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {}", error_text));
    }
    
    let response = response
        .json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;
    
    Ok(response)
}

pub async fn refresh_access_token(db: &DatabaseConnection) -> Result<TokenResponse, String> {
    let auth = gog_auth::Entity::find()
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("No auth token found")?;
    
    let refresh_token = decrypt_token(&auth.refresh_token)?;
    
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.as_str()),
    ];
    
    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed: {}", error_text));
    }
    
    let response = response
        .json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;
    
    Ok(response)
}

pub async fn store_tokens(
    db: &DatabaseConnection,
    access_token: &str,
    refresh_token: &str,
    expires_in: i64,
) -> Result<(), String> {
    let encrypted_access = encrypt_token(access_token);
    let encrypted_refresh = encrypt_token(refresh_token);
    let now = Utc::now();
    let expires_at = now + Duration::seconds(expires_in);
    
    // Delete existing auth
    gog_auth::Entity::delete_many()
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete old tokens: {}", e))?;
    
    // Insert new auth
    let new_auth = gog_auth::ActiveModel {
        access_token: Set(encrypted_access),
        refresh_token: Set(encrypted_refresh),
        expires_at: Set(expires_at.naive_utc()),
        created_at: Set(now.naive_utc()),
        updated_at: Set(now.naive_utc()),
        ..Default::default()
    };
    
    new_auth
        .insert(db)
        .await
        .map_err(|e| format!("Failed to store tokens: {}", e))?;
    
    Ok(())
}

pub async fn get_valid_token(db: &DatabaseConnection) -> Result<String, String> {
    let auth = gog_auth::Entity::find()
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Not authenticated")?;
    
    let now: DateTime<Utc> = Utc::now();
    let expires_at: DateTime<Utc> = DateTime::from_naive_utc_and_offset(auth.expires_at, Utc);
    
    // If token expires in less than 5 minutes, refresh it
    if expires_at < now + Duration::minutes(5) {
        let token_response = refresh_access_token(db).await?;
        store_tokens(
            db,
            &token_response.access_token,
            &token_response.refresh_token,
            token_response.expires_in,
        )
        .await?;
        Ok(token_response.access_token)
    } else {
        decrypt_token(&auth.access_token)
    }
}

pub async fn logout(db: &DatabaseConnection) -> Result<(), String> {
    gog_auth::Entity::delete_many()
        .exec(db)
        .await
        .map_err(|e| format!("Failed to logout: {}", e))?;
    
    Ok(())
}

pub async fn is_authenticated(db: &DatabaseConnection) -> Result<bool, String> {
    let count = gog_auth::Entity::find()
        .count(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    
    Ok(count > 0)
}

pub async fn get_user_games(db: &DatabaseConnection) -> Result<Vec<GogGame>, String> {
    let token = get_valid_token(db).await?;
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create GOG client: {}", e))?;
    
    // 1. Get owned game IDs from the correct endpoint
    let response = client
        .get("https://www.gog.com/user/data/games")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch owned games: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch owned games (HTTP {}). Try reconnecting your GOG account if this persists.", response.status()));
    }
    
    let data: serde_json::Value = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse owned games response: {}", e))?;
    
    let owned_ids = data
        .get("owned")
        .and_then(|v| v.as_array())
        .ok_or("Could not find 'owned' games in response")?;
    
    if owned_ids.is_empty() {
        return Ok(vec![]);
    }

    // 2. Fetch game details in chunks
    let mut all_games: Vec<GogGame> = owned_ids
        .iter()
        .map(|value| {
            let id = value.as_i64()
                .or_else(|| value.as_str().and_then(|id| id.parse::<i64>().ok()))
                .filter(|id| *id > 0)
                .ok_or_else(|| "Invalid game ID in GOG ownership response".to_string())?;
            Ok(GogGame { id, title: format!("GOG game {}", id), cover_url: None })
        })
        .collect::<Result<_, String>>()?;
    all_games.sort_by_key(|game| game.id);
    all_games.dedup_by_key(|game| game.id);

    // Ownership is authoritative; unavailable catalog metadata must not remove games.
    for chunk in all_games.chunks_mut(50) {
        let ids_param = chunk.iter().map(|game| game.id.to_string()).collect::<Vec<_>>().join(",");
        let response = client
            .get("https://api.gog.com/products")
            .query(&[("ids", ids_param)])
            .send().await;
        if let Ok(response) = response {
            if response.status().is_success() {
                if let Ok(products) = response.json::<Vec<serde_json::Value>>().await {
                    apply_product_metadata(chunk, &products);
                }
            }
        }
    }

    // Sort games by title
    all_games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    Ok(all_games)
}

fn apply_product_metadata(games: &mut [GogGame], products: &[serde_json::Value]) {
    for game in games {
        if let Some(product) = products.iter().find(|p| p.get("id").and_then(|id| id.as_i64()) == Some(game.id)) {
            if let Some(title) = product.get("title").and_then(|v| v.as_str()).filter(|title| !title.is_empty()) {
                game.title = title.to_string();
            }
            game.cover_url = product.pointer("/images/logo2x")
                .or_else(|| product.get("image"))
                .and_then(|v| v.as_str())
                .map(|url| if url.starts_with("//") { format!("https:{}", url) } else { url.to_string() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Opt-in diagnostic: never uses a saved account during ordinary test runs.
    #[tokio::test]
    #[ignore = "requires an explicitly authorized GOG_TEST_DATABASE_URL and network access"]
    async fn live_owned_library() {
        let url = std::env::var("GOG_TEST_DATABASE_URL").expect("GOG_TEST_DATABASE_URL is required");
        let db = sea_orm::Database::connect(url).await.unwrap();
        let games = get_user_games(&db).await.unwrap();
        let expected: usize = std::env::var("GOG_TEST_EXPECTED_COUNT").expect("expected count is required").parse().unwrap();
        assert_eq!(games.len(), expected);
        println!("Verified {} owned games; {} have catalog titles", games.len(), games.iter().filter(|game| !game.title.starts_with("GOG game ")).count());
    }

    #[test]
    fn missing_metadata_preserves_ownership() {
        let mut games = vec![
            GogGame { id: 1, title: "GOG game 1".into(), cover_url: None },
            GogGame { id: 2, title: "GOG game 2".into(), cover_url: None },
        ];
        apply_product_metadata(&mut games, &[]);
        assert_eq!(games.len(), 2);
        apply_product_metadata(&mut games, &[
            serde_json::json!({"id": 1, "title": "Owned game", "images": {"logo2x": "//images.gog.com/cover.jpg"}}),
            serde_json::json!({"id": 3, "title": "Not owned"}),
        ]);
        assert_eq!(games.iter().map(|game| game.id).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(games[0].title, "Owned game");
        assert_eq!(games[0].cover_url.as_deref(), Some("https://images.gog.com/cover.jpg"));
        assert_eq!(games[1].title, "GOG game 2");
    }
}

pub async fn get_game_installers(db: &DatabaseConnection, game_id: i64) -> Result<Vec<OfflineInstaller>, String> {
    let token = get_valid_token(db).await?;
    let client = reqwest::Client::new();
    
    let url = format!("https://www.gog.com/account/gameDetails/{}.json", game_id);
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch game details: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to fetch game details ({}): {}", status, error_text));
    }
    
    let details: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse game details JSON: {}", e))?;
    
    let mut installers = Vec::new();
    
    // Look for downloads in the 'downloads' array
    if let Some(downloads) = details.get("downloads").and_then(|d| d.as_array()) {
        for download in downloads {
            // downloads is usually an array of pairs [language, details]
            if let Some(entry) = download.as_array() {
                if entry.len() >= 2 {
                    if let Some(platform_details) = entry[1].get("windows").and_then(|w| w.as_array()) {
                        for item in platform_details {
                            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Installer").to_string();
                            let path = item.get("manualUrl").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                            
                            if !path.is_empty() {
                                installers.push(OfflineInstaller {
                                    id,
                                    name,
                                    path: format!("https://www.gog.com{}", path),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(installers)
}

fn encrypt_token(token: &str) -> String {
    general_purpose::STANDARD.encode(token.as_bytes())
}

fn decrypt_token(encrypted: &str) -> Result<String, String> {
    let decoded = general_purpose::STANDARD
        .decode(encrypted.as_bytes())
        .map_err(|e| format!("Failed to decrypt token: {}", e))?;
    
    String::from_utf8(decoded).map_err(|e| format!("Invalid token encoding: {}", e))
}
