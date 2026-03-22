//! OpenChat HTTP 服务（MVP）：路由组装，供 `main` 与集成测试共用。

use axum::routing::{get, post};
use axum::Router;
use serde::Serialize;

pub mod error;
pub mod handlers;
pub mod jwt;
pub mod password;
pub mod state;

pub use error::ApiError;
pub use state::AppState;

/// 健康检查 JSON 体（与 `openapi.yaml` 一致）。
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// 构建应用路由（单测与进程共用）。
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health", get(handlers::health_handler))
        .route("/api/v1/auth/register", post(handlers::register_handler))
        .route("/api/v1/auth/login", post(handlers::login_handler))
        .route("/api/v1/auth/refresh", post(handlers::refresh_handler))
        .route("/api/v1/me", get(handlers::me_handler))
        .with_state(state)
}
