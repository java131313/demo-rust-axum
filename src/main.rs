//! Demo of Rust and axum web framework.
//!
//! <https://github.com/joelparkerhenderson/demo-rust-axum>
//!
//! 架构：SQLite主数据库 + Redis读缓存 + 嵌入式前端静态资源。
//! 前端构建产物（frontend/dist）通过rust-embed编译进二进制，
//! 单个可执行文件同时提供API（/api/*）和Web界面（/）。

pub mod app;
pub mod cache;
pub mod config;
pub mod db;
pub mod openapi;
pub mod static_files;
pub mod swagger;
pub mod wubi;
pub mod wubi_dict;

/// See file book.rs, which defines the `Book` struct.
mod book;

/// See file data.rs, which defines the DATA global variable.
mod data;

use crate::db::Database;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use axum::routing::get;
/// Use tracing crates for application-level tracing output.
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::event!(tracing::Level::INFO, "main");

    let config =
        crate::config::AppConfig::from_file("config.yaml").expect("failed to load config.yaml");

    println!("Database type: {}", config.get_db_type());
    println!("Server address: {}", config.server_address());

    let db: Arc<dyn crate::db::Database> = match config.get_db_type() {
        "mysql" => {
            let database_url = config.get_database_url();
            let pool = sqlx::mysql::MySqlPool::connect(&database_url)
                .await
                .expect("failed to connect to MySQL database");

            let db = crate::db::MySqlDatabase::new(pool);
            db.init_db().await.expect("failed to initialize database");
            Arc::new(db)
        }
        "postgres" | "postgresql" => {
            let database_url = config.get_database_url();
            println!("PostgreSQL URL: {}", database_url);
            let pool = sqlx::postgres::PgPool::connect(&database_url)
                .await
                .expect("failed to connect to PostgreSQL database");

            let db = crate::db::PostgresDatabase::new(pool);
            db.init_db().await.expect("failed to initialize database");
            Arc::new(db)
        }
        "sqlite" => {
            let database_url = config.get_sqlite_url();
            println!("SQLite URL: {}", database_url);

            use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
            use std::str::FromStr;
            let options = SqliteConnectOptions::from_str(&database_url)
                .expect("invalid SQLite connection string")
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(std::time::Duration::from_secs(5));

            let pool = SqlitePoolOptions::new()
                .max_connections(config.database.sqlite.max_connections)
                .connect_with(options)
                .await
                .expect("failed to connect to SQLite database");

            let db = crate::db::SqliteDatabase::new(pool);
            db.init_db().await.expect("failed to initialize database");
            Arc::new(db)
        }
        "redis" => {
            let redis_url = config.database.redis.url.clone();
            let db =
                crate::db::RedisDatabase::new(&redis_url).expect("failed to create Redis client");
            db.init_db().await.expect("failed to initialize Redis");
            Arc::new(db)
        }
        "mongo" | "mongodb" => {
            let mongo_uri = config.database.mongo.uri.clone();
            let db = crate::db::MongoDatabase::new(&mongo_uri)
                .await
                .expect("failed to create MongoDB client");
            db.init_db().await.expect("failed to initialize MongoDB");
            Arc::new(db)
        }
        _ => panic!("Unsupported database type: {}", config.get_db_type()),
    };

    // 连接Redis缓存（可用REDIS_URL环境变量覆盖config.yaml中的配置）
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| config.database.redis.url.clone());

    let db: Arc<dyn Database> = match redis::Client::open(redis_url.as_str()) {
        Ok(redis_client) => match redis::aio::ConnectionManager::new(redis_client).await {
            Ok(conn_manager) => {
                println!("Redis cache connected: {}", sanitize_url(&redis_url));
                let cached = Arc::new(crate::cache::CachedDatabase::new(db, conn_manager));
                // 后台预热文章类缓存，避免首个前端请求穿透到数据库
                {
                    let warmer = cached.clone();
                    tokio::spawn(async move {
                        warmer.warm_cache().await;
                    });
                }
                cached
            }
            Err(e) => {
                eprintln!(
                    "WARNING: Redis connection failed ({}), running WITHOUT cache",
                    e
                );
                db
            }
        },
        Err(e) => {
            eprintln!("WARNING: invalid Redis URL ({}), running WITHOUT cache", e);
            db
        }
    };

    let state = crate::wubi::AppState { db };

    let app = crate::app::app(state)
        // Swagger UI：API文档页面 /swagger-ui/，规范JSON /api-docs/openapi.json
        .route("/api-docs/openapi.json", get(crate::swagger::openapi_json))
        .route("/swagger-ui", get(crate::swagger::index))
        .route("/swagger-ui/", get(crate::swagger::index))
        .route("/swagger-ui/index.html", get(crate::swagger::index))
        .route("/swagger-ui/{*file}", get(crate::swagger::assets));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = app.layer(cors);

    let listener = tokio::net::TcpListener::bind(config.server_address())
        .await
        .unwrap();
    println!("Server listening on {}", config.server_address());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// 隐藏Redis URL中的密码，用于日志输出
fn sanitize_url(url: &str) -> String {
    match url.split_once('@') {
        Some((head, tail)) => match head.split_once("://") {
            Some((scheme, _)) => format!("{}://***@{}", scheme, tail),
            None => "***".to_string(),
        },
        None => url.to_string(),
    }
}

/// Shutdown signal to run axum with graceful shutdown when
/// a user presses Ctrl+C or Unix sends a terminate signal.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
