use axum::{extract::{Path, State}, http::StatusCode, Json, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};
use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
use argon2::password_hash::SaltString;
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use std::env;
use rand::Rng;
use axum_extra::extract::TypedHeader;
use axum_extra::headers::authorization::{Authorization, Bearer};
use std::fs;
use std::path::Path as StdPath;
use std::sync::Arc;
use crate::db::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Database>,
}

/// Wubi dictionary entry loaded from JSON file.
#[derive(Debug, Deserialize)]
struct WubiDictEntry {
    character: String,
    code: String,
}

/// Load wubi dictionary from JSON file.
fn load_wubi_dict() -> Vec<WubiDictEntry> {
    let dict_path = "data/wubi_dict.json";
    
    if !StdPath::new(dict_path).exists() {
        eprintln!("Wubi dictionary file not found: {}", dict_path);
        return Vec::new();
    }
    
    match fs::read_to_string(dict_path) {
        Ok(content) => {
            match serde_json::from_str::<Vec<WubiDictEntry>>(&content) {
                Ok(entries) => {
                    println!("Loaded {} entries from wubi dictionary", entries.len());
                    entries
                }
                Err(e) => {
                    eprintln!("Failed to parse wubi dictionary JSON: {:?}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read wubi dictionary file: {:?}", e);
            Vec::new()
        }
    }
}

/// A user record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Input payload for user login.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Input payload for user registration.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Output payload for user login.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub user: crate::config::User,
}

/// Claims for JWT token.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

/// A Wubi tutorial lesson record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lesson {
    pub id: i32,
    pub character: String,
    pub code: String,
    pub description: String,
}

/// Input payload for creating a new lesson.
#[derive(Debug, Deserialize)]
pub struct NewLesson {
    pub character: String,
    pub code: String,
    pub description: String,
}

/// Input payload for recording progress.
#[derive(Debug, Deserialize)]
pub struct ProgressUpdate {
    pub user_name: String,
    pub lesson_id: i32,
    pub accuracy: f32,
    pub score: i32,
}

/// Default MySQL connection string for local development.
pub fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "mysql://root:sdsSDG123*^DD@127.0.0.1:3306/wubi".to_string()
    })
}

/// A Wubi practice article record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub difficulty: String,
}

/// Input payload for creating a new article.
#[derive(Debug, Deserialize)]
pub struct NewArticle {
    pub title: String,
    pub content: String,
    pub difficulty: String,
}

/// A Wubi root character record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WubiRoot {
    pub id: i32,
    pub character: String,
    pub code: String,
    pub position: String,
    pub description: String,
}

/// Input payload for creating a new wubi root.
#[derive(Debug, Deserialize)]
pub struct NewWubiRoot {
    pub character: String,
    pub code: String,
    pub position: String,
    pub description: String,
}

/// A simple health check endpoint.
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

/// GET /api/lessons
pub async fn get_lessons(State(state): State<AppState>) -> Result<Json<Vec<crate::config::Lesson>>, StatusCode> {
    let lessons = state.db.get_lessons().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(lessons))
}

pub async fn get_lesson(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::Lesson>, StatusCode> {
    let lesson = state.db.get_lesson_by_id(id).await.map_err(|err| match err.as_str() {
        "Lesson not found" => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(lesson))
}

pub async fn create_lesson(
    State(state): State<AppState>,
    Json(payload): Json<NewLesson>,
) -> Result<(StatusCode, Json<crate::config::Lesson>), (StatusCode, String)> {
    let lesson = state.db.create_lesson(&payload.character, &payload.code, &payload.description).await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    Ok((StatusCode::CREATED, Json(lesson)))
}

pub async fn post_progress(
    State(state): State<AppState>,
    Json(payload): Json<ProgressUpdate>,
) -> Result<StatusCode, StatusCode> {
    state.db.save_progress(&payload.user_name, payload.lesson_id, payload.accuracy, payload.score).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::CREATED)
}

pub async fn get_articles(State(state): State<AppState>) -> Result<Json<Vec<crate::config::Article>>, StatusCode> {
    let articles = state.db.get_articles().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(articles))
}

pub async fn get_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::Article>, StatusCode> {
    let article = state.db.get_article_by_id(id).await.map_err(|err| match err.as_str() {
        "Article not found" => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(article))
}

pub async fn create_article(
    State(state): State<AppState>,
    Json(payload): Json<NewArticle>,
) -> Result<(StatusCode, Json<crate::config::Article>), (StatusCode, String)> {
    let article = state.db.create_article(&payload.title, &payload.content, &payload.difficulty).await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    Ok((StatusCode::CREATED, Json(article)))
}

pub async fn update_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<NewArticle>,
) -> Result<Json<crate::config::Article>, (StatusCode, String)> {
    let article = state.db.update_article(id, &payload.title, &payload.content, &payload.difficulty).await
        .map_err(|err| match err.as_str() {
            "Article not found" => (StatusCode::NOT_FOUND, err),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err),
        })?;
    Ok(Json(article))
}

pub async fn delete_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.db.delete_article(id).await
        .map_err(|err| match err.as_str() {
            "Article not found" => (StatusCode::NOT_FOUND, err),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct UpdateWubiCode {
    pub character: String,
    pub code: String,
}

pub async fn update_wubi_code_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdateWubiCode>,
) -> Result<Json<crate::config::WubiCharacter>, StatusCode> {
    let result = state.db.update_wubi_code(&payload.character, &payload.code).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(result))
}

pub async fn get_wubi_roots(State(state): State<AppState>) -> Result<Json<Vec<crate::config::WubiRoot>>, StatusCode> {
    let roots = state.db.get_wubi_roots().await.map_err(|e| {
        eprintln!("Error fetching wubi roots: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(roots))
}

pub async fn get_wubi_root(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::WubiRoot>, StatusCode> {
    let root = state.db.get_wubi_root_by_id(id).await.map_err(|err| match err.as_str() {
        "Wubi root not found" => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(root))
}

pub async fn create_wubi_root(
    State(state): State<AppState>,
    Json(payload): Json<NewWubiRoot>,
) -> Result<(StatusCode, Json<crate::config::WubiRoot>), (StatusCode, String)> {
    let root = state.db.create_wubi_root(&payload.character, &payload.code, &payload.position, &payload.description).await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    Ok((StatusCode::CREATED, Json(root)))
}

pub async fn search_wubi_root(
    Path(character): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Option<crate::config::WubiRoot>>, StatusCode> {
    let root = state.db.search_wubi_root(&character).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(root))
}

pub async fn get_wubi_code(
    Path(character): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::WubiCharacter>, StatusCode> {
    if character.chars().count() != 1 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let result = state.db.get_wubi_code(&character).await
        .map_err(|err| match err.as_str() {
            "Character not found" => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(result))
}

pub async fn get_key_radicals(State(state): State<AppState>) -> Result<Json<Vec<crate::config::KeyRadical>>, StatusCode> {
    let radicals = state.db.get_key_radicals().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(radicals))
}

pub async fn get_key_radical_by_key(
    Path(key_char): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::KeyRadical>, StatusCode> {
    let result = state.db.get_key_radical_by_key(&key_char).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Some(radical) => Ok(Json(radical)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_english_texts(State(state): State<AppState>) -> Result<Json<Vec<crate::config::EnglishText>>, StatusCode> {
    let texts = state.db.get_english_texts().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(texts))
}

/// Generate JWT token for user.
pub fn generate_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let expiration = Utc::now() + Duration::hours(24);
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp(),
    };
    
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::default(), &claims, &key)
}

/// Validate JWT token.
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let validation = Validation::new(Algorithm::HS256);
    
    let key = DecodingKey::from_secret(secret.as_bytes());
    let decoded = decode::<Claims>(token, &key, &validation)?;
    Ok(decoded.claims)
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user_opt = state.db.get_user_by_username(&payload.username).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match user_opt {
        Some(user) => {
            let argon2 = Argon2::default();
            let password_hash = PasswordHash::new(&user.password_hash)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            if argon2.verify_password(payload.password.as_bytes(), &password_hash)
                .is_err() {
                return Err(StatusCode::UNAUTHORIZED);
            }
            
            let token = generate_token(user.id)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            Ok(Json(LoginResponse {
                access_token: token,
                user,
            }))
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Logout user (client-side logout by removing token).
pub async fn logout() -> StatusCode {
    // Logout is handled client-side by removing the token
    StatusCode::OK
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let existing_user = state.db.get_user_by_username(&payload.username).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }
    
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();
    
    let user = state.db.create_user(&payload.username, &payload.email, &password_hash).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let token = generate_token(user.id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(LoginResponse {
        access_token: token,
        user,
    }))
}

pub async fn auth_middleware(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = auth.token();
    
    let claims = validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user_id = claims.sub.parse::<i32>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user_exists = state.db.get_user_by_id(user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match user_exists {
        Some(_) => {
            let mut req = req;
            req.extensions_mut().insert(user_id);
            Ok(next.run(req).await)
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
