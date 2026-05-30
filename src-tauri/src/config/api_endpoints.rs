//! API endpoint resolution — maps named services to URLs.
//! Used by the bridge's update-check and other external integrations.

/// Resolve a named API endpoint to a full URL.
/// Returns empty string if the service is unknown.
pub fn resolve_api_url(service: Option<&str>, path: &str) -> String {
    let base = match service {
        Some("clawparrot") => "https://clawparrot.com",
        Some("anthropic") => "https://api.anthropic.com",
        Some("openai") => "https://api.openai.com",
        Some("github") => "https://api.github.com",
        _ => return String::new(),
    };
    format!("{}/{}", base, path.trim_start_matches('/'))
}
