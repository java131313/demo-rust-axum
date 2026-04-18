//! Demo of Rust and axum web framework.
//!
//! <https://github.com/joelparkerhenderson/demo-rust-axum>
//!
//! This demo shows how to:
//!
//! * Create a project using Rust and the axum web framework.
//!
//! * Create axum router routes and their handler functions.
//!
//! * Create responses with HTTP status code OK and HTML text.
//!
//! * Create a binary image and respond with a custom header.
//!
//! * Create functionality for HTTP GET, PUT, PATCH, POST, DELETE.
//!
//! * Use axum extractors for query parameters and path parameters.
//!
//! * Create a data store and access it using RESTful routes.
//!
//! For more see the file `README.md` in the project root.

pub mod app;
pub mod wubi;
pub mod config;
pub mod db;

/// See file book.rs, which defines the `Book` struct.
mod book;

/// See file data.rs, which defines the DATA global variable.
mod data;

/// Use tracing crates for application-level tracing output.
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use crate::db::Database;

#[tokio::main]  
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::event!(tracing::Level::INFO, "main");

    let config = crate::config::AppConfig::from_file("config.yaml")
        .expect("failed to load config.yaml");
    
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
        "redis" => {
            let redis_url = config.database.redis.url.clone();
            let db = crate::db::RedisDatabase::new(&redis_url)
                .expect("failed to create Redis client");
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

    let state = crate::wubi::AppState { db };

    let app = crate::app::app(state);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = app.layer(cors);

    let listener = tokio::net::TcpListener::bind(config.server_address()).await.unwrap();
    println!("Server listening on {}", config.server_address());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
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
