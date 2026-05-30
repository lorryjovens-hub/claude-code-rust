use super::AppState;

pub mod system;
pub mod chat;
pub mod conversations;
pub mod tools;
pub mod projects;
pub mod uploads;
pub mod providers;
pub mod config;
pub mod skills;
pub mod tasks;
pub mod mcp;
pub mod engines;
pub mod research;
pub mod computer_use;
pub mod git;
pub mod terminal;
pub mod process;
pub mod clipboard;
pub mod notification;
pub mod logs;
pub mod analytics;
pub mod memory;
pub mod watcher;
pub mod updater;
pub mod worktrees;
pub mod agents;
pub mod filesystem;
pub mod ide;
pub mod h5;
pub mod im;
pub mod costs;
pub mod preview;
pub mod workflow;
pub mod sync;
pub mod caveman;

use axum::Router;

pub fn build_all_routes() -> Router<AppState> {
    Router::new()
        .merge(system::routes())
        .merge(chat::routes())
        .merge(conversations::routes())
        .merge(tools::routes())
        .merge(projects::routes())
        .merge(uploads::routes())
        .merge(providers::routes())
        .merge(config::routes())
        .merge(skills::routes())
        .merge(tasks::routes())
        .merge(mcp::routes())
        .merge(engines::routes())
        .merge(research::routes())
        .merge(computer_use::routes())
        .merge(git::routes())
        .merge(terminal::routes())
        .merge(process::routes())
        .merge(clipboard::routes())
        .merge(notification::routes())
        .merge(logs::routes())
        .merge(analytics::routes())
        .merge(memory::routes())
        .merge(watcher::routes())
        .merge(updater::routes())
        .merge(worktrees::routes())
        .merge(agents::routes())
        .merge(filesystem::routes())
        .merge(ide::routes())
        .merge(h5::routes())
        .merge(im::routes())
        .merge(costs::routes())
        .merge(preview::routes())
        .merge(workflow::routes())
        .merge(sync::routes())
        .merge(caveman::routes())
}
