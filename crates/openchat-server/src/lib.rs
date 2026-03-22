//! OpenChat HTTP 服务（MVP）：路由组装，供 `main` 与集成测试共用。

use std::sync::Arc;

use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub mod api_doc;
pub mod error;
pub mod handlers;
pub mod jwt;
pub mod password;
pub mod state;

pub use error::ApiError;
pub use state::AppState;
pub use utoipa::openapi::OpenApi as OpenApiDocument;

/// 健康检查 JSON 体（与 OpenAPI 约定一致）。
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

fn split_openapi(state: AppState) -> (Router, OpenApiDocument) {
    // 不同 path 必须分别 `routes!`；同一路径多方法才可写在同一个 `routes!(a, b)` 里。
    OpenApiRouter::with_openapi(api_doc::ApiDoc::openapi())
        .routes(routes!(handlers::health_handler))
        .routes(routes!(handlers::register_handler))
        .routes(routes!(handlers::login_handler))
        .routes(routes!(handlers::refresh_handler))
        .routes(routes!(handlers::me_handler))
        .with_state(state)
        .split_for_parts()
}

/// 构建应用路由（单测与进程共用）。
pub fn app(state: AppState) -> Router {
    let (api_router, openapi) = split_openapi(state);
    let spec = Arc::new(openapi);
    let spec_route = spec.clone();
    api_router.route(
        "/api/v1/openapi.json",
        get(move || async move { Json(spec_route.as_ref().clone()) }),
    )
}

/// 与 `app()` 使用相同路由声明，用于导出与快照测试。
pub fn openapi_spec(state: AppState) -> OpenApiDocument {
    split_openapi(state).1
}
