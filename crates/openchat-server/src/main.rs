//! OpenChat 服务端入口。

use std::net::SocketAddr;

use openchat_server::app;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("listening on http://{addr}");
    axum::serve(listener, app()).await?;
    Ok(())
}
