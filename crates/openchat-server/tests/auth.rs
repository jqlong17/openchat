//! 阶段 1：注册 / 登录 / refresh / `/me` 集成测试。

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn register_login_me_happy_path() {
    let app = common::test_router().await;

    let reg = json!({
        "username": "alice",
        "password": "password123",
        "display_name": "Alice"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(reg.to_string()))
                .unwrap(),
        )
        .await
        .expect("register");
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let access = v["access_token"].as_str().unwrap().to_string();
    let _refresh = v["refresh_token"].as_str().unwrap();

    let me = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/me")
                .header(header::AUTHORIZATION, format!("Bearer {}", access))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("me");
    assert_eq!(me.status(), StatusCode::OK);
    let me_body = me.into_body().collect().await.unwrap().to_bytes();
    let u: serde_json::Value = serde_json::from_slice(&me_body).unwrap();
    assert_eq!(u["username"], "alice");
    assert_eq!(u["display_name"], "Alice");
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let app = common::test_router().await;

    let reg = json!({
        "username": "bob",
        "password": "password123",
        "display_name": "Bob"
    });
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(reg.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let login = json!({
        "username": "bob",
        "password": "wrongpassword"
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(login.to_string()))
                .unwrap(),
        )
        .await
        .expect("login");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn refresh_returns_new_tokens() {
    let app = common::test_router().await;

    let reg = json!({
        "username": "carol",
        "password": "password123",
        "display_name": "Carol"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(reg.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let refresh = v["refresh_token"].as_str().unwrap().to_string();

    let refresh_body = json!({ "refresh_token": refresh });
    let res2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(refresh_body.to_string()))
                .unwrap(),
        )
        .await
        .expect("refresh");
    assert_eq!(res2.status(), StatusCode::OK);
    let b2 = res2.into_body().collect().await.unwrap().to_bytes();
    let v2: serde_json::Value = serde_json::from_slice(&b2).unwrap();
    assert!(v2["access_token"].as_str().unwrap().len() > 10);
    assert!(v2["refresh_token"].as_str().unwrap().len() > 10);
}
