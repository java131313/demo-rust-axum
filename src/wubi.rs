use crate::db::Database;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::TypedHeader;
use axum_extra::headers::authorization::{Authorization, Bearer};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path as StdPath;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Database>,
    /// SQLite连接池：数据库管理(WS)功能需要；非SQLite部署为None
    pub sqlite: Option<sqlx::SqlitePool>,
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
        Ok(content) => match serde_json::from_str::<Vec<WubiDictEntry>>(&content) {
            Ok(entries) => {
                println!("Loaded {} entries from wubi dictionary", entries.len());
                entries
            }
            Err(e) => {
                eprintln!("Failed to parse wubi dictionary JSON: {:?}", e);
                Vec::new()
            }
        },
        Err(e) => {
            eprintln!("Failed to read wubi dictionary file: {:?}", e);
            Vec::new()
        }
    }
}

/// Input payload for user login.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Input payload for user registration.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Output payload for user login.
#[derive(Debug, Serialize, utoipa::ToSchema)]
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
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct NewLesson {
    pub character: String,
    pub code: String,
    pub description: String,
}

/// Input payload for recording progress.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProgressUpdate {
    pub user_name: String,
    pub lesson_id: i32,
    pub accuracy: f32,
    pub score: i32,
}

/// Default MySQL connection string for local development.
pub fn default_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:sdsSDG123*^DD@127.0.0.1:3306/wubi".to_string())
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
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct NewArticle {
    pub title: String,
    pub content: String,
    pub difficulty: String,
    /// 语言: zh(默认)/ja/tw
    #[serde(default)]
    pub lang: Option<String>,
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
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct NewWubiRoot {
    pub character: String,
    pub code: String,
    pub position: String,
    pub description: String,
}

/// 健康检查：检测数据库与Redis缓存状态
#[utoipa::path(get, path = "/api/health", tag = "system", responses((status = 200, description = "服务健康状态")))]
pub async fn health(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let db_ok = state.db.health_check().await.is_ok();
    Json(serde_json::json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "database": if db_ok { "up" } else { "down" },
    }))
}

/// GET /api/lessons
#[utoipa::path(get, path = "/api/lessons", tag = "lessons", responses((status = 200, description = "课程列表", body = [crate::config::Lesson])))]
pub async fn get_lessons(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::Lesson>>, StatusCode> {
    let lessons = state
        .db
        .get_lessons()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(lessons))
}

#[utoipa::path(get, path = "/api/lessons/{id}", tag = "lessons", params(("id" = i32, Path, description = "课程ID")), responses((status = 200, description = "课程详情", body = crate::config::Lesson), (status = 404, description = "不存在")))]
pub async fn get_lesson(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::Lesson>, StatusCode> {
    let lesson = state
        .db
        .get_lesson_by_id(id)
        .await
        .map_err(|err| match err.as_str() {
            "Lesson not found" => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(lesson))
}

#[utoipa::path(post, path = "/api/lessons", tag = "lessons", request_body = NewLesson, responses((status = 201, description = "创建成功", body = crate::config::Lesson)))]
pub async fn create_lesson(
    State(state): State<AppState>,
    Json(payload): Json<NewLesson>,
) -> Result<(StatusCode, Json<crate::config::Lesson>), (StatusCode, String)> {
    let lesson = state
        .db
        .create_lesson(&payload.character, &payload.code, &payload.description)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    Ok((StatusCode::CREATED, Json(lesson)))
}

#[utoipa::path(post, path = "/api/progress", tag = "progress", request_body = ProgressUpdate, responses((status = 201, description = "进度已保存")))]
pub async fn post_progress(
    State(state): State<AppState>,
    Json(payload): Json<ProgressUpdate>,
) -> Result<StatusCode, StatusCode> {
    state
        .db
        .save_progress(
            &payload.user_name,
            payload.lesson_id,
            payload.accuracy,
            payload.score,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(get, path = "/api/articles", tag = "articles", responses((status = 200, description = "系统文章列表", body = [crate::config::Article])))]
pub async fn get_articles(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::Article>>, StatusCode> {
    let articles = state
        .db
        .get_articles()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(articles))
}

#[utoipa::path(get, path = "/api/articles/{id}", tag = "articles", params(("id" = i32, Path, description = "文章ID")), responses((status = 200, description = "文章详情", body = crate::config::Article), (status = 404, description = "不存在")))]
pub async fn get_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::Article>, StatusCode> {
    let article = state
        .db
        .get_article_by_id(id)
        .await
        .map_err(|err| match err.as_str() {
            "Article not found" => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(article))
}

#[utoipa::path(post, path = "/api/articles", tag = "articles", request_body = NewArticle, responses((status = 201, description = "创建成功", body = crate::config::Article)))]
pub async fn create_article(
    State(state): State<AppState>,
    Json(payload): Json<NewArticle>,
) -> Result<(StatusCode, Json<crate::config::Article>), (StatusCode, String)> {
    let article = state
        .db
        .create_article(&payload.title, &payload.content, &payload.difficulty)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    Ok((StatusCode::CREATED, Json(article)))
}

#[utoipa::path(put, path = "/api/articles/{id}", tag = "articles", params(("id" = i32, Path, description = "文章ID")), request_body = NewArticle, responses((status = 200, description = "更新成功", body = crate::config::Article), (status = 404, description = "不存在")))]
pub async fn update_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<NewArticle>,
) -> Result<Json<crate::config::Article>, (StatusCode, String)> {
    let article = state
        .db
        .update_article(id, &payload.title, &payload.content, &payload.difficulty)
        .await
        .map_err(|err| match err.as_str() {
            "Article not found" => (StatusCode::NOT_FOUND, err),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err),
        })?;
    Ok(Json(article))
}

#[utoipa::path(delete, path = "/api/articles/{id}", tag = "articles", params(("id" = i32, Path, description = "文章ID")), responses((status = 204, description = "已删除"), (status = 404, description = "不存在")))]
pub async fn delete_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .db
        .delete_article(id)
        .await
        .map_err(|err| match err.as_str() {
            "Article not found" => (StatusCode::NOT_FOUND, err),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/custom-articles", tag = "articles", params(("lang" = Option<String>, Query, description = "语言: zh(默认)/ja/tw/en")), responses((status = 200, description = "自定义文章列表", body = [crate::config::Article])))]
pub async fn get_custom_articles(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::config::Article>>, StatusCode> {
    let lang = params.get("lang").map(|s| s.as_str()).unwrap_or("zh");
    let articles = state
        .db
        .get_custom_articles(lang)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(articles))
}

#[utoipa::path(post, path = "/api/custom-articles", tag = "articles", request_body = NewArticle, responses((status = 201, description = "创建成功", body = crate::config::Article)))]
pub async fn create_custom_article(
    State(state): State<AppState>,
    Json(payload): Json<NewArticle>,
) -> Result<(StatusCode, Json<crate::config::Article>), (StatusCode, String)> {
    let lang = payload.lang.clone().unwrap_or_else(|| "zh".to_string());
    let article = state
        .db
        .create_custom_article(&payload.title, &payload.content, &payload.difficulty, &lang)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    Ok((StatusCode::CREATED, Json(article)))
}

#[utoipa::path(delete, path = "/api/custom-articles/{id}", tag = "articles", params(("id" = i32, Path, description = "文章ID")), responses((status = 204, description = "已删除"), (status = 404, description = "不存在")))]
pub async fn delete_custom_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .db
        .delete_custom_article(id)
        .await
        .map_err(|err| match err.as_str() {
            "Custom article not found" => (StatusCode::NOT_FOUND, err),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(put, path = "/api/custom-articles/{id}", tag = "articles", params(("id" = i32, Path, description = "文章ID")), request_body = NewArticle, responses((status = 200, description = "更新成功", body = crate::config::Article), (status = 404, description = "不存在")))]
pub async fn update_custom_article(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<NewArticle>,
) -> Result<Json<crate::config::Article>, (StatusCode, String)> {
    let article = state
        .db
        .update_custom_article(id, &payload.title, &payload.content, &payload.difficulty)
        .await
        .map_err(|err| match err.as_str() {
            "Custom article not found" => (StatusCode::NOT_FOUND, err),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err),
        })?;
    Ok(Json(article))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWubiCode {
    pub character: String,
    pub code: String,
}

#[utoipa::path(put, path = "/api/wubi-code", tag = "wubi", request_body = UpdateWubiCode, responses((status = 200, description = "更新成功", body = crate::config::WubiCharacter)))]
pub async fn update_wubi_code_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdateWubiCode>,
) -> Result<Json<crate::config::WubiCharacter>, StatusCode> {
    let result = state
        .db
        .update_wubi_code(&payload.character, &payload.code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(result))
}

#[utoipa::path(get, path = "/api/wubi-roots", tag = "wubi", responses((status = 200, description = "字根列表", body = [crate::config::WubiRoot])))]
pub async fn get_wubi_roots(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::WubiRoot>>, StatusCode> {
    let roots = state.db.get_wubi_roots().await.map_err(|e| {
        eprintln!("Error fetching wubi roots: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(roots))
}

#[utoipa::path(get, path = "/api/wubi-roots/{id}", tag = "wubi", params(("id" = i32, Path, description = "字根ID")), responses((status = 200, description = "字根详情", body = crate::config::WubiRoot), (status = 404, description = "不存在")))]
pub async fn get_wubi_root(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::WubiRoot>, StatusCode> {
    let root = state
        .db
        .get_wubi_root_by_id(id)
        .await
        .map_err(|err| match err.as_str() {
            "Wubi root not found" => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(root))
}

#[utoipa::path(post, path = "/api/wubi-roots", tag = "wubi", request_body = NewWubiRoot, responses((status = 201, description = "创建成功", body = crate::config::WubiRoot)))]
pub async fn create_wubi_root(
    State(state): State<AppState>,
    Json(payload): Json<NewWubiRoot>,
) -> Result<(StatusCode, Json<crate::config::WubiRoot>), (StatusCode, String)> {
    let root = state
        .db
        .create_wubi_root(
            &payload.character,
            &payload.code,
            &payload.position,
            &payload.description,
        )
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    Ok((StatusCode::CREATED, Json(root)))
}

#[utoipa::path(get, path = "/api/search-wubi-root/{character}", tag = "wubi", params(("character" = String, Path, description = "汉字")), responses((status = 200, description = "查询结果", body = Option<crate::config::WubiRoot>)))]
pub async fn search_wubi_root(
    Path(character): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Option<crate::config::WubiRoot>>, StatusCode> {
    let root = state
        .db
        .search_wubi_root(&character)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(root))
}

#[utoipa::path(get, path = "/api/wubi/{character}", tag = "wubi", params(("character" = String, Path, description = "单个汉字")), responses((status = 200, description = "五笔编码", body = crate::config::WubiCharacter), (status = 400, description = "参数错误"), (status = 404, description = "未找到")))]
pub async fn get_wubi_code(
    Path(character): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::WubiCharacter>, StatusCode> {
    if character.chars().count() != 1 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 先尝试从数据库查询
    match state.db.get_wubi_code(&character).await {
        Ok(result) => return Ok(Json(result)),
        Err(err) => {
            tracing::warn!("get_wubi_code db error for '{}': {}", character, err);
            // 数据库查询失败时，回退到内存字典
        }
    }

    // 回退到内存字典
    match crate::wubi_dict::get_wubi_code(&character) {
        Some(code) => {
            let pinyin = crate::wubi_dict::get_pinyin(&character).unwrap_or("");
            Ok(Json(crate::config::WubiCharacter {
                id: 0,
                character: character.to_string(),
                simple_code: code.to_string(),
                full_code: code.to_string(),
                pinyin: pinyin.to_string(),
                remark: String::new(),
            }))
        }
        None => {
            tracing::error!(
                "get_wubi_code: character '{}' not found in db or dict",
                character
            );
            Err(StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(get, path = "/api/key-radicals", tag = "wubi", responses((status = 200, description = "键位字根列表", body = [crate::config::KeyRadical])))]
pub async fn get_key_radicals(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::KeyRadical>>, StatusCode> {
    let radicals = state
        .db
        .get_key_radicals()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(radicals))
}

#[utoipa::path(get, path = "/api/key-radicals/{key}", tag = "wubi", params(("key" = String, Path, description = "键位字符")), responses((status = 200, description = "键位字根", body = crate::config::KeyRadical), (status = 404, description = "不存在")))]
pub async fn get_key_radical_by_key(
    Path(key_char): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::config::KeyRadical>, StatusCode> {
    let result = state
        .db
        .get_key_radical_by_key(&key_char)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Some(radical) => Ok(Json(radical)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[utoipa::path(get, path = "/api/english-texts", tag = "texts", responses((status = 200, description = "英语练习文本", body = [crate::config::EnglishText])))]
pub async fn get_english_texts(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::EnglishText>>, StatusCode> {
    let texts = state
        .db
        .get_english_texts()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(texts))
}

/// GET /api/japanese-texts
#[utoipa::path(get, path = "/api/japanese-texts", tag = "texts", responses((status = 200, description = "日语练习文本", body = [crate::config::JapaneseText])))]
pub async fn get_japanese_texts(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::JapaneseText>>, StatusCode> {
    let texts = state
        .db
        .get_japanese_texts()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(texts))
}

/// GET /api/japanese-keyboards
#[utoipa::path(get, path = "/api/japanese-keyboards", tag = "texts", responses((status = 200, description = "日语键盘布局", body = [crate::config::JapaneseKeyboard])))]
pub async fn get_japanese_keyboards(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::JapaneseKeyboard>>, StatusCode> {
    let keyboards = state
        .db
        .get_japanese_keyboards()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(keyboards))
}

/// GET /api/japanese-characters
#[utoipa::path(get, path = "/api/japanese-characters", tag = "texts", responses((status = 200, description = "日语字符", body = [crate::config::JapaneseCharacter])))]
pub async fn get_japanese_characters(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::JapaneseCharacter>>, StatusCode> {
    let characters = state
        .db
        .get_japanese_characters()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(characters))
}

/// GET /api/traditional-chinese-texts
#[utoipa::path(get, path = "/api/traditional-chinese-texts", tag = "texts", responses((status = 200, description = "繁体中文练习文本", body = [crate::config::TraditionalChineseText])))]
pub async fn get_traditional_chinese_texts(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::TraditionalChineseText>>, StatusCode> {
    let texts = state
        .db
        .get_traditional_chinese_texts()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(texts))
}

/// GET /api/bopomofo-keyboards
#[utoipa::path(get, path = "/api/bopomofo-keyboards", tag = "texts", responses((status = 200, description = "注音键盘布局", body = [crate::config::BopomofoKeyboard])))]
pub async fn get_bopomofo_keyboards(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::BopomofoKeyboard>>, StatusCode> {
    let keyboards = state
        .db
        .get_bopomofo_keyboards()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(keyboards))
}

/// GET /api/bopomofo-characters
#[utoipa::path(get, path = "/api/bopomofo-characters", tag = "texts", responses((status = 200, description = "注音字符", body = [crate::config::BopomofoCharacter])))]
pub async fn get_bopomofo_characters(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::BopomofoCharacter>>, StatusCode> {
    let characters = state
        .db
        .get_bopomofo_characters()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(characters))
}

/// GET /api/japanese-gojuon
#[utoipa::path(get, path = "/api/japanese-gojuon", tag = "texts", responses((status = 200, description = "五十音表", body = [crate::config::JapaneseGojuon])))]
pub async fn get_japanese_gojuon(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::JapaneseGojuon>>, StatusCode> {
    let gojuon = state
        .db
        .get_japanese_gojuon()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(gojuon))
}

/// GET /api/japanese-kanji
#[utoipa::path(get, path = "/api/japanese-kanji", tag = "texts", responses((status = 200, description = "日语汉字列表", body = [crate::config::JapaneseKanji])))]
pub async fn get_japanese_kanji(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::config::JapaneseKanji>>, StatusCode> {
    let kanji = state
        .db
        .get_japanese_kanji()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(kanji))
}

/// GET /api/japanese-kanji/{kanji}
#[utoipa::path(get, path = "/api/japanese-kanji/{kanji}", tag = "texts", params(("kanji" = String, Path, description = "汉字")), responses((status = 200, description = "查询结果", body = Option<crate::config::JapaneseKanji>)))]
pub async fn get_japanese_kanji_by_char(
    Path(kanji): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Option<crate::config::JapaneseKanji>>, StatusCode> {
    let result = state
        .db
        .get_japanese_kanji_by_char(&kanji)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

#[utoipa::path(post, path = "/api/login", tag = "auth", request_body = LoginRequest, responses((status = 200, description = "登录成功", body = LoginResponse), (status = 401, description = "认证失败")))]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user_opt = state
        .db
        .get_user_by_username(&payload.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user_opt {
        Some(user) => {
            let argon2 = Argon2::default();
            let password_hash = PasswordHash::new(&user.password_hash)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if argon2
                .verify_password(payload.password.as_bytes(), &password_hash)
                .is_err()
            {
                return Err(StatusCode::UNAUTHORIZED);
            }

            let token = generate_token(user.id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(LoginResponse {
                access_token: token,
                user,
            }))
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Logout user (client-side logout by removing token).
#[utoipa::path(post, path = "/api/logout", tag = "auth", responses((status = 200, description = "已退出")))]
pub async fn logout() -> StatusCode {
    // Logout is handled client-side by removing the token
    StatusCode::OK
}

#[utoipa::path(post, path = "/api/register", tag = "auth", request_body = RegisterRequest, responses((status = 200, description = "注册成功", body = LoginResponse), (status = 409, description = "用户已存在")))]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let existing_user = state
        .db
        .get_user_by_username(&payload.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let user = state
        .db
        .create_user(&payload.username, &payload.email, &password_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = generate_token(user.id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

    let claims = validate_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id = claims
        .sub
        .parse::<i32>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_exists = state
        .db
        .get_user_by_id(user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user_exists {
        Some(_) => {
            let mut req = req;
            req.extensions_mut().insert(user_id);
            Ok(next.run(req).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
