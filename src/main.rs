//! Claude Code Rust - Main Entry Point

use clap::Parser;
use claude_code_rs::cli::Cli;
use claude_code_rs::config::Settings;
use claude_code_rs::state::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let settings = Settings::load()?;
    let state = AppState::new(settings);

    match cli.run_async(state).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
