use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::DbManager;
use crate::db::im_permission_repo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    Open,
    Whitelist,
    PairingCode,
}

impl PermissionMode {
    pub fn as_str(&self) -> &str {
        match self {
            PermissionMode::Open => "open",
            PermissionMode::Whitelist => "whitelist",
            PermissionMode::PairingCode => "pairing",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "open" => Some(PermissionMode::Open),
            "whitelist" => Some(PermissionMode::Whitelist),
            "pairing" => Some(PermissionMode::PairingCode),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermission {
    pub id: String,
    pub platform: String,
    pub user_id: String,
    pub permission_mode: PermissionMode,
    pub is_allowed: bool,
    pub paired_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<im_permission_repo::ImUserPermissionRow> for UserPermission {
    fn from(row: im_permission_repo::ImUserPermissionRow) -> Self {
        Self {
            id: row.id,
            platform: row.platform,
            user_id: row.user_id,
            permission_mode: PermissionMode::from_str(&row.permission_mode)
                .unwrap_or(PermissionMode::Open),
            is_allowed: row.is_allowed,
            paired_code: row.paired_code,
            created_at: row.created_at.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: row.updated_at.parse().unwrap_or_else(|_| Utc::now()),
        }
    }
}

pub struct PermissionManager {
    db: Arc<DbManager>,
    platform_modes: Mutex<HashMap<String, PermissionMode>>,
}

impl PermissionManager {
    pub fn new(db: Arc<DbManager>) -> Self {
        Self {
            db,
            platform_modes: Mutex::new(HashMap::new()),
        }
    }

    pub async fn set_permission_mode(&self, platform: &str, mode: PermissionMode) -> Result<()> {
        let mut modes = self.platform_modes.lock().await;
        modes.insert(platform.to_string(), mode.clone());
        Ok(())
    }

    pub async fn get_permission_mode(&self, platform: &str) -> PermissionMode {
        let modes = self.platform_modes.lock().await;
        modes
            .get(platform)
            .cloned()
            .unwrap_or(PermissionMode::Open)
    }

    pub async fn generate_pairing_code(
        &self,
        platform: &str,
        user_id: &str,
    ) -> Result<String> {
        let code = format!("{:06}", uuid::Uuid::new_v4().as_u128() % 1_000_000u128);
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let db = self.db.clone();
        let platform_clone = platform.to_string();
        let user_id_clone = user_id.to_string();
        let code_clone = code.clone();
        let now_str_clone = now_str.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let existing = db.with_conn(|conn| {
                im_permission_repo::get_im_user_permission_by_platform_user(
                    conn,
                    &platform_clone,
                    &user_id_clone,
                )
            })??;

            match existing {
                Some(row) => {
                    db.with_conn(|conn| {
                        im_permission_repo::update_im_user_permission(
                            conn,
                            &row.id,
                            PermissionMode::PairingCode.as_str(),
                            false,
                            Some(&code_clone),
                            &now_str_clone,
                        )
                    })??;
                }
                None => {
                    let id = uuid::Uuid::new_v4().to_string();
                    db.with_conn(|conn| {
                        im_permission_repo::insert_im_user_permission(
                            conn,
                            &id,
                            &platform_clone,
                            &user_id_clone,
                            PermissionMode::PairingCode.as_str(),
                            false,
                            Some(&code_clone),
                            &now_str_clone,
                            &now_str_clone,
                        )
                    })??;
                }
            }
            Ok(())
        })
        .await??;

        Ok(code)
    }

    pub async fn verify_pairing_code(
        &self,
        platform: &str,
        user_id: &str,
        code: &str,
    ) -> Result<bool> {
        let db = self.db.clone();
        let platform_clone = platform.to_string();
        let user_id_clone = user_id.to_string();
        let code_clone = code.to_string();

        let valid = tokio::task::spawn_blocking(move || -> Result<bool> {
            let existing = db.with_conn(|conn| {
                im_permission_repo::get_im_user_permission_by_platform_user(
                    conn,
                    &platform_clone,
                    &user_id_clone,
                )
            })??;

            match existing {
                Some(row) => Ok(row.paired_code.as_deref() == Some(&code_clone)),
                None => Ok(false),
            }
        })
        .await??;

        Ok(valid)
    }

    pub async fn add_to_whitelist(&self, platform: &str, user_id: &str) -> Result<()> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let db = self.db.clone();
        let platform_clone = platform.to_string();
        let user_id_clone = user_id.to_string();
        let now_str_clone = now_str.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let existing = db.with_conn(|conn| {
                im_permission_repo::get_im_user_permission_by_platform_user(
                    conn,
                    &platform_clone,
                    &user_id_clone,
                )
            })??;

            match existing {
                Some(row) => {
                    db.with_conn(|conn| {
                        im_permission_repo::update_im_user_permission(
                            conn,
                            &row.id,
                            PermissionMode::Whitelist.as_str(),
                            true,
                            None,
                            &now_str_clone,
                        )
                    })??;
                }
                None => {
                    let id = uuid::Uuid::new_v4().to_string();
                    db.with_conn(|conn| {
                        im_permission_repo::insert_im_user_permission(
                            conn,
                            &id,
                            &platform_clone,
                            &user_id_clone,
                            PermissionMode::Whitelist.as_str(),
                            true,
                            None,
                            &now_str_clone,
                            &now_str_clone,
                        )
                    })??;
                }
            }
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub async fn remove_from_whitelist(&self, platform: &str, user_id: &str) -> Result<()> {
        let db = self.db.clone();
        let platform_clone = platform.to_string();
        let user_id_clone = user_id.to_string();

        tokio::task::spawn_blocking(move || -> Result<()> {
            db.with_conn(|conn| {
                im_permission_repo::delete_im_user_permission_by_platform_user(
                    conn,
                    &platform_clone,
                    &user_id_clone,
                )
            })??;
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub async fn check_permission(&self, platform: &str, user_id: &str) -> Result<bool> {
        let mode = self.get_permission_mode(platform).await;

        match mode {
            PermissionMode::Open => Ok(true),
            PermissionMode::Whitelist | PermissionMode::PairingCode => {
                let db = self.db.clone();
                let platform_clone = platform.to_string();
                let user_id_clone = user_id.to_string();

                let allowed = tokio::task::spawn_blocking(move || -> Result<bool> {
                    let existing = db.with_conn(|conn| {
                        im_permission_repo::get_im_user_permission_by_platform_user(
                            conn,
                            &platform_clone,
                            &user_id_clone,
                        )
                    })??;

                    match existing {
                        Some(row) => Ok(row.is_allowed),
                        None => Ok(false),
                    }
                })
                .await??;

                Ok(allowed)
            }
        }
    }

    pub async fn list_permissions(&self, platform: &str) -> Result<Vec<UserPermission>> {
        let db = self.db.clone();
        let platform_clone = platform.to_string();

        let rows: Vec<im_permission_repo::ImUserPermissionRow> = tokio::task::spawn_blocking(move || -> Result<Vec<im_permission_repo::ImUserPermissionRow>> {
            db.with_conn(|conn| {
                im_permission_repo::list_im_user_permissions_by_platform(conn, &platform_clone)
            })?
        })
        .await??;

        Ok(rows.into_iter().map(UserPermission::from).collect())
    }

    pub async fn get_pending_pairing_requests(&self, platform: &str) -> Result<Vec<UserPermission>> {
        let db = self.db.clone();
        let platform_clone = platform.to_string();

        let rows: Vec<im_permission_repo::ImUserPermissionRow> = tokio::task::spawn_blocking(move || -> Result<Vec<im_permission_repo::ImUserPermissionRow>> {
            db.with_conn(|conn| {
                im_permission_repo::list_pending_im_user_permissions(conn, &platform_clone)
            })?
        })
        .await??;

        Ok(rows.into_iter().map(UserPermission::from).collect())
    }

    pub async fn approve_pairing_request(&self, platform: &str, user_id: &str) -> Result<()> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let db = self.db.clone();
        let platform_clone = platform.to_string();
        let user_id_clone = user_id.to_string();
        let now_str_clone = now_str;

        tokio::task::spawn_blocking(move || -> Result<()> {
            let existing = db.with_conn(|conn| {
                im_permission_repo::get_im_user_permission_by_platform_user(
                    conn,
                    &platform_clone,
                    &user_id_clone,
                )
            })??;

            if let Some(row) = existing {
                db.with_conn(|conn| {
                    im_permission_repo::update_im_user_permission(
                        conn,
                        &row.id,
                        PermissionMode::PairingCode.as_str(),
                        true,
                        row.paired_code.as_deref(),
                        &now_str_clone,
                    )
                })??;
            }
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub async fn reject_pairing_request(&self, platform: &str, user_id: &str) -> Result<()> {
        let db = self.db.clone();
        let platform_clone = platform.to_string();
        let user_id_clone = user_id.to_string();

        tokio::task::spawn_blocking(move || -> Result<()> {
            db.with_conn(|conn| {
                im_permission_repo::delete_im_user_permission_by_platform_user(
                    conn,
                    &platform_clone,
                    &user_id_clone,
                )
            })??;
            Ok(())
        })
        .await??;

        Ok(())
    }
}
