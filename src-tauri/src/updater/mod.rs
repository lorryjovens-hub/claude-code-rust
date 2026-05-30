use anyhow::Result;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const EMBEDDED_PUBLIC_KEY: &[u8; 32] = include_bytes!("../../update_pubkey.bin");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub release_date: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub is_mandatory: bool,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub available: bool,
    pub downloading: bool,
    pub downloaded: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

pub struct AutoUpdater {
    update_url: String,
    current_version: String,
    cache_dir: PathBuf,
}

impl AutoUpdater {
    pub fn new(update_url: &str, current_version: &str, cache_dir: PathBuf) -> Self {
        Self {
            update_url: update_url.to_string(),
            current_version: current_version.to_string(),
            cache_dir,
        }
    }

    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        let manifest_url = format!("{}/manifest.json", self.update_url);

        let response = reqwest::get(&manifest_url).await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let manifest: serde_json::Value = response.json().await?;

        let latest_version = manifest["version"].as_str().unwrap_or("0.0.0");

        if self.is_newer_version(latest_version) {
            Ok(Some(UpdateInfo {
                version: latest_version.to_string(),
                release_date: manifest["release_date"].as_str().unwrap_or("").to_string(),
                release_notes: manifest["release_notes"].as_str().map(|s| s.to_string()),
                download_url: manifest["download_url"].as_str().map(|s| s.to_string()),
                is_mandatory: manifest["is_mandatory"].as_bool().unwrap_or(false),
                signature: manifest["signature"].as_str().map(|s| s.to_string()),
            }))
        } else {
            Ok(None)
        }
    }

    fn is_newer_version(&self, latest: &str) -> bool {
        let current = self.current_version.trim_start_matches('v');
        let latest = latest.trim_start_matches('v');

        let current_parts: Vec<u32> = current
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let latest_parts: Vec<u32> = latest
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        for (c, l) in current_parts.iter().zip(latest_parts.iter()) {
            if l > c {
                return true;
            } else if l < c {
                return false;
            }
        }

        latest_parts.len() > current_parts.len()
    }

    pub async fn download_update(&self, url: &str, signature: Option<&str>) -> Result<PathBuf> {
        let response = reqwest::get(url).await?;
        let _total_bytes = response.content_length().unwrap_or(0);

        let file_name = url.split('/').last().unwrap_or("update");
        let output_path = self.cache_dir.join(file_name);

        std::fs::create_dir_all(&self.cache_dir)?;

        let mut file = tokio::fs::File::create(&output_path).await?;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tokio::io::copy(&mut chunk.as_ref(), &mut file).await?;
        }

        if let Some(sig) = signature {
            if !self.verify_signature(&output_path, sig)? {
                tracing::error!("Update signature verification failed, deleting downloaded file");
                let _ = std::fs::remove_file(&output_path);
                return Err(anyhow::anyhow!("Signature verification failed"));
            }
            tracing::info!("Update signature verified successfully");
        }

        Ok(output_path)
    }

    pub fn verify_signature(&self, file_path: &PathBuf, signature_hex: &str) -> Result<bool> {
        let pubkey_bytes = *EMBEDDED_PUBLIC_KEY;
        let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {}", e))?;

        let signature_bytes: Vec<u8> = (0..signature_hex.len())
            .step_by(2)
            .filter_map(|i| {
                u8::from_str_radix(&signature_hex[i..i + 2.min(signature_hex.len() - i)], 16).ok()
            })
            .collect();

        if signature_bytes.len() != 64 {
            tracing::warn!(signature_len = signature_bytes.len(), "Invalid signature length");
            return Ok(false);
        }

        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid signature: {}", e))?;

        let file_content = std::fs::read(file_path)?;

        match verifying_key.verify(&file_content, &signature) {
            Ok(()) => Ok(true),
            Err(e) => {
                tracing::warn!(error = %e, "Signature verification failed");
                Ok(false)
            }
        }
    }

    pub fn install_update(&self, file_path: &PathBuf) -> Result<()> {
        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("cmd.exe")
                .args(["/C", "start", "", file_path.to_str().unwrap_or("")])
                .spawn()?;
        }

        #[cfg(not(windows))]
        {
            use std::process::Command;
            Command::new("open")
                .arg(file_path)
                .spawn()?;
        }

        Ok(())
    }
}
