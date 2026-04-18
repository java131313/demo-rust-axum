use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Generic database pool trait
#[async_trait]
pub trait DatabasePool: Send + Sync {
    type Pool;
    
    fn get_pool(&self) -> &Self::Pool;
    fn get_db_type(&self) -> &str;
}

/// Generic user model that works with both databases
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
}

/// Generic lesson model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbLesson {
    pub id: i32,
    pub character: String,
    pub code: String,
    pub description: String,
}

/// Generic article model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbArticle {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub difficulty: String,
}

/// Generic wubi root model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbWubiRoot {
    pub id: i32,
    pub character: String,
    pub code: String,
    pub position: String,
    pub description: String,
}

/// Generic wubi character model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbWubiCharacter {
    pub character: String,
    pub wubi_code: String,
}

/// Generic database operations trait
#[async_trait]
pub trait DatabaseOps: Send + Sync {
    /// Initialize database schema
    async fn init_db(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get all lessons
    async fn get_lessons(&self) -> Result<Vec<DbLesson>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get lesson by ID
    async fn get_lesson_by_id(&self, id: i32) -> Result<DbLesson, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Create a new lesson
    async fn create_lesson(&self, character: &str, code: &str, description: &str) -> Result<DbLesson, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get all articles
    async fn get_articles(&self) -> Result<Vec<DbArticle>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get article by ID
    async fn get_article_by_id(&self, id: i32) -> Result<DbArticle, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Create a new article
    async fn create_article(&self, title: &str, content: &str, difficulty: &str) -> Result<DbArticle, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get all wubi roots
    async fn get_wubi_roots(&self) -> Result<Vec<DbWubiRoot>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get wubi root by ID
    async fn get_wubi_root_by_id(&self, id: i32) -> Result<DbWubiRoot, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Create a new wubi root
    async fn create_wubi_root(&self, character: &str, code: &str, position: &str, description: &str) -> Result<DbWubiRoot, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Search wubi root by character
    async fn search_wubi_root(&self, character: &str) -> Result<Option<DbWubiRoot>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get wubi code for a character
    async fn get_wubi_code(&self, character: &str) -> Result<DbWubiCharacter, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get user by username
    async fn get_user_by_username(&self, username: &str) -> Result<Option<DbUser>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get user by ID
    async fn get_user_by_id(&self, id: i32) -> Result<Option<DbUser>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Create a new user
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<DbUser, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Save user progress
    async fn save_progress(&self, user_name: &str, lesson_id: i32, accuracy: f32, score: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
