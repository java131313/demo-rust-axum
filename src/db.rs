use async_trait::async_trait;
use sqlx::{MySql, Postgres, Pool};
use crate::config::{User, Lesson, Article, WubiRoot, WubiCharacter};

/// 数据库池的枚举类型
#[derive(Clone)]
pub enum DbPool {
    MySql(Pool<MySql>),
    Postgres(Pool<Postgres>),
}

/// 数据库操作trait
#[async_trait]
pub trait Database: Send + Sync {
    /// 初始化数据库表结构
    async fn init_db(&self) -> Result<(), String>;
    
    /// 获取五笔编码
    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String>;
    
    /// 获取所有五笔字符
    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String>;
    
    /// 获取课程列表
    async fn get_lessons(&self) -> Result<Vec<Lesson>, String>;
    
    /// 根据ID获取课程
    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String>;
    
    /// 创建课程
    async fn create_lesson(&self, character: &str, code: &str, description: &str) -> Result<Lesson, String>;
    
    /// 获取文章列表
    async fn get_articles(&self) -> Result<Vec<Article>, String>;
    
    /// 根据ID获取文章
    async fn get_article_by_id(&self, id: i32) -> Result<Article, String>;
    
    /// 创建文章
    async fn create_article(&self, title: &str, content: &str, difficulty: &str) -> Result<Article, String>;
    
    /// 获取字根列表
    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String>;
    
    /// 根据ID获取字根
    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String>;
    
    /// 创建字根
    async fn create_wubi_root(&self, character: &str, code: &str, position: &str, description: &str) -> Result<WubiRoot, String>;
    
    /// 根据字符搜索字根
    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String>;
    
    /// 根据用户名获取用户
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String>;
    
    /// 根据ID获取用户
    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String>;
    
    /// 创建用户
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, String>;
    
    /// 保存用户进度
    async fn save_progress(&self, user_name: &str, lesson_id: i32, accuracy: f32, score: i32) -> Result<(), String>;
}

/// MySQL数据库实现
pub struct MySqlDatabase {
    pool: Pool<MySql>,
}

impl MySqlDatabase {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Database for MySqlDatabase {
    async fn init_db(&self) -> Result<(), String> {
        // 创建表结构
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INT AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(64) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lessons (
                id INT AUTO_INCREMENT PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                description TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS articles (
                id INT AUTO_INCREMENT PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wubi_characters (
                id INT AUTO_INCREMENT PRIMARY KEY,
                character_val VARCHAR(4) NOT NULL UNIQUE,
                wubi_code VARCHAR(8) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wubi_roots (
                id INT AUTO_INCREMENT PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                position VARCHAR(64) NOT NULL,
                description TEXT
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

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
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 插入示例数据
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lessons")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        if count == 0 {
            let lessons = [
                ("人", "W", "单人旁，常用字根"),
                ("口", "K", "口字旁，常用字根"),
                ("日", "J", "日字旁，常用字根"),
            ];
            
            for (char, code, desc) in lessons {
                sqlx::query(
                    "INSERT INTO lessons (character_val, code, description) VALUES (?, ?, ?)"
                )
                .bind(char)
                .bind(code)
                .bind(desc)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        sqlx::query_as::<_, (String, String)>(
            "SELECT character_val, wubi_code FROM wubi_characters WHERE character_val = ?"
        )
        .bind(character)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Character not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(character, wubi_code)| WubiCharacter { character, wubi_code })
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        sqlx::query_as::<_, (String, String)>(
            "SELECT character_val, wubi_code FROM wubi_characters ORDER BY character_val"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(character, wubi_code)| WubiCharacter {
            character, wubi_code
        }).collect())
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, character_val, code, description FROM lessons ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, code, description)| Lesson {
            id, character, code, description
        }).collect())
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, character_val, code, description FROM lessons WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Lesson not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, code, description)| Lesson {
            id, character, code, description
        })
    }

    async fn create_lesson(&self, character: &str, code: &str, description: &str) -> Result<Lesson, String> {
        let result = sqlx::query(
            "INSERT INTO lessons (character_val, code, description) VALUES (?, ?, ?)"
        )
        .bind(character)
        .bind(code)
        .bind(description)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        self.get_lesson_by_id(id).await
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM articles ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, title, content, difficulty)| Article {
            id, title, content, difficulty
        }).collect())
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM articles WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Article not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, title, content, difficulty)| Article {
            id, title, content, difficulty
        })
    }

    async fn create_article(&self, title: &str, content: &str, difficulty: &str) -> Result<Article, String> {
        let result = sqlx::query(
            "INSERT INTO articles (title, content, difficulty) VALUES (?, ?, ?)"
        )
        .bind(title)
        .bind(content)
        .bind(difficulty)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        self.get_article_by_id(id).await
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots ORDER BY position"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        }).collect())
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Wubi root not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        })
    }

    async fn create_wubi_root(&self, character: &str, code: &str, position: &str, description: &str) -> Result<WubiRoot, String> {
        let result = sqlx::query(
            "INSERT INTO wubi_roots (character_val, code, position, description) VALUES (?, ?, ?, ?)"
        )
        .bind(character)
        .bind(code)
        .bind(position)
        .bind(description)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        self.get_wubi_root_by_id(id).await
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots WHERE character_val = ?"
        )
        .bind(character)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        }))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.to_rfc3339()
        }))
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.to_rfc3339()
        }))
    }

    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, String> {
        let result = sqlx::query(
            "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        self.get_user_by_id(id).await
            .map(|opt| opt.expect("User should exist after creation"))
    }

    async fn save_progress(&self, user_name: &str, lesson_id: i32, accuracy: f32, score: i32) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO user_progress (user_name, lesson_id, accuracy, score) VALUES (?, ?, ?, ?)"
        )
        .bind(user_name)
        .bind(lesson_id)
        .bind(accuracy)
        .bind(score)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

/// PostgreSQL数据库实现
pub struct PostgresDatabase {
    pool: Pool<Postgres>,
}

impl PostgresDatabase {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Database for PostgresDatabase {
    async fn init_db(&self) -> Result<(), String> {
        // 创建表结构
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(64) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lessons (
                id SERIAL PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                description TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS articles (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wubi_characters (
                id SERIAL PRIMARY KEY,
                character_val VARCHAR(4) NOT NULL UNIQUE,
                wubi_code VARCHAR(8) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wubi_roots (
                id SERIAL PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                position VARCHAR(64) NOT NULL,
                description TEXT
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_progress (
                id SERIAL PRIMARY KEY,
                user_name VARCHAR(64) NOT NULL,
                lesson_id INT NOT NULL,
                accuracy FLOAT NOT NULL,
                score INT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 插入示例数据
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lessons")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        if count == 0 {
            let lessons = [
                ("人", "W", "单人旁，常用字根"),
                ("口", "K", "口字旁，常用字根"),
                ("日", "J", "日字旁，常用字根"),
            ];
            
            for (char, code, desc) in lessons {
                sqlx::query(
                    "INSERT INTO lessons (character_val, code, description) VALUES ($1, $2, $3)"
                )
                .bind(char)
                .bind(code)
                .bind(desc)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        sqlx::query_as::<_, (String, String)>(
            "SELECT character_val, wubi_code FROM wubi_characters WHERE character_val = $1"
        )
        .bind(character)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Character not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(character, wubi_code)| WubiCharacter { character, wubi_code })
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        sqlx::query_as::<_, (String, String)>(
            "SELECT character_val, wubi_code FROM wubi_characters ORDER BY character_val"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(character, wubi_code)| WubiCharacter {
            character, wubi_code
        }).collect())
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, character_val, code, description FROM lessons ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, code, description)| Lesson {
            id, character, code, description
        }).collect())
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, character_val, code, description FROM lessons WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Lesson not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, code, description)| Lesson {
            id, character, code, description
        })
    }

    async fn create_lesson(&self, character: &str, code: &str, description: &str) -> Result<Lesson, String> {
        let lesson = sqlx::query_as::<_, (i32, String, String, String)>(
            "INSERT INTO lessons (character_val, code, description) VALUES ($1, $2, $3) RETURNING id, character_val, code, description"
        )
        .bind(character)
        .bind(code)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, character, code, description)| Lesson {
            id, character, code, description
        })
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM articles ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, title, content, difficulty)| Article {
            id, title, content, difficulty
        }).collect())
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM articles WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Article not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, title, content, difficulty)| Article {
            id, title, content, difficulty
        })
    }

    async fn create_article(&self, title: &str, content: &str, difficulty: &str) -> Result<Article, String> {
        let article = sqlx::query_as::<_, (i32, String, String, String)>(
            "INSERT INTO articles (title, content, difficulty) VALUES ($1, $2, $3) RETURNING id, title, content, difficulty"
        )
        .bind(title)
        .bind(content)
        .bind(difficulty)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, title, content, difficulty)| Article {
            id, title, content, difficulty
        })
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots ORDER BY position"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        }).collect())
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Wubi root not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        })
    }

    async fn create_wubi_root(&self, character: &str, code: &str, position: &str, description: &str) -> Result<WubiRoot, String> {
        let root = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "INSERT INTO wubi_roots (character_val, code, position, description) VALUES ($1, $2, $3, $4) RETURNING id, character_val, code, position, description"
        )
        .bind(character)
        .bind(code)
        .bind(position)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        })
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots WHERE character_val = $1"
        )
        .bind(character)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        }))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.to_rfc3339()
        }))
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.to_rfc3339()
        }))
    }

    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, String> {
        let user = sqlx::query_as::<_, (i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email, password_hash, created_at"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.to_rfc3339()
        })
    }

    async fn save_progress(&self, user_name: &str, lesson_id: i32, accuracy: f32, score: i32) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO user_progress (user_name, lesson_id, accuracy, score) VALUES ($1, $2, $3, $4)"
        )
        .bind(user_name)
        .bind(lesson_id)
        .bind(accuracy)
        .bind(score)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}
