//! Web Server - Axum server for the plugin marketplace

use axum::serve;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error};

use super::{
    handlers::AppState,
    routes::create_router,
};

/// Web server for the plugin marketplace
pub struct WebServer {
    addr: SocketAddr,
    state: Arc<AppState>,
}

impl WebServer {
    /// Create a new web server
    pub fn new(port: u16) -> Self {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let state = Arc::new(AppState::new());

        Self { addr, state }
    }

    /// Create a new web server with custom address
    pub fn with_addr(addr: SocketAddr) -> Self {
        let state = Arc::new(AppState::new());

        Self { addr, state }
    }

    /// Get the server address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Run the web server
    pub async fn run(self) -> anyhow::Result<()> {
        let app = create_router(self.state);

        info!("Starting web server on http://{}", self.addr);

        let listener = TcpListener::bind(self.addr).await?;
        
        serve(listener, app)
            .await?;

        Ok(())
    }

    /// Run the web server with graceful shutdown
    pub async fn run_with_shutdown(self, shutdown_signal: tokio::sync::oneshot::Receiver<()>) -> anyhow::Result<()> {
        let app = create_router(self.state);

        info!("Starting web server on http://{}", self.addr);

        let listener = TcpListener::bind(self.addr).await?;
        
        serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_signal.await;
                info!("Shutdown signal received, stopping web server");
            })
            .await?;

        Ok(())
    }
}

impl Default for WebServer {
    fn default() -> Self {
        Self::new(8080)
    }
}

/// Start the web server (convenience function)
pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let server = WebServer::new(port);
    server.run().await
}
