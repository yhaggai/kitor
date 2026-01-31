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
    
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://embed.gog.com/account/getFilteredProducts")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch games: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to fetch games: {}", error_text));
    }
    
    // Parse GOG response and extract games
    // Note: The actual API structure may need adjustment based on GOG's response
    let games_data: serde_json::Value = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse games response: {}", e))?;
    
    // Extract games from response (structure may vary)
    let games = games_data
        .get("products")
        .and_then(|p| p.as_array())
        .map(|products| {
            products
                .iter()
                .filter_map(|product| {
                    let id = product.get("id")?.as_i64()?;
                    let title = product.get("title")?.as_str()?.to_string();
                    let cover_url = product
                        .get("image")
                        .and_then(|i| i.as_str())
                        .map(|s| s.to_string());
                    
                    Some(GogGame {
                        id,
                        title,
                        cover_url,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    
    Ok(games)
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
