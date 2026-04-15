use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

/// Shared application state for Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
}

/// A Wubi tutorial lesson record.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lesson {
    pub id: u32,
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
    pub lesson_id: u32,
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
    pub id: u32,
    pub title: String,
    pub content: String,
    pub difficulty: String, // easy, medium, hard
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
    pub id: u32,
    pub character: String,
    pub code: String,
    pub position: String, // position in the keyboard
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
            difficulty ENUM('easy', 'medium', 'hard') DEFAULT 'medium'
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
            ("人", "WG", "练习“人”字的五笔编码。"),
            ("日", "KH", "练习“日”字的五笔编码。"),
            ("山", "FQ", "练习“山”字的五笔编码。"),
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
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM articles").fetch_one(pool).await?;
    if count == 0 {
        let rows = [
            ("练习文章一", "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。", "easy"),
            ("练习文章二", "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。", "medium"),
            ("练习文章三", "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。", "hard"),
        ];

        for (title, content, difficulty) in rows {
            sqlx::query(
                "INSERT INTO articles (title, content, difficulty) VALUES (?, ?, ?)"
            )
            .bind(title)
            .bind(content)
            .bind(difficulty)
            .execute(pool)
            .await?;
        }
    }

    // Insert sample wubi roots if table is empty
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wubi_roots").fetch_one(pool).await?;
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
            sqlx::query(
                "INSERT INTO wubi_roots (character_val, code, position, description) VALUES (?, ?, ?, ?)"
            )
            .bind(character)
            .bind(code)
            .bind(position)
            .bind(description)
            .execute(pool)
            .await?;
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
    Path(id): Path<u32>,
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

    let id = result.last_insert_id();
    let lesson = sqlx::query_as::<_, Lesson>(
        "SELECT id, character_val AS character, code, description FROM lessons WHERE id = ?",
    )
    .bind(id as u32)
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
    Path(id): Path<u32>,
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

    let id = result.last_insert_id();
    let article = sqlx::query_as::<_, Article>(
        "SELECT id, title, content, difficulty FROM articles WHERE id = ?",
    )
    .bind(id as u32)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok((StatusCode::CREATED, Json(article)))
}

/// GET /api/wubi-roots
pub async fn get_wubi_roots(State(state): State<AppState>) -> Result<Json<Vec<WubiRoot>>, StatusCode> {
    let roots = sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS character, code, position, description FROM wubi_roots ORDER BY position",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(roots))
}

/// GET /api/wubi-roots/{id}
pub async fn get_wubi_root(
    Path(id): Path<u32>,
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

    let id = result.last_insert_id();
    let root = sqlx::query_as::<_, WubiRoot>(
        "SELECT id, character_val AS character, code, position, description FROM wubi_roots WHERE id = ?",
    )
    .bind(id as u32)
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
