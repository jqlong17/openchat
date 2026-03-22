//! 将当前 OpenAPI 文档打印为 JSON（stdout）。在仓库根执行：`cargo run -p openchat-server --bin export-openapi > openapi.json`

use openchat_server::{openapi_spec, AppState};
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let state = AppState {
        pool,
        jwt_secret: "export".to_string(),
    };
    let spec = openapi_spec(state);
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
