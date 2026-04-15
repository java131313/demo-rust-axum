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

/// Initialize the database schema and insert starter lessons.
pub async fn init_db(pool: &MySqlPool) -> Result<(), sqlx::Error> {
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
