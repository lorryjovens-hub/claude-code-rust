use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationOptions {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub silent: Option<bool>,
    pub urgency: Option<String>,
    pub timeout: Option<u32>,
}

pub struct NotificationManager;

impl NotificationManager {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    pub fn show(&self, options: &NotificationOptions) -> Result<()> {
        use std::process::Command;

        let title = options.title.replace("\"", "\\\"");
        let body = options.body.replace("\"", "\\\"");

        let script = format!(
            r#"[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null; $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); $textNodes = $template.GetElementsByTagName('text'); $textNodes.Item(0).AppendChild($template.CreateTextNode('{}')) | Out-Null; $textNodes.Item(1).AppendChild($template.CreateTextNode('{}')) | Out-Null; $toast = [Windows.UI.Notifications.ToastNotification]::new($template); [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Claude Desktop').Show($toast)"#,
            title, body
        );

        Command::new("powershell")
            .args(["-WindowStyle", "Hidden", "-Command", &script])
            .spawn()?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn show(&self, options: &NotificationOptions) -> Result<()> {
        use std::process::Command;

        let title = options.title.replace("\"", "\\\"");
        let body = options.body.replace("\"", "\\\"");

        let script = format!(
            r#"display notification "{}" with title "{}""#,
            body, title
        );

        Command::new("osascript")
            .args(["-e", &script])
            .spawn()?;

        Ok(())
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    pub fn show(&self, options: &NotificationOptions) -> Result<()> {
        use std::process::Command;

        let mut args = vec![];

        if let Some(urgency) = &options.urgency {
            args.push("--urgency".to_string());
            args.push(urgency.clone());
        }

        args.push("--app-name".to_string());
        args.push("Claude Desktop".to_string());

        args.push(options.title.clone());
        args.push(options.body.clone());

        Command::new("notify-send")
            .args(&args)
            .spawn()?;

        Ok(())
    }

    pub fn show_simple(&self, title: &str, body: &str) -> Result<()> {
        self.show(&NotificationOptions {
            title: title.to_string(),
            body: body.to_string(),
            icon: None,
            silent: None,
            urgency: None,
            timeout: None,
        })
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}