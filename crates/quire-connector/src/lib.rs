//! `quire serve`: a local HTTP endpoint for a browser extension (`.knowledge/roadmap.md`, M8),
//! like Zotero Connector's and ReadCube's. The extension sends what the user's browser already
//! shows: the page's HTML (so publishers' "are you human" checks never apply; the user passed
//! them) or a PDF it downloaded.
//!
//! Safety: it listens on 127.0.0.1 only, and every request but `OPTIONS` needs the token from
//! [`token`] (`Authorization: Bearer <token>`), so web pages cannot use it.
//!
//! - [`token`]: the access token file.
//! - [`routes`]: the API.

pub mod routes;
pub mod token;

use std::net::SocketAddr;
use std::sync::Arc;

use quire_engine::Engine;

/// Default port (unassigned by IANA; Zotero's connector uses 23119).
pub const DEFAULT_PORT: u16 = 23150;

/// Serves `engine` on 127.0.0.1:`port` until the process ends. Automatic exports are rewritten
/// after every request that changed the library.
pub async fn serve(engine: Arc<Engine>, port: u16, token: String) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await?;
    run(listener, engine, token).await
}

/// Serves on an already bound `listener` (tests bind port 0).
pub async fn run(listener: tokio::net::TcpListener, engine: Arc<Engine>, token: String) -> std::io::Result<()> {
    tracing::info!(address = %listener.local_addr()?, "serving");
    axum::serve(listener, routes::router(engine, token)).await
}
