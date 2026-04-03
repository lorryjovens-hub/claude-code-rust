//! Time utilities

use chrono::{DateTime, Utc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get the current timestamp as a string
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// Get the current timestamp in milliseconds
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Format a duration in a human-readable way
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Format a duration in a compact way
pub fn format_duration_compact(duration: Duration) -> String {
    let secs = duration.as_secs_f64();
    
    if secs < 1.0 {
        format!("{:.0}ms", secs * 1000.0)
    } else if secs < 60.0 {
        format!("{:.1}s", secs)
    } else if secs < 3600.0 {
        format!("{:.1}m", secs / 60.0)
    } else {
        format!("{:.1}h", secs / 3600.0)
    }
}

/// Parse an RFC3339 timestamp
pub fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Get a relative time string (e.g., "2 minutes ago")
pub fn relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(dt);
    
    let secs = diff.num_seconds();
    let mins = diff.num_minutes();
    let hours = diff.num_hours();
    let days = diff.num_days();
    
    if secs < 60 {
        "just now".to_string()
    } else if mins < 60 {
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if hours < 24 {
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if days < 30 {
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

/// Sleep for a given duration (async)
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// Sleep for a given number of milliseconds (async)
pub async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

/// Measure the execution time of a function
pub fn measure<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = SystemTime::now();
    let result = f();
    let elapsed = start.elapsed().unwrap_or_default();
    (result, elapsed)
}

/// Measure the execution time of an async function
pub async fn measure_async<F, T>(f: F) -> (T, Duration)
where
    F: std::future::Future<Output = T>,
{
    let start = SystemTime::now();
    let result = f.await;
    let elapsed = start.elapsed().unwrap_or_default();
    (result, elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
    }

    #[test]
    fn test_format_duration_compact() {
        assert_eq!(format_duration_compact(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration_compact(Duration::from_secs(5)), "5.0s");
    }

    #[test]
    fn test_relative_time() {
        let now = Utc::now();
        assert_eq!(relative_time(now), "just now");
        
        let past = now - chrono::Duration::minutes(5);
        assert_eq!(relative_time(past), "5 minutes ago");
    }

    #[test]
    fn test_measure() {
        let (result, duration) = measure(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
    }
}
