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
use sqlx::mysql::MySqlPool;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::cors::{CorsLayer, Any};

/// The main function does these steps: 
/// - Start tracing and emit a tracing event.
/// - Get a command line argument as our bind address.
/// - Create our application which is an axum router/.
/// - Run our app using a hyper server.
#[tokio::main]  
async fn main() {
    // Start tracing and emit a tracing event.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::event!(tracing::Level::INFO, "main");

    // Load configuration from config.yaml
    let config = crate::config::AppConfig::from_file("config.yaml")
        .expect("failed to load config.yaml");
    
    println!("Database type: {}", config.get_db_type());
    println!("Server address: {}", config.server_address());

    // Create MySQL connection pool
    let database_url = config.get_database_url();
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("failed to connect to MySQL database");
    
    // Initialize database schema
    crate::wubi::init_db(&pool)
        .await
        .expect("failed to initialize database schema");

    let state = crate::wubi::AppState { pool };

    // Create our application which is an axum router.
    let app = crate::app::app(state);

    // Add CORS middleware to allow frontend requests.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = app.layer(cors);

    // Run our app using a hyper server.
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
