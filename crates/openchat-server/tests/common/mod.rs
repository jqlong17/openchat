//! 集成测试共用：内存 SQLite + 迁移 + `AppState`。

use axum::Router;
use openchat_server::{app, AppState};
use sqlx::sqlite::SqlitePool;

pub async fn test_router() -> Router {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("sqlite memory pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrate");
    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };
    app(state)
}
