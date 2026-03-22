//! OpenChat 服务端入口。

use std::net::SocketAddr;
use std::path::Path;

use openchat_server::{app, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/openchat.db".to_string());

    if !database_url.contains("memory") {
        if let Some(rest) = database_url.strip_prefix("sqlite:") {
            if let Some(parent) = Path::new(rest).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
        }
    }

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        eprintln!(
            "WARNING: JWT_SECRET unset; using insecure dev default. Set JWT_SECRET in production."
        );
        "dev-insecure-change-me".to_string()
    });

    let state = AppState { pool, jwt_secret };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("listening on http://{addr}");
    axum::serve(listener, app(state)).await?;
    Ok(())
}
