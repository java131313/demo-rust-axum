use axum::{extract::{Path, State}, http::StatusCode, Json, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
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

/// Shared application state for Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
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
    pub user: User,
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

/// Initialize the database schema and insert starter lessons.
pub async fn init_db(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(64) NOT NULL UNIQUE,
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create lessons table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS lessons (
            id INT AUTO_INCREMENT PRIMARY KEY,
            character_val VARCHAR(32) NOT NULL,
            code VARCHAR(32) NOT NULL,
            description TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create user_progress table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_progress (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_name VARCHAR(64) NOT NULL,
            lesson_id INT NOT NULL,
            accuracy FLOAT NOT NULL,
            score INT NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create articles table for practice texts
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS articles (
            id INT AUTO_INCREMENT PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            content TEXT NOT NULL,
            difficulty VARCHAR(10) DEFAULT 'medium'
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create wubi_characters table for all chinese characters and their wubi codes
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wubi_characters (
            id INT AUTO_INCREMENT PRIMARY KEY,
            character_val VARCHAR(4) NOT NULL UNIQUE,
            wubi_code VARCHAR(8) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create wubi_roots table for wubi root characters
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wubi_roots (
            id INT AUTO_INCREMENT PRIMARY KEY,
            character_val VARCHAR(32) NOT NULL,
            code VARCHAR(32) NOT NULL,
            position VARCHAR(64) NOT NULL,
            description TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Insert sample lessons if table is empty
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lessons").fetch_one(pool).await?;
    if count == 0 {
        let rows = [
            ("人", "WG", "练习人字的五笔编码。"),
            ("日", "KH", "练习日字的五笔编码。"),
            ("山", "FQ", "练习山字的五笔编码。"),
        ];

        for (character, code, description) in rows {
            sqlx::query(
                "INSERT INTO lessons (character_val, code, description) VALUES (?, ?, ?)"
            )
            .bind(character)
            .bind(code)
            .bind(description)
            .execute(pool)
            .await?;
        }
    }

    // Insert sample articles if table is empty
    let count_result: Result<i64, sqlx::Error> = sqlx::query_scalar("SELECT COUNT(*) FROM articles")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eprintln!("Error querying articles count: {:?}", e);
            e
        });
    
    if let Ok(count) = count_result {
        if count == 0 {
            let rows = [
                ("练习文章一", "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。", "easy"),
                ("练习文章二", "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。", "medium"),
                ("练习文章三", "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。", "hard"),
            ];

            for (title, content, difficulty) in rows {
                if let Err(e) = sqlx::query(
                    "INSERT INTO articles (title, content, difficulty) VALUES (?, ?, ?)"
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(pool)
                .await {
                    eprintln!("Error inserting article: {:?}", e);
                }
            }
        }
    }

    // Insert sample wubi roots if table is empty
    let count_result: Result<i64, sqlx::Error> = sqlx::query_scalar("SELECT COUNT(*) FROM wubi_roots")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eprintln!("Error querying wubi_roots count: {:?}", e);
            e
        });
    
    if let Ok(count) = count_result {
        if count == 0 {
            let rows = [
                ("一", "GGLL", "G区第一键", "横区起首字根"),
                ("丿", "TTLL", "T区第一键", "撇区起首字根"),
                ("丨", "HHLL", "H区第一键", "竖区起首字根"),
                ("丶", "YYLL", "Y区第一键", "捺区起首字根"),
                ("乙", "NNLL", "N区第一键", "折区起首字根"),
                ("九", "VTNG", "V区第二键", "字根：乙"),
                ("力", "LTNN", "L键", "字根：力"),
                ("乃", "DETN", "N键", "字根：乃"),
                ("刀", "VNTE", "V键", "字根：刀"),
                ("卜", "HHYD", "H键", "字根：卜"),
            ];

            for (character, code, position, description) in rows {
                if let Err(e) = sqlx::query(
                    "INSERT INTO wubi_roots (character_val, code, position, description) VALUES (?, ?, ?, ?)"
                )
                .bind(character)
                .bind(code)
                .bind(position)
                .bind(description)
                .execute(pool)
                .await {
                    eprintln!("Error inserting wubi root: {:?}", e);
                }
            }
        }
    }

    // Clear and repopulate wubi characters table with comprehensive data from JSON dictionary
    sqlx::query("DELETE FROM wubi_characters")
        .execute(pool)
        .await
        .map_err(|e| {
            eprintln!("Error clearing wubi_characters table: {:?}", e);
            e
        })?;

    // Load dictionary from JSON file
    let dict_entries = load_wubi_dict();
    
    if !dict_entries.is_empty() {
        println!("Importing {} wubi dictionary entries to database...", dict_entries.len());
        
        // Batch insert for better performance
        let batch_size = 100;
        let mut success_count = 0;
        let mut error_count = 0;
        
        for chunk in dict_entries.chunks(batch_size) {
            for entry in chunk {
                match sqlx::query(
                    "INSERT IGNORE INTO wubi_characters (character_val, wubi_code) VALUES (?, ?)"
                )
                .bind(&entry.character)
                .bind(&entry.code)
                .execute(pool)
                .await {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        error_count += 1;
                        if error_count <= 5 {
                            eprintln!("Error inserting wubi character '{}': {:?}", entry.character, e);
                        }
                    }
                }
            }
        }
        
        println!("Dictionary import complete: {} succeeded, {} failed", success_count, error_count);
    } else {
        // Fallback: use hardcoded sample data if dictionary file not available
        println!("Using fallback sample data for wubi characters");
        let rows = [
            ("王", "GGGG"), ("李", "SBVB"), ("张", "XTAY"), ("刘", "YJLV"), ("陈", "BII"),
            ("杨", "MNR"), ("赵", "FHQI"), ("黄", "AMWU"), ("周", "MFKD"), ("吴", "KGDU"),
            ("你", "WQIY"), ("好", "VBG"), ("我", "TRNT"), ("们", "WUN"), ("学", "IPBF"),
            ("习", "NUD"), ("五", "GGHG"), ("笔", "TTH"), ("的", "RQYY"), ("一", "GGLL"),
        ];

        for (character, code) in rows {
            if let Err(e) = sqlx::query(
                "INSERT IGNORE INTO wubi_characters (character_val, wubi_code) VALUES (?, ?)"
            )
            .bind(character)
            .bind(code)
            .execute(pool)
            .await {
                eprintln!("Error inserting wubi character: {:?}", e);
            }
        }
    }

    Ok(())
}

/// A simple health check endpoint.
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

/// GET /api/lessons
pub async fn get_lessons(State(state): State<AppState>) -> Result<Json<Vec<Lesson>>, StatusCode> {
    let lessons = sqlx::query_as::<_, Lesson>(
        "SELECT id, character_val AS character, code, description FROM lessons ORDER BY id",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(lessons))
}

/// GET /api/lessons/{id}
pub async fn get_lesson(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<Lesson>, StatusCode> {
    let lesson = sqlx::query_as::<_, Lesson>(
        "SELECT id, character_val AS character, code, description FROM lessons WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(lesson))
}

/// POST /api/lessons
pub async fn create_lesson(
    State(state): State<AppState>,
    Json(payload): Json<NewLesson>,
) -> Result<(StatusCode, Json<Lesson>), (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT INTO lessons (character_val, code, description) VALUES (?, ?, ?)"
    )
    .bind(&payload.character)
    .bind(&payload.code)
    .bind(&payload.description)
    .execute(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let id: i64 = result.last_insert_id();
    let lesson = sqlx::query_as::<_, Lesson>(
        "SELECT id, character_val AS character, code, description FROM lessons WHERE id = ?",
    )
    .bind(id as i32)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok((StatusCode::CREATED, Json(lesson)))
}

/// POST /api/progress
pub async fn post_progress(
    State(state): State<AppState>,
    Json(payload): Json<ProgressUpdate>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query(
        "INSERT INTO user_progress (user_name, lesson_id, accuracy, score) VALUES (?, ?, ?, ?)"
    )
    .bind(&payload.user_name)
    .bind(payload.lesson_id)
    .bind(payload.accuracy)
    .bind(payload.score)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

/// GET /api/articles
pub async fn get_articles(State(state): State<AppState>) -> Result<Json<Vec<Article>>, StatusCode> {
    let articles = sqlx::query_as::<_, Article>(
        "SELECT id, title, content, difficulty FROM articles ORDER BY id",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(articles))
}

/// GET /api/articles/{id}
pub async fn get_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<Article>, StatusCode> {
    let article = sqlx::query_as::<_, Article>(
        "SELECT id, title, content, difficulty FROM articles WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(article))
}

/// POST /api/articles
pub async fn create_article(
    State(state): State<AppState>,
    Json(payload): Json<NewArticle>,
) -> Result<(StatusCode, Json<Article>), (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT INTO articles (title, content, difficulty) VALUES (?, ?, ?)"
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.difficulty)
    .execute(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let id: i64 = result.last_insert_id();
    let article = sqlx::query_as::<_, Article>(
        "SELECT id, title, content, difficulty FROM articles WHERE id = ?",
    )
    .bind(id as i32)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok((StatusCode::CREATED, Json(article)))
}

/// GET /api/wubi-roots
pub async fn get_wubi_roots(State(state): State<AppState>) -> Result<Json<Vec<WubiRoot>>, StatusCode> {
    match sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS character, code, position, description FROM wubi_roots ORDER BY position",
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(roots) => Ok(Json(roots)),
        Err(e) => {
            eprintln!("Error fetching wubi roots: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /api/wubi-roots/{id}
pub async fn get_wubi_root(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<WubiRoot>, StatusCode> {
    let root = sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS character, code, position, description FROM wubi_roots WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(root))
}

/// POST /api/wubi-roots
pub async fn create_wubi_root(
    State(state): State<AppState>,
    Json(payload): Json<NewWubiRoot>,
) -> Result<(StatusCode, Json<WubiRoot>), (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT INTO wubi_roots (character_val, code, position, description) VALUES (?, ?, ?, ?)"
    )
    .bind(&payload.character)
    .bind(&payload.code)
    .bind(&payload.position)
    .bind(&payload.description)
    .execute(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let id: i64 = result.last_insert_id();
    let root = sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS character, code, position, description FROM wubi_roots WHERE id = ?",
    )
    .bind(id as i32)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok((StatusCode::CREATED, Json(root)))
}

/// Search for wubi root by character
pub async fn search_wubi_root(
    Path(character): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Option<WubiRoot>>, StatusCode> {
    let root = sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS character, code, position, description FROM wubi_roots WHERE character_val = ?",
    )
    .bind(character)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(root))
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct WubiCharacter {
    pub character: String,
    pub wubi_code: String,
}

pub async fn get_wubi_code(
    Path(character): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<WubiCharacter>, StatusCode> {
    if character.chars().count() != 1 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Query wubi code from database
    let result = sqlx::query_as::<_, WubiCharacter>(
        "SELECT character_val AS character, wubi_code FROM wubi_characters WHERE character_val = ?"
    )
    .bind(&character)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(result))
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

/// Login user.
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Find user by username
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match user {
        Some(user) => {
            // Verify password
            let argon2 = Argon2::default();
            let password_hash = PasswordHash::new(&user.password_hash)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            if argon2.verify_password(payload.password.as_bytes(), &password_hash)
                .is_err() {
                return Err(StatusCode::UNAUTHORIZED);
            }
            
            // Generate token
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

/// Register user.
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Check if username already exists
    let existing_user = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT id FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }
    
    // Hash password
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();
    
    // Insert user into database
    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)"
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let user_id = result.last_insert_id() as i32;
    
    // Get the created user
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Generate token
    let token = generate_token(user.id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(LoginResponse {
        access_token: token,
        user,
    }))
}

/// Auth middleware to protect routes.
pub async fn auth_middleware(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = auth.token();
    
    // Validate token
    let claims = validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user_id = claims.sub.parse::<i32>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Verify user exists
    let user_exists = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT id FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match user_exists {
        Some(_) => {
            // Add user_id to request extensions
            let mut req = req;
            req.extensions_mut().insert(user_id);
            Ok(next.run(req).await)
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
