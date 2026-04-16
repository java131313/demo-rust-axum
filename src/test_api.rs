use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

// Simplified Article struct for testing
#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleArticle {
    pub id: i32,  // Changed from u32 to i32 to match MySQL INT
    pub title: String,
    pub content: String,
    pub difficulty: Option<String>,  // Changed to Option to handle potential null values
}

/// GET /api/articles - Simplified version for testing
pub async fn get_articles_simple(State(state): State<crate::wubi::AppState>) -> Result<Json<Vec<SimpleArticle>>, StatusCode> {
    println!("get_articles_simple called"); // Debug print
    
    match sqlx::query_as::<_, SimpleArticle>(
        "SELECT id, title, content, difficulty FROM articles ORDER BY id"
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(articles) => {
            println!("Fetched {} articles", articles.len()); // Debug print
            Ok(Json(articles))
        },
        Err(e) => {
            eprintln!("Error fetching articles: {:?}", e); // Error print
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}