use serde::{Deserialize, Serialize};
use std::fs;

/// 应用程序配置结构
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

/// 数据库配置
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// 数据库类型: "mysql" 或 "postgres" 或 "redis" 或 "mongo"
    #[serde(rename = "type", default = "default_db_type")]
    pub db_type: String,
    pub mysql: DbConnectionConfig,
    pub postgres: DbConnectionConfig,
    pub redis: RedisConfig,
    pub mongo: MongoConfig,
}

fn default_db_type() -> String {
    "mongo".to_string()
}

/// Redis配置
#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

/// MongoDB配置
#[derive(Debug, Deserialize, Clone)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
    #[serde(default = "default_max_connections")]
    pub max_pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DbConnectionConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

/// 认证配置
#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    #[serde(default = "default_token_expiration")]
    pub token_expiration_hours: i64,
}

fn default_token_expiration() -> i64 {
    24
}

impl AppConfig {
    /// 从YAML文件加载配置
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 根据配置类型获取数据库连接URL
    pub fn get_database_url(&self) -> String {
        match self.database.db_type.as_str() {
            "mysql" => {
                let db = &self.database.mysql;
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    db.username, db.password, db.host, db.port, db.database
                )
            }
            "postgres" | "postgresql" => {
                let db = &self.database.postgres;
                format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    db.username, db.password, db.host, db.port, db.database
                )
            }
            "redis" => {
                self.database.redis.url.clone()
            }
            "mongo" | "mongodb" => {
                self.database.mongo.uri.clone()
            }
            _ => panic!("不支持的数据库类型: {}", self.database.db_type),
        }
    }

    /// 获取数据库类型
    pub fn get_db_type(&self) -> &str {
        &self.database.db_type
    }

    /// 获取服务器地址
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

/// 数据库模型 - 与数据库类型无关
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lesson {
    pub id: i32,
    pub character: String,
    pub code: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub difficulty: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WubiRoot {
    pub id: i32,
    pub character: String,
    pub code: String,
    pub position: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WubiCharacter {
    pub id: i32,
    pub character: String,
    pub simple_code: String,
    pub full_code: String,
    pub pinyin: String,
    pub remark: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnglishText {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub difficulty: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyRadical {
    pub id: i32,
    pub key_char: String,
    pub radicals: String,
    pub formula: String,
    pub description: String,
}
