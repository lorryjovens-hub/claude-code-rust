use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardContent {
    pub text: Option<String>,
    pub html: Option<String>,
    pub image: Option<String>,
}

pub struct ClipboardManager;

impl ClipboardManager {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    pub fn read_text(&self) -> Result<String> {
        use std::process::Command;
        let output = Command::new("powershell")
            .args(["-Command", "Get-Clipboard -Text"])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    #[cfg(not(windows))]
    pub fn read_text(&self) -> Result<String> {
        use std::process::Command;
        let output = Command::new("pbpaste")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[cfg(windows)]
    pub fn write_text(&self, text: &str) -> Result<()> {
        use std::process::Command;
        Command::new("powershell")
            .args(["-Command", &format!("Set-Clipboard -Value '{}'", text.replace("'", "''"))])
            .output()?;
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn write_text(&self, text: &str) -> Result<()> {
        use std::process::Command;
        let mut child = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn()?;
        if let Some(ref mut stdin) = child.stdin {
            use std::io::Write;
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }

    pub fn read(&self) -> Result<ClipboardContent> {
        Ok(ClipboardContent {
            text: self.read_text().ok(),
            html: None,
            image: None,
        })
    }

    pub fn write(&self, content: &ClipboardContent) -> Result<()> {
        if let Some(text) = &content.text {
            self.write_text(text)?;
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.write_text("")
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}