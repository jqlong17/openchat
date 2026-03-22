//! OpenChat HTTP 服务（MVP）：路由组装，供 `main` 与集成测试共用。

use axum::{routing::get, Json, Router};
use serde::Serialize;

/// 健康检查 JSON 体（与 `openapi.yaml` 一致）。
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// 构建应用路由（单测与进程共用）。
pub fn app() -> Router {
    Router::new().route("/api/v1/health", get(health_handler))
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
