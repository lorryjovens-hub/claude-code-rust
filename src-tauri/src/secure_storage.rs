use tauri_plugin_secure_storage::{OptionsRequest, SecureStorageExt};

const API_KEY_STORAGE_KEY: &str = "claude_desktop_api_key";
const GATEWAY_USER_KEY: &str = "claude_desktop_gateway_user";
const GATEWAY_QUOTA_KEY: &str = "claude_desktop_gateway_quota";
const AUTH_TOKEN_KEY: &str = "claude_desktop_auth_token";

#[derive(Debug, thiserror::Error)]
pub enum SecureStorageError {
    #[error("secure storage not available")]
    NotAvailable,
    #[error("storage operation failed: {0}")]
    OperationFailed(String),
}

pub struct SecureKeyStore;

impl SecureKeyStore {
    pub fn new() -> Self {
        Self
    }

    fn get<R: tauri::Runtime>(
        &self,
        app_handle: &tauri::AppHandle<R>,
        key: &str,
    ) -> Result<Option<String>, SecureStorageError> {
        let storage = app_handle.secure_storage();
        let resp = storage
            .get_item(
                app_handle.clone(),
                OptionsRequest {
                    prefixed_key: Some(key.to_string()),
                    data: None,
                    sync: None,
                    keychain_access: None,
                },
            )
            .map_err(|e| SecureStorageError::OperationFailed(e.to_string()))?;
        Ok(resp.data)
    }

    fn set<R: tauri::Runtime>(
        &self,
        app_handle: &tauri::AppHandle<R>,
        key: &str,
        value: &str,
    ) -> Result<(), SecureStorageError> {
        let storage = app_handle.secure_storage();
        storage
            .set_item(
                app_handle.clone(),
                OptionsRequest {
                    prefixed_key: Some(key.to_string()),
                    data: Some(value.to_string()),
                    sync: None,
                    keychain_access: None,
                },
            )
            .map_err(|e| SecureStorageError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    fn delete<R: tauri::Runtime>(
        &self,
        app_handle: &tauri::AppHandle<R>,
        key: &str,
    ) -> Result<(), SecureStorageError> {
        let storage = app_handle.secure_storage();
        storage
            .remove_item(
                app_handle.clone(),
                OptionsRequest {
                    prefixed_key: Some(key.to_string()),
                    data: None,
                    sync: None,
                    keychain_access: None,
                },
            )
            .map_err(|e| SecureStorageError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    pub fn get_api_key<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<Option<String>, SecureStorageError> {
        self.get(app_handle, API_KEY_STORAGE_KEY)
    }

    pub fn set_api_key<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>, key: &str) -> Result<(), SecureStorageError> {
        self.set(app_handle, API_KEY_STORAGE_KEY, key)
    }

    pub fn delete_api_key<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<(), SecureStorageError> {
        self.delete(app_handle, API_KEY_STORAGE_KEY)
    }

    pub fn get_gateway_user<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<Option<String>, SecureStorageError> {
        self.get(app_handle, GATEWAY_USER_KEY)
    }

    pub fn set_gateway_user<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>, user: &str) -> Result<(), SecureStorageError> {
        self.set(app_handle, GATEWAY_USER_KEY, user)
    }

    pub fn delete_gateway_user<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<(), SecureStorageError> {
        self.delete(app_handle, GATEWAY_USER_KEY)
    }

    pub fn get_gateway_quota<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<Option<String>, SecureStorageError> {
        self.get(app_handle, GATEWAY_QUOTA_KEY)
    }

    pub fn set_gateway_quota<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>, quota: &str) -> Result<(), SecureStorageError> {
        self.set(app_handle, GATEWAY_QUOTA_KEY, quota)
    }

    pub fn delete_gateway_quota<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<(), SecureStorageError> {
        self.delete(app_handle, GATEWAY_QUOTA_KEY)
    }

    pub fn get_auth_token<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<Option<String>, SecureStorageError> {
        self.get(app_handle, AUTH_TOKEN_KEY)
    }

    pub fn set_auth_token<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>, token: &str) -> Result<(), SecureStorageError> {
        self.set(app_handle, AUTH_TOKEN_KEY, token)
    }

    pub fn delete_auth_token<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<(), SecureStorageError> {
        self.delete(app_handle, AUTH_TOKEN_KEY)
    }

    pub fn clear_all<R: tauri::Runtime>(&self, app_handle: &tauri::AppHandle<R>) -> Result<(), SecureStorageError> {
        self.delete_api_key(app_handle)?;
        self.delete_gateway_user(app_handle)?;
        self.delete_gateway_quota(app_handle)?;
        self.delete_auth_token(app_handle)?;
        Ok(())
    }
}

#[tauri::command]
pub async fn secure_get_api_key(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = SecureKeyStore::new();
    store.get_api_key(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_set_api_key(app_handle: tauri::AppHandle, key: String) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.set_api_key(&app_handle, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_delete_api_key(app_handle: tauri::AppHandle) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.delete_api_key(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_get_gateway_user(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = SecureKeyStore::new();
    store.get_gateway_user(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_set_gateway_user(app_handle: tauri::AppHandle, user: String) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.set_gateway_user(&app_handle, &user).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_delete_gateway_user(app_handle: tauri::AppHandle) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.delete_gateway_user(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_get_gateway_quota(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = SecureKeyStore::new();
    store.get_gateway_quota(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_set_gateway_quota(app_handle: tauri::AppHandle, quota: String) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.set_gateway_quota(&app_handle, &quota).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_delete_gateway_quota(app_handle: tauri::AppHandle) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.delete_gateway_quota(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_get_auth_token(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = SecureKeyStore::new();
    store.get_auth_token(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_set_auth_token(app_handle: tauri::AppHandle, token: String) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.set_auth_token(&app_handle, &token).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_delete_auth_token(app_handle: tauri::AppHandle) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.delete_auth_token(&app_handle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secure_clear_all(app_handle: tauri::AppHandle) -> Result<(), String> {
    let store = SecureKeyStore::new();
    store.clear_all(&app_handle).map_err(|e| e.to_string())
}
