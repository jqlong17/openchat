//! OpenAPI：由 utoipa 生成，与根目录 `openapi.json` 一致。

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use openchat_server::{openapi_spec, AppState};
use serde_json::Value;
use sqlx::sqlite::SqlitePool;
use tower::ServiceExt;

#[tokio::test]
async fn exported_openapi_matches_committed_json() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrate");
    let state = AppState {
        pool,
        jwt_secret: "test".to_string(),
    };
    let spec = openapi_spec(state);
    let actual: Value = serde_json::to_value(&spec).expect("serialize spec");

    let expected_raw = include_str!("../../../openapi.json");
    let expected: Value = serde_json::from_str(expected_raw).expect("parse openapi.json");

    assert_eq!(
        actual, expected,
        "运行 `cargo run -p openchat-server --bin export-openapi > openapi.json` 更新契约"
    );
}

#[tokio::test]
async fn openapi_json_route_returns_spec() {
    let app = common::test_router().await;
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("openapi.json");
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.expect("body").to_bytes();
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["openapi"], "3.1.0");
    assert!(v["paths"].get("/api/v1/health").is_some());
}
