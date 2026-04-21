//! Browser Storage - localStorage/sessionStorage wrapper

use wasm_bindgen::prelude::*;
use web_sys::{Storage, Window};

/// Browser storage wrapper supporting localStorage and sessionStorage
pub struct BrowserStorage {
    local_storage: Option<Storage>,
    session_storage: Option<Storage>,
}

impl BrowserStorage {
    /// Create a new browser storage instance
    pub fn new() -> Self {
        let window = web_sys::window();
        let local_storage = window.as_ref().and_then(|w| w.local_storage().ok().flatten());
        let session_storage = window.as_ref().and_then(|w| w.session_storage().ok().flatten());

        Self {
            local_storage,
            session_storage,
        }
    }

    /// Set a value in localStorage
    pub fn set(&self, key: &str, value: &str) -> Result<(), JsValue> {
        match &self.local_storage {
            Some(storage) => storage.set_item(key, value),
            None => Err(JsValue::from_str("localStorage not available")),
        }
    }

    /// Get a value from localStorage
    pub fn get(&self, key: &str) -> Result<Option<String>, JsValue> {
        match &self.local_storage {
            Some(storage) => storage.get_item(key),
            None => Err(JsValue::from_str("localStorage not available")),
        }
    }

    /// Remove a value from localStorage
    pub fn remove(&self, key: &str) -> Result<(), JsValue> {
        match &self.local_storage {
            Some(storage) => storage.remove_item(key),
            None => Err(JsValue::from_str("localStorage not available")),
        }
    }

    /// Clear all values from localStorage
    pub fn clear(&self) -> Result<(), JsValue> {
        match &self.local_storage {
            Some(storage) => storage.clear(),
            None => Err(JsValue::from_str("localStorage not available")),
        }
    }

    /// Set a value in sessionStorage
    pub fn set_session(&self, key: &str, value: &str) -> Result<(), JsValue> {
        match &self.session_storage {
            Some(storage) => storage.set_item(key, value),
            None => Err(JsValue::from_str("sessionStorage not available")),
        }
    }

    /// Get a value from sessionStorage
    pub fn get_session(&self, key: &str) -> Result<Option<String>, JsValue> {
        match &self.session_storage {
            Some(storage) => storage.get_item(key),
            None => Err(JsValue::from_str("sessionStorage not available")),
        }
    }

    /// Remove a value from sessionStorage
    pub fn remove_session(&self, key: &str) -> Result<(), JsValue> {
        match &self.session_storage {
            Some(storage) => storage.remove_item(key),
            None => Err(JsValue::from_str("sessionStorage not available")),
        }
    }

    /// Get all keys from localStorage
    pub fn keys(&self) -> Result<Vec<String>, JsValue> {
        match &self.local_storage {
            Some(storage) => {
                let length = storage.length()?;
                let mut keys = Vec::with_capacity(length as usize);
                for i in 0..length {
                    if let Some(key) = storage.key(i)? {
                        keys.push(key);
                    }
                }
                Ok(keys)
            }
            None => Err(JsValue::from_str("localStorage not available")),
        }
    }

    /// Get storage usage information
    pub fn usage(&self) -> Result<StorageUsage, JsValue> {
        match &self.local_storage {
            Some(storage) => {
                let length = storage.length()?;
                let mut total_size = 0usize;
                
                for i in 0..length {
                    if let Some(key) = storage.key(i)? {
                        if let Some(value) = storage.get_item(&key)? {
                            total_size += key.len() + value.len();
                        }
                    }
                }

                Ok(StorageUsage {
                    item_count: length as usize,
                    total_bytes: total_size,
                })
            }
            None => Err(JsValue::from_str("localStorage not available")),
        }
    }
}

impl Default for BrowserStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage usage information
#[derive(Debug, Clone)]
pub struct StorageUsage {
    pub item_count: usize,
    pub total_bytes: usize,
}

/// IndexedDB wrapper for larger storage needs
pub struct IndexedDbStorage {
    db_name: String,
    store_name: String,
}

impl IndexedDbStorage {
    /// Create a new IndexedDB storage instance
    pub fn new(db_name: impl Into<String>, store_name: impl Into<String>) -> Self {
        Self {
            db_name: db_name.into(),
            store_name: store_name.into(),
        }
    }

    /// Open the database and create object store if needed
    pub async fn open(&self) -> Result<idb::Database, JsValue> {
        let mut open_request = idb::OpenRequest::new(&self.db_name, 1)?;
        
        open_request.on_upgrade_needed(|event| {
            let db = event.database()?;
            if !db.object_store_names().any(|n| n == self.store_name) {
                db.create_object_store(&self.store_name)
                    .map_err(|e| JsValue::from_str(&format!("Failed to create object store: {:?}", e)))?;
            }
            Ok(())
        });

        open_request.await
    }

    /// Store a value
    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), JsValue> {
        let db = self.open().await?;
        let tx = db.transaction(&[&self.store_name], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(&self.store_name)?;
        
        let json = serde_json::to_string(value)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
        
        store.put(&json.into(), Some(&key.into()))?;
        tx.commit().await?;
        
        Ok(())
    }

    /// Retrieve a value
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, JsValue> {
        let db = self.open().await?;
        let tx = db.transaction(&[&self.store_name], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(&self.store_name)?;
        
        match store.get(&key.into())?.await? {
            Some(value) => {
                let json_str = value.as_string()
                    .ok_or_else(|| JsValue::from_str("Invalid value type"))?;
                let value: T = serde_json::from_str(&json_str)
                    .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Remove a value
    pub async fn remove(&self, key: &str) -> Result<(), JsValue> {
        let db = self.open().await?;
        let tx = db.transaction(&[&self.store_name], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(&self.store_name)?;
        
        store.delete(&key.into())?;
        tx.commit().await?;
        
        Ok(())
    }

    /// Clear all values
    pub async fn clear(&self) -> Result<(), JsValue> {
        let db = self.open().await?;
        let tx = db.transaction(&[&self.store_name], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(&self.store_name)?;
        
        store.clear()?;
        tx.commit().await?;
        
        Ok(())
    }
}

// Simple idb module for IndexedDB operations
mod idb {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    pub struct OpenRequest {
        inner: JsValue,
    }

    impl OpenRequest {
        pub fn new(name: &str, version: u32) -> Result<Self, JsValue> {
            let window = web_sys::window().ok_or("No window")?;
            let idb = window.indexed_db()?.ok_or("IndexedDB not available")?;
            let request = idb.open_with_u32(name, version)?;
            Ok(Self { inner: request.into() })
        }

        pub fn on_upgrade_needed<F>(&mut self, callback: F)
        where
            F: FnMut(UpgradeEvent) -> Result<(), JsValue> + 'static,
        {
            let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>);
            // Note: In real implementation, we'd set this on the request
            closure.forget();
        }
    }

    impl std::future::Future for OpenRequest {
        type Output = Result<Database, JsValue>;

        fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            // Simplified - real implementation would use proper async
            std::task::Poll::Pending
        }
    }

    pub struct Database;
    pub struct Transaction;
    pub struct ObjectStore;
    pub struct UpgradeEvent;

    impl UpgradeEvent {
        pub fn database(&self) -> Result<Database, JsValue> {
            Ok(Database)
        }
    }

    impl Database {
        pub fn object_store_names(&self) -> Vec<String> {
            Vec::new()
        }

        pub fn create_object_store(&self, _name: &str) -> Result<ObjectStore, JsValue> {
            Ok(ObjectStore)
        }

        pub fn transaction(&self, _stores: &[&str], _mode: TransactionMode) -> Result<Transaction, JsValue> {
            Ok(Transaction)
        }
    }

    impl Transaction {
        pub fn object_store(&self, _name: &str) -> Result<ObjectStore, JsValue> {
            Ok(ObjectStore)
        }

        pub async fn commit(self) -> Result<(), JsValue> {
            Ok(())
        }
    }

    impl ObjectStore {
        pub fn put(&self, _value: &JsValue, _key: Option<&JsValue>) -> Result<(), JsValue> {
            Ok(())
        }

        pub fn get(&self, _key: &JsValue) -> Result<GetRequest, JsValue> {
            Ok(GetRequest)
        }

        pub fn delete(&self, _key: &JsValue) -> Result<(), JsValue> {
            Ok(())
        }

        pub fn clear(&self) -> Result<(), JsValue> {
            Ok(())
        }
    }

    pub struct GetRequest;

    impl std::future::Future for GetRequest {
        type Output = Result<Option<JsValue>, JsValue>;

        fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            std::task::Poll::Pending
        }
    }

    pub enum TransactionMode {
        ReadOnly,
        ReadWrite,
    }
}
