use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub kind: String,
    pub mode: String,
}

pub struct FileWatcher {
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    subscriptions: Arc<Mutex<HashMap<String, broadcast::Sender<FileChange>>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: Arc::new(Mutex::new(None)),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let subscriptions = self.subscriptions.clone();
        
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let change = FileChange {
                        path: event.paths.first()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        kind: format!("{:?}", event.kind),
                        mode: match event.kind {
                            notify::EventKind::Create(_) => "created",
                            notify::EventKind::Modify(_) => "modified",
                            notify::EventKind::Remove(_) => "removed",
                            notify::EventKind::Access(_) => "accessed",
                            _ => "other",
                        }.to_string(),
                    };

                    let subscriptions = subscriptions.blocking_lock();
                    for tx in subscriptions.values() {
                        let _ = tx.send(change.clone());
                    }
                }
            },
            Config::default(),
        )?;

        *self.watcher.lock().await = Some(watcher);
        Ok(())
    }

    pub async fn watch(&self, path: &str) -> Result<()> {
        let mut watcher_guard = self.watcher.lock().await;
        if let Some(watcher) = watcher_guard.as_mut() {
            let path = Path::new(path);
            if path.exists() {
                let mode = if path.is_dir() {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };
                watcher.watch(path, mode)?;
            }
        }
        Ok(())
    }

    pub async fn unwatch(&self, path: &str) -> Result<()> {
        let mut watcher_guard = self.watcher.lock().await;
        if let Some(watcher) = watcher_guard.as_mut() {
            watcher.unwatch(Path::new(path))?;
        }
        Ok(())
    }

    pub async fn subscribe(&self, id: &str) -> broadcast::Receiver<FileChange> {
        let (tx, rx) = broadcast::channel(100);
        self.subscriptions.lock().await.insert(id.to_string(), tx);
        rx
    }

    pub async fn unsubscribe(&self, id: &str) {
        self.subscriptions.lock().await.remove(id);
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}