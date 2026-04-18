use async_trait::async_trait;
use sqlx::{MySql, Postgres, Pool};
use redis::AsyncCommands;
use mongodb::{Client, Database as MongoDb};
use mongodb::bson::{doc, Document};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument, FindOptions};
use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::config::{User, Lesson, Article, WubiRoot, WubiCharacter};

#[derive(Debug, Deserialize)]
struct WubiDictEntry {
    character: String,
    code: String,
}

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

/// Redis数据库实现
pub struct RedisDatabase {
    client: redis::Client,
}

impl RedisDatabase {
    pub fn new(url: &str) -> Result<Self, String> {
        let client = redis::Client::open(url)
            .map_err(|e| format!("Failed to create Redis client: {}", e))?;
        Ok(Self { client })
    }

    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, String> {
        self.client.get_multiplexed_async_connection().await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))
    }

    async fn import_wubi_dict(&self, conn: &mut redis::aio::MultiplexedConnection) -> Result<(), String> {
        let dict_path = "data/wubi_dict.json";
        
        if !Path::new(dict_path).exists() {
            return Err(format!("Wubi dictionary file not found: {}", dict_path));
        }
        
        let content = fs::read_to_string(dict_path)
            .map_err(|e| format!("Failed to read wubi dictionary file: {}", e))?;
        
        let entries: Vec<WubiDictEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse wubi dictionary JSON: {}", e))?;
        
        println!("Importing {} wubi dictionary entries to Redis...", entries.len());
        
        let batch_size = 100;
        for chunk in entries.chunks(batch_size) {
            let mut pipe = redis::pipe();
            for entry in chunk {
                pipe.set(&format!("wubi:char:{}", entry.character), &entry.code);
            }
            let _: () = pipe.query_async(conn).await
                .map_err(|e| format!("Failed to batch insert wubi characters: {}", e))?;
        }
        
        println!("Wubi dictionary import complete: {} entries", entries.len());
        Ok(())
    }

    async fn import_sample_data(&self, conn: &mut redis::aio::MultiplexedConnection) -> Result<(), String> {
        let _: () = conn.incr("wubi:id:lessons", 3).await
            .map_err(|e| format!("Failed to set lesson id counter: {}", e))?;
        
        let lessons = [
            ("1", "人", "WG", "练习人字的五笔编码。"),
            ("2", "日", "KH", "练习日字的五笔编码。"),
            ("3", "山", "FQ", "练习山字的五笔编码。"),
        ];
        
        for (id, char, code, desc) in lessons {
            let _: () = conn.hset_multiple(&format!("wubi:lesson:{}", id), &[
                ("id", id),
                ("character", char),
                ("code", code),
                ("description", desc),
            ]).await.map_err(|e| format!("Failed to insert lesson: {}", e))?;
            
            let _: () = conn.sadd("wubi:lessons", id).await
                .map_err(|e| format!("Failed to add lesson to set: {}", e))?;
        }
        
        let _: () = conn.incr("wubi:id:articles", 3).await
            .map_err(|e| format!("Failed to set article id counter: {}", e))?;
        
        let articles = [
            ("1", "练习文章一", "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。", "easy"),
            ("2", "练习文章二", "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。", "medium"),
            ("3", "练习文章三", "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。", "hard"),
        ];
        
        for (id, title, content, difficulty) in articles {
            let _: () = conn.hset_multiple(&format!("wubi:article:{}", id), &[
                ("id", id),
                ("title", title),
                ("content", content),
                ("difficulty", difficulty),
            ]).await.map_err(|e| format!("Failed to insert article: {}", e))?;
            
            let _: () = conn.sadd("wubi:articles", id).await
                .map_err(|e| format!("Failed to add article to set: {}", e))?;
        }
        
        let _: () = conn.incr("wubi:id:roots", 10).await
            .map_err(|e| format!("Failed to set root id counter: {}", e))?;
        
        let roots = [
            ("1", "一", "GGLL", "G区第一键", "横区起首字根"),
            ("2", "丿", "TTLL", "T区第一键", "撇区起首字根"),
            ("3", "丨", "HHLL", "H区第一键", "竖区起首字根"),
            ("4", "丶", "YYLL", "Y区第一键", "捺区起首字根"),
            ("5", "乙", "NNLL", "N区第一键", "折区起首字根"),
            ("6", "九", "VTNG", "V区第二键", "字根：乙"),
            ("7", "力", "LTNN", "L键", "字根：力"),
            ("8", "乃", "DETN", "N键", "字根：乃"),
            ("9", "刀", "VNTE", "V键", "字根：刀"),
            ("10", "卜", "HHYD", "H键", "字根：卜"),
        ];
        
        for (id, char, code, position, desc) in roots {
            let _: () = conn.hset_multiple(&format!("wubi:root:{}", id), &[
                ("id", id),
                ("character", char),
                ("code", code),
                ("position", position),
                ("description", desc),
            ]).await.map_err(|e| format!("Failed to insert root: {}", e))?;
            
            let _: () = conn.sadd("wubi:roots", id).await
                .map_err(|e| format!("Failed to add root to set: {}", e))?;
        }
        
        println!("Sample data imported to Redis");
        Ok(())
    }
}

#[async_trait]
impl Database for RedisDatabase {
    async fn init_db(&self) -> Result<(), String> {
        let mut conn = self.get_connection().await?;
        
        let _: () = conn.set("wubi:initialized", "true").await
            .map_err(|e| format!("Failed to initialize Redis: {}", e))?;
        
        let initialized: Option<String> = conn.get("wubi:data_imported").await
            .map_err(|e| format!("Failed to check import status: {}", e))?;
        
        if initialized.is_none() {
            self.import_wubi_dict(&mut conn).await?;
            self.import_sample_data(&mut conn).await?;
            
            let _: () = conn.set("wubi:data_imported", "true").await
                .map_err(|e| format!("Failed to set import flag: {}", e))?;
        }
        
        Ok(())
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        let mut conn = self.get_connection().await?;
        
        let code: Option<String> = conn.get(&format!("wubi:char:{}", character)).await
            .map_err(|e| format!("Failed to get wubi code: {}", e))?;
        
        match code {
            Some(wubi_code) => Ok(WubiCharacter {
                character: character.to_string(),
                wubi_code,
            }),
            None => Err("Character not found".to_string()),
        }
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        let mut conn = self.get_connection().await?;
        
        let keys: Vec<String> = conn.keys("wubi:char:*").await
            .map_err(|e| format!("Failed to get keys: {}", e))?;
        
        let mut characters = Vec::new();
        for key in keys {
            let wubi_code: Option<String> = conn.get(&key).await
                .map_err(|e| format!("Failed to get value: {}", e))?;
            
            if let Some(code) = wubi_code {
                let character = key.strip_prefix("wubi:char:").unwrap_or("").to_string();
                characters.push(WubiCharacter { character, wubi_code: code });
            }
        }
        
        characters.sort_by(|a, b| a.character.cmp(&b.character));
        Ok(characters)
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        let mut conn = self.get_connection().await?;
        
        let lesson_ids: Vec<String> = conn.smembers("wubi:lessons").await
            .map_err(|e| format!("Failed to get lesson ids: {}", e))?;
        
        let mut lessons = Vec::new();
        for id in lesson_ids {
            let character: String = conn.hget(&format!("wubi:lesson:{}", id), "character").await
                .map_err(|e| format!("Failed to get character: {}", e))?;
            let code: String = conn.hget(&format!("wubi:lesson:{}", id), "code").await
                .map_err(|e| format!("Failed to get code: {}", e))?;
            let description: String = conn.hget(&format!("wubi:lesson:{}", id), "description").await
                .map_err(|e| format!("Failed to get description: {}", e))?;
            
            lessons.push(Lesson {
                id: id.parse().unwrap_or(0),
                character,
                code,
                description,
            });
        }
        
        lessons.sort_by_key(|l| l.id);
        Ok(lessons)
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:lesson:{}", id);
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| format!("Failed to check lesson: {}", e))?;
        
        if !exists {
            return Err("Lesson not found".to_string());
        }
        
        let character: String = conn.hget(&key, "character").await
            .map_err(|e| format!("Failed to get character: {}", e))?;
        let code: String = conn.hget(&key, "code").await
            .map_err(|e| format!("Failed to get code: {}", e))?;
        let description: String = conn.hget(&key, "description").await
            .map_err(|e| format!("Failed to get description: {}", e))?;
        
        Ok(Lesson {
            id,
            character,
            code,
            description,
        })
    }

    async fn create_lesson(&self, character: &str, code: &str, description: &str) -> Result<Lesson, String> {
        let mut conn = self.get_connection().await?;
        
        let id: i32 = conn.incr("wubi:id:lessons", 1).await
            .map_err(|e| format!("Failed to generate id: {}", e))?;
        
        let key = format!("wubi:lesson:{}", id);
        let _: () = conn.hset_multiple(&key, &[
            ("id", id.to_string()),
            ("character", character.to_string()),
            ("code", code.to_string()),
            ("description", description.to_string()),
        ]).await.map_err(|e| format!("Failed to create lesson: {}", e))?;
        
        let _: () = conn.sadd("wubi:lessons", id.to_string()).await
            .map_err(|e| format!("Failed to add lesson to set: {}", e))?;
        
        Ok(Lesson {
            id,
            character: character.to_string(),
            code: code.to_string(),
            description: description.to_string(),
        })
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        let mut conn = self.get_connection().await?;
        
        let article_ids: Vec<String> = conn.smembers("wubi:articles").await
            .map_err(|e| format!("Failed to get article ids: {}", e))?;
        
        let mut articles = Vec::new();
        for id in article_ids {
            let title: String = conn.hget(&format!("wubi:article:{}", id), "title").await
                .map_err(|e| format!("Failed to get title: {}", e))?;
            let content: String = conn.hget(&format!("wubi:article:{}", id), "content").await
                .map_err(|e| format!("Failed to get content: {}", e))?;
            let difficulty: String = conn.hget(&format!("wubi:article:{}", id), "difficulty").await
                .map_err(|e| format!("Failed to get difficulty: {}", e))?;
            
            articles.push(Article {
                id: id.parse().unwrap_or(0),
                title,
                content,
                difficulty,
            });
        }
        
        articles.sort_by_key(|a| a.id);
        Ok(articles)
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:article:{}", id);
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| format!("Failed to check article: {}", e))?;
        
        if !exists {
            return Err("Article not found".to_string());
        }
        
        let title: String = conn.hget(&key, "title").await
            .map_err(|e| format!("Failed to get title: {}", e))?;
        let content: String = conn.hget(&key, "content").await
            .map_err(|e| format!("Failed to get content: {}", e))?;
        let difficulty: String = conn.hget(&key, "difficulty").await
            .map_err(|e| format!("Failed to get difficulty: {}", e))?;
        
        Ok(Article {
            id,
            title,
            content,
            difficulty,
        })
    }

    async fn create_article(&self, title: &str, content: &str, difficulty: &str) -> Result<Article, String> {
        let mut conn = self.get_connection().await?;
        
        let id: i32 = conn.incr("wubi:id:articles", 1).await
            .map_err(|e| format!("Failed to generate id: {}", e))?;
        
        let key = format!("wubi:article:{}", id);
        let _: () = conn.hset_multiple(&key, &[
            ("id", id.to_string()),
            ("title", title.to_string()),
            ("content", content.to_string()),
            ("difficulty", difficulty.to_string()),
        ]).await.map_err(|e| format!("Failed to create article: {}", e))?;
        
        let _: () = conn.sadd("wubi:articles", id.to_string()).await
            .map_err(|e| format!("Failed to add article to set: {}", e))?;
        
        Ok(Article {
            id,
            title: title.to_string(),
            content: content.to_string(),
            difficulty: difficulty.to_string(),
        })
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        let mut conn = self.get_connection().await?;
        
        let root_ids: Vec<String> = conn.smembers("wubi:roots").await
            .map_err(|e| format!("Failed to get root ids: {}", e))?;
        
        let mut roots = Vec::new();
        for id in root_ids {
            let character: String = conn.hget(&format!("wubi:root:{}", id), "character").await
                .map_err(|e| format!("Failed to get character: {}", e))?;
            let code: String = conn.hget(&format!("wubi:root:{}", id), "code").await
                .map_err(|e| format!("Failed to get code: {}", e))?;
            let position: String = conn.hget(&format!("wubi:root:{}", id), "position").await
                .map_err(|e| format!("Failed to get position: {}", e))?;
            let description: String = conn.hget(&format!("wubi:root:{}", id), "description").await
                .map_err(|e| format!("Failed to get description: {}", e))?;
            
            roots.push(WubiRoot {
                id: id.parse().unwrap_or(0),
                character,
                code,
                position,
                description,
            });
        }
        
        roots.sort_by_key(|r| r.position.clone());
        Ok(roots)
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:root:{}", id);
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| format!("Failed to check root: {}", e))?;
        
        if !exists {
            return Err("Wubi root not found".to_string());
        }
        
        let character: String = conn.hget(&key, "character").await
            .map_err(|e| format!("Failed to get character: {}", e))?;
        let code: String = conn.hget(&key, "code").await
            .map_err(|e| format!("Failed to get code: {}", e))?;
        let position: String = conn.hget(&key, "position").await
            .map_err(|e| format!("Failed to get position: {}", e))?;
        let description: String = conn.hget(&key, "description").await
            .map_err(|e| format!("Failed to get description: {}", e))?;
        
        Ok(WubiRoot {
            id,
            character,
            code,
            position,
            description,
        })
    }

    async fn create_wubi_root(&self, character: &str, code: &str, position: &str, description: &str) -> Result<WubiRoot, String> {
        let mut conn = self.get_connection().await?;
        
        let id: i32 = conn.incr("wubi:id:roots", 1).await
            .map_err(|e| format!("Failed to generate id: {}", e))?;
        
        let key = format!("wubi:root:{}", id);
        let _: () = conn.hset_multiple(&key, &[
            ("id", id.to_string()),
            ("character", character.to_string()),
            ("code", code.to_string()),
            ("position", position.to_string()),
            ("description", description.to_string()),
        ]).await.map_err(|e| format!("Failed to create root: {}", e))?;
        
        let _: () = conn.sadd("wubi:roots", id.to_string()).await
            .map_err(|e| format!("Failed to add root to set: {}", e))?;
        
        Ok(WubiRoot {
            id,
            character: character.to_string(),
            code: code.to_string(),
            position: position.to_string(),
            description: description.to_string(),
        })
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        let mut conn = self.get_connection().await?;
        
        let root_ids: Vec<String> = conn.smembers("wubi:roots").await
            .map_err(|e| format!("Failed to get root ids: {}", e))?;
        
        for id in root_ids {
            let stored_char: String = conn.hget(&format!("wubi:root:{}", id), "character").await
                .map_err(|e| format!("Failed to get character: {}", e))?;
            
            if stored_char == character {
                let code: String = conn.hget(&format!("wubi:root:{}", id), "code").await
                    .map_err(|e| format!("Failed to get code: {}", e))?;
                let position: String = conn.hget(&format!("wubi:root:{}", id), "position").await
                    .map_err(|e| format!("Failed to get position: {}", e))?;
                let description: String = conn.hget(&format!("wubi:root:{}", id), "description").await
                    .map_err(|e| format!("Failed to get description: {}", e))?;
                
                return Ok(Some(WubiRoot {
                    id: id.parse().unwrap_or(0),
                    character: stored_char,
                    code,
                    position,
                    description,
                }));
            }
        }
        
        Ok(None)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:user:{}", username);
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| format!("Failed to check user: {}", e))?;
        
        if !exists {
            return Ok(None);
        }
        
        let id: String = conn.hget(&key, "id").await
            .map_err(|e| format!("Failed to get id: {}", e))?;
        let email: String = conn.hget(&key, "email").await
            .map_err(|e| format!("Failed to get email: {}", e))?;
        let password_hash: String = conn.hget(&key, "password_hash").await
            .map_err(|e| format!("Failed to get password_hash: {}", e))?;
        let created_at: String = conn.hget(&key, "created_at").await
            .map_err(|e| format!("Failed to get created_at: {}", e))?;
        
        Ok(Some(User {
            id: id.parse().unwrap_or(0),
            username: username.to_string(),
            email,
            password_hash,
            created_at,
        }))
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let mut conn = self.get_connection().await?;
        
        let username: Option<String> = conn.get(&format!("wubi:userid:{}:username", id)).await
            .map_err(|e| format!("Failed to get username mapping: {}", e))?;
        
        match username {
            Some(username) => self.get_user_by_username(&username).await,
            None => Ok(None),
        }
    }

    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, String> {
        let mut conn = self.get_connection().await?;
        
        let id: i32 = conn.incr("wubi:id:users", 1).await
            .map_err(|e| format!("Failed to generate id: {}", e))?;
        
        let now = chrono::Utc::now().to_rfc3339();
        let key = format!("wubi:user:{}", username);
        
        let _: () = conn.hset_multiple(&key, &[
            ("id", id.to_string()),
            ("username", username.to_string()),
            ("email", email.to_string()),
            ("password_hash", password_hash.to_string()),
            ("created_at", now.clone()),
        ]).await.map_err(|e| format!("Failed to create user: {}", e))?;
        
        let _: () = conn.set(&format!("wubi:userid:{}:username", id), username).await
            .map_err(|e| format!("Failed to create user id mapping: {}", e))?;
        
        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            created_at: now,
        })
    }

    async fn save_progress(&self, user_name: &str, lesson_id: i32, accuracy: f32, score: i32) -> Result<(), String> {
        let mut conn = self.get_connection().await?;
        
        let key = format!("wubi:progress:{}:{}", user_name, lesson_id);
        let now = chrono::Utc::now().to_rfc3339();
        
        let _: () = conn.hset_multiple(&key, &[
            ("user_name", user_name.to_string()),
            ("lesson_id", lesson_id.to_string()),
            ("accuracy", accuracy.to_string()),
            ("score", score.to_string()),
            ("updated_at", now),
        ]).await.map_err(|e| format!("Failed to save progress: {}", e))?;
        
        Ok(())
    }
}

/// MongoDB数据库实现
pub struct MongoDatabase {
    client: Client,
    database: String,
}

impl MongoDatabase {
    pub async fn new(uri: &str) -> Result<Self, String> {
        let client = Client::with_uri_str(uri).await
            .map_err(|e| format!("Failed to create MongoDB client: {}", e))?;
        Ok(Self { client, database: "wubi".to_string() })
    }

    fn get_db(&self) -> MongoDb {
        self.client.database(&self.database)
    }

    async fn import_wubi_dict(&self) -> Result<(), String> {
        let dict_path = "data/wubi_dict.json";
        if !Path::new(dict_path).exists() {
            return Err(format!("Wubi dictionary file not found: {}", dict_path));
        }
        let content = fs::read_to_string(dict_path)
            .map_err(|e| format!("Failed to read wubi dictionary file: {}", e))?;
        let entries: Vec<WubiDictEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse wubi dictionary JSON: {}", e))?;
        let db = self.get_db();
        let chars_collection = db.collection::<Document>("wubi_characters");
        let entries_len = entries.len();
        println!("Importing {} wubi dictionary entries to MongoDB...", entries_len);
        let mut docs = Vec::new();
        for entry in entries {
            docs.push(doc! { "character": entry.character, "wubi_code": entry.code });
        }
        if !docs.is_empty() {
            chars_collection.insert_many(docs, None).await
                .map_err(|e| format!("Failed to insert wubi characters: {}", e))?;
        }
        println!("Wubi dictionary import complete: {} entries", entries_len);
        Ok(())
    }

    async fn import_sample_data(&self) -> Result<(), String> {
        let db = self.get_db();
        let lessons_col = db.collection::<Document>("lessons");
        let articles_col = db.collection::<Document>("articles");
        let roots_col = db.collection::<Document>("wubi_roots");
        let mut lessons_docs = Vec::new();
        for (char, code, desc) in [("人", "WG", "练习人字的五笔编码。"), ("日", "KH", "练习日字的五笔编码。"), ("山", "FQ", "练习山字的五笔编码。")] {
            lessons_docs.push(doc! { "character": char, "code": code, "description": desc });
        }
        lessons_col.insert_many(lessons_docs, None).await.map_err(|e| format!("Failed to insert lessons: {}", e))?;
        let mut articles_docs = Vec::new();
        for (title, content, difficulty) in [
            ("练习文章一", "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。", "easy"),
            ("练习文章二", "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。", "medium"),
            ("练习文章三", "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。", "hard"),
        ] {
            articles_docs.push(doc! { "title": title, "content": content, "difficulty": difficulty });
        }
        articles_col.insert_many(articles_docs, None).await.map_err(|e| format!("Failed to insert articles: {}", e))?;
        let mut roots_docs = Vec::new();
        for (char, code, position, desc) in [
            ("一", "GGLL", "G区第一键", "横区起首字根"), ("丿", "TTLL", "T区第一键", "撇区起首字根"),
            ("丨", "HHLL", "H区第一键", "竖区起首字根"), ("丶", "YYLL", "Y区第一键", "捺区起首字根"),
            ("乙", "NNLL", "N区第一键", "折区起首字根"), ("九", "VTNG", "V区第二键", "字根：乙"),
            ("力", "LTNN", "L键", "字根：力"), ("乃", "DETN", "N键", "字根：乃"),
            ("刀", "VNTE", "V键", "字根：刀"), ("卜", "HHYD", "H键", "字根：卜"),
        ] {
            roots_docs.push(doc! { "character": char, "code": code, "position": position, "description": desc });
        }
        roots_col.insert_many(roots_docs, None).await.map_err(|e| format!("Failed to insert roots: {}", e))?;
        println!("Sample data imported to MongoDB");
        Ok(())
    }

    async fn next_id(&self, collection_name: &str) -> Result<i32, String> {
        let db = self.get_db();
        let counter_col = db.collection::<Document>("_counters");
        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::After)
            .build();
        let result = counter_col.find_one_and_update(
            doc! { "_id": collection_name },
            doc! { "$inc": { "seq": 1 } },
            options,
        ).await.map_err(|e| format!("Failed to generate id: {}", e))?;
        let id = result.and_then(|d| d.get_i32("seq").ok()).unwrap_or(1);
        Ok(id)
    }
}

#[async_trait]
impl Database for MongoDatabase {
    async fn init_db(&self) -> Result<(), String> {
        let db = self.get_db();
        let collections = db.list_collection_names(None).await
            .map_err(|e| format!("Failed to list collections: {}", e))?;
        if collections.is_empty() {
            self.import_wubi_dict().await?;
            self.import_sample_data().await?;
        }
        Ok(())
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_characters");
        let doc = collection.find_one(doc! { "character": character }, None).await
            .map_err(|e| format!("Failed to query wubi code: {}", e))?;
        match doc {
            Some(doc) => {
                let wubi_code = doc.get_str("wubi_code").map_err(|e| e.to_string())?.to_string();
                Ok(WubiCharacter { character: character.to_string(), wubi_code })
            }
            None => Err("Character not found".to_string()),
        }
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_characters");
        let mut cursor = collection.find(doc! {}, None).await
            .map_err(|e| format!("Failed to query characters: {}", e))?;
        let mut characters = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e: mongodb::error::Error| e.to_string())? {
            let character = doc.get_str("character").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let wubi_code = doc.get_str("wubi_code").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            characters.push(WubiCharacter { character, wubi_code });
        }
        characters.sort_by(|a, b| a.character.cmp(&b.character));
        Ok(characters)
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("lessons");
        let mut cursor = collection.find(doc! {}, None).await
            .map_err(|e: mongodb::error::Error| format!("Failed to query lessons: {}", e))?;
        let mut lessons = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e: mongodb::error::Error| e.to_string())? {
            let id = doc.get_object_id("_id").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let character = doc.get_str("character").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let code = doc.get_str("code").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let description = doc.get_str("description").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            lessons.push(Lesson { id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(), character, code, description });
        }
        lessons.sort_by_key(|l| l.id);
        Ok(lessons)
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        let lessons = self.get_lessons().await?;
        lessons.into_iter().find(|l| l.id == id)
            .ok_or_else(|| "Lesson not found".to_string())
    }

    async fn create_lesson(&self, character: &str, code: &str, description: &str) -> Result<Lesson, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("lessons");
        let doc = doc! { "character": character, "code": code, "description": description };
        let result = collection.insert_one(doc, None).await
            .map_err(|e| format!("Failed to create lesson: {}", e))?;
        let id = result.inserted_id.as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(Lesson { id, character: character.to_string(), code: code.to_string(), description: description.to_string() })
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("articles");
        let mut cursor = collection.find(doc! {}, None).await
            .map_err(|e: mongodb::error::Error| format!("Failed to query articles: {}", e))?;
        let mut articles = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e: mongodb::error::Error| e.to_string())? {
            let id = doc.get_object_id("_id").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let title = doc.get_str("title").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let content = doc.get_str("content").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let difficulty = doc.get_str("difficulty").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            articles.push(Article { id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(), title, content, difficulty });
        }
        articles.sort_by_key(|a| a.id);
        Ok(articles)
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        let articles = self.get_articles().await?;
        articles.into_iter().find(|a| a.id == id)
            .ok_or_else(|| "Article not found".to_string())
    }

    async fn create_article(&self, title: &str, content: &str, difficulty: &str) -> Result<Article, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("articles");
        let doc = doc! { "title": title, "content": content, "difficulty": difficulty };
        let result = collection.insert_one(doc, None).await
            .map_err(|e| format!("Failed to create article: {}", e))?;
        let id = result.inserted_id.as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(Article { id, title: title.to_string(), content: content.to_string(), difficulty: difficulty.to_string() })
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_roots");
        let mut cursor = collection.find(doc! {}, None).await
            .map_err(|e: mongodb::error::Error| format!("Failed to query roots: {}", e))?;
        let mut roots = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e: mongodb::error::Error| e.to_string())? {
            let id = doc.get_object_id("_id").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let character = doc.get_str("character").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let code = doc.get_str("code").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let position = doc.get_str("position").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let description = doc.get_str("description").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            roots.push(WubiRoot { id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(), character, code, position, description });
        }
        roots.sort_by_key(|r| r.position.clone());
        Ok(roots)
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        let roots = self.get_wubi_roots().await?;
        roots.into_iter().find(|r| r.id == id)
            .ok_or_else(|| "Wubi root not found".to_string())
    }

    async fn create_wubi_root(&self, character: &str, code: &str, position: &str, description: &str) -> Result<WubiRoot, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_roots");
        let doc = doc! { "character": character, "code": code, "position": position, "description": description };
        let result = collection.insert_one(doc, None).await
            .map_err(|e| format!("Failed to create root: {}", e))?;
        let id = result.inserted_id.as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(WubiRoot { id, character: character.to_string(), code: code.to_string(), position: position.to_string(), description: description.to_string() })
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_roots");
        let doc = collection.find_one(doc! { "character": character }, None).await
            .map_err(|e: mongodb::error::Error| format!("Failed to query root: {}", e))?;
        match doc {
            Some(doc) => {
                let id = doc.get_object_id("_id").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
                let character = doc.get_str("character").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
                let code = doc.get_str("code").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
                let position = doc.get_str("position").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
                let description = doc.get_str("description").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
                Ok(Some(WubiRoot { id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(), character, code, position, description }))
            }
            None => Ok(None),
        }
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("users");
        let doc = collection.find_one(doc! { "username": username }, None).await
            .map_err(|e: mongodb::error::Error| format!("Failed to query user: {}", e))?;
        match doc {
            Some(doc) => {
                let id = doc.get_object_id("_id").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
                let username = doc.get_str("username").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
                let email = doc.get_str("email").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
                let password_hash = doc.get_str("password_hash").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
                let created_at = doc.get_str("created_at").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
                Ok(Some(User { id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(), username, email, password_hash, created_at }))
            }
            None => Ok(None),
        }
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let users = self.get_all_users().await?;
        Ok(users.into_iter().find(|u| u.id == id))
    }

    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("users");
        let now = chrono::Utc::now().to_rfc3339();
        let doc = doc! { "username": username, "email": email, "password_hash": password_hash, "created_at": now.clone() };
        let result = collection.insert_one(doc, None).await
            .map_err(|e| format!("Failed to create user: {}", e))?;
        let id = result.inserted_id.as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(User { id, username: username.to_string(), email: email.to_string(), password_hash: password_hash.to_string(), created_at: now })
    }

    async fn save_progress(&self, user_name: &str, lesson_id: i32, accuracy: f32, score: i32) -> Result<(), String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("user_progress");
        let now = chrono::Utc::now().to_rfc3339();
        let doc = doc! { "user_name": user_name, "lesson_id": lesson_id, "accuracy": accuracy, "score": score, "updated_at": now };
        collection.insert_one(doc, None).await
            .map_err(|e| format!("Failed to save progress: {}", e))?;
        Ok(())
    }
}

impl MongoDatabase {
    async fn get_all_users(&self) -> Result<Vec<User>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("users");
        let mut cursor = collection.find(doc! {}, None).await
            .map_err(|e: mongodb::error::Error| format!("Failed to query users: {}", e))?;
        let mut users = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e: mongodb::error::Error| e.to_string())? {
            let id = doc.get_object_id("_id").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let username = doc.get_str("username").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let email = doc.get_str("email").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let password_hash = doc.get_str("password_hash").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            let created_at = doc.get_str("created_at").map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?.to_string();
            users.push(User { id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(), username, email, password_hash, created_at });
        }
        Ok(users)
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
                ("人", "WG", "练习人字的五笔编码。"),
                ("日", "KH", "练习日字的五笔编码。"),
                ("山", "FQ", "练习山字的五笔编码。"),
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

        let art_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM articles")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        if art_count == 0 {
            let articles = [
                ("练习文章一", "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。", "easy"),
                ("练习文章二", "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。", "medium"),
                ("练习文章三", "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。", "hard"),
            ];
            
            for (title, content, difficulty) in articles {
                sqlx::query(
                    "INSERT INTO articles (title, content, difficulty) VALUES ($1, $2, $3)"
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        let root_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wubi_roots")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        if root_count == 0 {
            let roots = [
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
            
            for (char, code, position, desc) in roots {
                sqlx::query(
                    "INSERT INTO wubi_roots (character_val, code, position, description) VALUES ($1, $2, $3, $4)"
                )
                .bind(char)
                .bind(code)
                .bind(position)
                .bind(desc)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        let char_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wubi_characters")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        if char_count == 0 {
            let dict_path = "data/wubi_dict.json";
            if Path::new(dict_path).exists() {
                let content = fs::read_to_string(dict_path)
                    .map_err(|e| format!("Failed to read wubi dictionary: {}", e))?;
                let entries: Vec<WubiDictEntry> = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse wubi dictionary: {}", e))?;
                
                let entries_len = entries.len();
                println!("Importing {} wubi dictionary entries to PostgreSQL...", entries_len);
                
                for entry in entries {
                    sqlx::query(
                        "INSERT INTO wubi_characters (character_val, wubi_code) VALUES ($1, $2) ON CONFLICT (character_val) DO NOTHING"
                    )
                    .bind(&entry.character)
                    .bind(&entry.code)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }
                
                println!("Wubi dictionary import complete: {} entries", entries_len);
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
        });
        lesson
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
        });
        article
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
        });
        root
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
        let result = sqlx::query_as::<_, (i32, String, String, String, Option<chrono::NaiveDateTime>)>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.map(|dt| dt.and_utc().to_rfc3339()).unwrap_or_default()
        }))
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, Option<chrono::NaiveDateTime>)>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.map(|dt| dt.and_utc().to_rfc3339()).unwrap_or_default()
        }))
    }

    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User, String> {
        let user = sqlx::query_as::<_, (i32, String, String, String, Option<chrono::NaiveDateTime>)>(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email, password_hash, created_at"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.map(|dt| dt.and_utc().to_rfc3339()).unwrap_or_default()
        });
        user
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
