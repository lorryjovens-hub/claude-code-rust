use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

pub struct SsePayloads {
    pub payloads: Vec<String>,
    pub remainder: String,
}

pub fn consume_sse_payloads(buffer: &str) -> SsePayloads {
    let normalized = buffer.replace("\r\n", "\n");
    let mut parts: Vec<&str> = normalized.split("\n\n").collect();
    let remainder = parts.pop().unwrap_or("").to_string();

    let mut payloads = Vec::new();
    for part in parts {
        let mut data_lines: Vec<String> = Vec::new();
        for raw_line in part.split('\n') {
            if let Some(data) = raw_line.strip_prefix("data:") {
                let trimmed = data.strip_prefix(' ').unwrap_or(data);
                if !trimmed.is_empty() {
                    data_lines.push(trimmed.to_string());
                }
            }
        }
        if !data_lines.is_empty() {
            payloads.push(data_lines.join("\n").trim().to_string());
        }
    }

    SsePayloads { payloads, remainder }
}

pub fn decode_loose_json_string(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    // Try direct JSON parsing first (handles valid escape sequences like \n, \t, \\)
    let quoted = format!("\"{}\"", value);
    if let Ok(Value::String(s)) = serde_json::from_str::<Value>(&quoted) {
        return s;
    }
    // Fall back to loose parsing: escape special characters then re-parse
    let escaped = value
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n");
    let quoted = format!("\"{}\"", escaped);
    match serde_json::from_str::<Value>(&quoted) {
        Ok(Value::String(s)) => s,
        _ => value.to_string(),
    }
}

fn escape_regex_field(field: &str) -> String {
    regex::escape(field)
}

pub fn extract_loose_json_string_field(raw: &str, field_name: &str, allow_truncated: bool) -> Option<String> {
    if raw.is_empty() {
        return None;
    }

    let escaped_field = escape_regex_field(field_name);
    let pattern_str = format!(r#""{}"\s*:\s*"((?:\\\\.|[^"\\\\])*)""#, escaped_field);
    let re = Regex::new(&pattern_str).ok()?;

    if let Some(caps) = re.captures(raw) {
        if let Some(m) = caps.get(1) {
            return Some(decode_loose_json_string(m.as_str()));
        }
    }

    if allow_truncated {
        let open_pattern_str = format!(r#""{}"\s*:\s*""#, escaped_field);
        let open_re = Regex::new(&open_pattern_str).ok()?;
        if let Some(open_match) = open_re.find(raw) {
            let start_idx = open_match.end();
            let truncated = &raw[start_idx..];
            let truncated = strip_trailing_incomplete_escape(truncated);
            if !truncated.is_empty() {
                return Some(decode_loose_json_string(&truncated));
            }
        }
    }

    None
}

fn strip_trailing_incomplete_escape(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut backslash_count = 0;
    for &b in bytes.iter().rev() {
        if b == b'\\' {
            backslash_count += 1;
        } else {
            break;
        }
    }
    if backslash_count % 2 == 1 {
        s[..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

pub fn extract_loose_json_boolean_field(raw: &str, field_name: &str) -> Option<bool> {
    if raw.is_empty() {
        return None;
    }
    let escaped_field = escape_regex_field(field_name);
    let pattern_str = format!(r#""{}"\s*:\s*(true|false)"#, escaped_field);
    let re = Regex::new(&pattern_str).ok()?;
    let caps = re.captures(raw)?;
    let val = caps.get(1)?.as_str().to_lowercase();
    Some(val == "true")
}

pub fn extract_loose_json_number_field(raw: &str, field_name: &str) -> Option<f64> {
    if raw.is_empty() {
        return None;
    }
    let escaped_field = escape_regex_field(field_name);
    let pattern_str = format!(r#""{}"\s*:\s*(-?\d+(?:\.\d+)?)"#, escaped_field);
    let re = Regex::new(&pattern_str).ok()?;
    let caps = re.captures(raw)?;
    let val: f64 = caps.get(1)?.as_str().parse().ok()?;
    if val.is_finite() {
        Some(val)
    } else {
        None
    }
}

pub fn recover_malformed_tool_input(tool_name: &str, raw_args: &str) -> Option<Value> {
    if raw_args.is_empty() {
        return None;
    }

    if let Ok(parsed) = serde_json::from_str::<Value>(raw_args) {
        return Some(parsed);
    }

    match tool_name {
        "Write" => {
            let file_path = extract_loose_json_string_field(raw_args, "file_path", false);
            let content = extract_loose_json_string_field(raw_args, "content", true);
            match (file_path, content) {
                (Some(fp), Some(c)) => Some(serde_json::json!({
                    "file_path": fp,
                    "content": c
                })),
                _ => None,
            }
        }
        "Edit" => {
            let file_path = extract_loose_json_string_field(raw_args, "file_path", false);
            let old_string = extract_loose_json_string_field(raw_args, "old_string", false);
            let new_string = extract_loose_json_string_field(raw_args, "new_string", false);
            let replace_all = extract_loose_json_boolean_field(raw_args, "replace_all");
            match (file_path, old_string, new_string) {
                (Some(fp), Some(os), Some(ns)) => {
                    let mut obj = serde_json::json!({
                        "file_path": fp,
                        "old_string": os,
                        "new_string": ns
                    });
                    if let Some(ra) = replace_all {
                        obj["replace_all"] = Value::Bool(ra);
                    }
                    Some(obj)
                }
                _ => None,
            }
        }
        "MultiEdit" => {
            let file_path = extract_loose_json_string_field(raw_args, "file_path", false);
            if let Some(fp) = file_path {
                let obj = serde_json::json!({
                    "file_path": fp,
                    "edits": []
                });
                return Some(obj);
            }
            None
        }
        "Read" => {
            let file_path = extract_loose_json_string_field(raw_args, "file_path", false);
            let offset = extract_loose_json_number_field(raw_args, "offset");
            let limit = extract_loose_json_number_field(raw_args, "limit");
            match file_path {
                Some(fp) => {
                    let mut obj = serde_json::json!({ "file_path": fp });
                    if let Some(o) = offset {
                        obj["offset"] = serde_json::json!(o as u64);
                    }
                    if let Some(l) = limit {
                        obj["limit"] = serde_json::json!(l as u64);
                    }
                    Some(obj)
                }
                _ => None,
            }
        }
        "Bash" => {
            let command = extract_loose_json_string_field(raw_args, "command", true);
            let timeout = extract_loose_json_number_field(raw_args, "timeout");
            match command {
                Some(cmd) => {
                    let mut obj = serde_json::json!({ "command": cmd });
                    if let Some(t) = timeout {
                        obj["timeout"] = serde_json::json!(t as u64);
                    }
                    Some(obj)
                }
                _ => None,
            }
        }
        "Glob" => {
            let pattern = extract_loose_json_string_field(raw_args, "pattern", false);
            let path = extract_loose_json_string_field(raw_args, "path", false);
            match pattern {
                Some(p) => {
                    let mut obj = serde_json::json!({ "pattern": p });
                    if let Some(pt) = path {
                        obj["path"] = Value::String(pt);
                    }
                    Some(obj)
                }
                _ => None,
            }
        }
        "Grep" => {
            let pattern = extract_loose_json_string_field(raw_args, "pattern", false);
            let path = extract_loose_json_string_field(raw_args, "path", false);
            let include = extract_loose_json_string_field(raw_args, "include", false);
            match pattern {
                Some(p) => {
                    let mut obj = serde_json::json!({ "pattern": p });
                    if let Some(pt) = path {
                        obj["path"] = Value::String(pt);
                    }
                    if let Some(inc) = include {
                        obj["include"] = Value::String(inc);
                    }
                    Some(obj)
                }
                _ => None,
            }
        }
        "ListDir" => {
            let path = extract_loose_json_string_field(raw_args, "path", false);
            path.map(|p| serde_json::json!({ "path": p }))
        }
        "WebFetch" => {
            let url = extract_loose_json_string_field(raw_args, "url", false);
            match url {
                Some(u) => {
                    let mut obj = serde_json::json!({ "url": u });
                    let headers = extract_loose_json_string_field(raw_args, "headers", false);
                    if let Some(h) = headers {
                        if let Ok(parsed_headers) = serde_json::from_str::<Value>(&h) {
                            obj["headers"] = parsed_headers;
                        }
                    }
                    Some(obj)
                }
                _ => None,
            }
        }
        "WebSearch" => {
            let query = extract_loose_json_string_field(raw_args, "query", false);
            query.map(|q| serde_json::json!({ "query": q }))
        }
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct PendingToolCall {
    pub id: String,
    pub name: String,
    pub args: String,
    pub recovered_input: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct ToolCallAnalysis {
    pub tool_names: Vec<String>,
    pub all_empty: bool,
    pub has_malformed: bool,
}

pub fn merge_tool_args(current: &str, incoming: &str) -> String {
    if current.is_empty() {
        return incoming.to_string();
    }
    if incoming.is_empty() {
        return current.to_string();
    }
    if incoming.len() >= current.len() && incoming.starts_with(current) {
        return incoming.to_string();
    }
    if current.len() > incoming.len() && current.starts_with(incoming) {
        return current.to_string();
    }
    format!("{}{}", current, incoming)
}

pub fn analyze_pending_tool_calls(tool_calls: &HashMap<String, PendingToolCall>) -> ToolCallAnalysis {
    let mut tool_names = Vec::new();
    let mut all_empty = !tool_calls.is_empty();
    let mut has_malformed = false;

    for ptc in tool_calls.values() {
        let tool_name = &ptc.name;
        if !tool_name.is_empty() {
            tool_names.push(tool_name.clone());
        }

        let parsed_ok = serde_json::from_str::<Value>(&ptc.args).is_ok();
        let recovered = if !parsed_ok {
            recover_malformed_tool_input(tool_name, &ptc.args)
        } else {
            None
        };

        if !ptc.args.is_empty() {
            if let Ok(parsed) = serde_json::from_str::<Value>(&ptc.args) {
                if let Value::Object(map) = parsed {
                    if !map.is_empty() {
                        all_empty = false;
                    }
                }
            }
            if recovered
                .as_ref()
                .and_then(|v| v.as_object())
                .map(|m| !m.is_empty())
                .unwrap_or(false)
            {
                all_empty = false;
            }
        }

        if !ptc.args.is_empty() && !parsed_ok {
            has_malformed = true;
        }
    }

    ToolCallAnalysis {
        tool_names,
        all_empty,
        has_malformed,
    }
}

pub fn try_parse_tool_input(tool_name: &str, raw_args: &str) -> Value {
    if raw_args.is_empty() {
        return serde_json::json!({});
    }

    if let Ok(parsed) = serde_json::from_str::<Value>(raw_args) {
        return parsed;
    }

    if let Some(recovered) = recover_malformed_tool_input(tool_name, raw_args) {
        tracing::info!(
            module = "SSE",
            "Recovered malformed tool input for '{}' ({} bytes → {} fields)",
            tool_name,
            raw_args.len(),
            recovered.as_object().map(|m| m.len()).unwrap_or(0)
        );
        return recovered;
    }

    tracing::warn!(
        module = "SSE",
        "Failed to parse tool input for '{}' ({} bytes), preview: {}",
        tool_name,
        raw_args.len(),
        &raw_args[..raw_args.len().min(200)]
    );
    serde_json::json!({})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_sse_payloads_basic() {
        let buffer = "data: hello\n\ndata: world\n\n";
        let result = consume_sse_payloads(buffer);
        assert_eq!(result.payloads, vec!["hello", "world"]);
        assert_eq!(result.remainder, "");
    }

    #[test]
    fn test_consume_sse_payloads_with_remainder() {
        let buffer = "data: hello\n\ndata: partial";
        let result = consume_sse_payloads(buffer);
        assert_eq!(result.payloads, vec!["hello"]);
        assert_eq!(result.remainder, "data: partial");
    }

    #[test]
    fn test_consume_sse_payloads_multiline_data() {
        let buffer = "data: line1\ndata: line2\n\n";
        let result = consume_sse_payloads(buffer);
        assert_eq!(result.payloads, vec!["line1\nline2"]);
    }

    #[test]
    fn test_consume_sse_payloads_crlf() {
        let buffer = "data: hello\r\n\r\ndata: world\r\n\r\n";
        let result = consume_sse_payloads(buffer);
        assert_eq!(result.payloads, vec!["hello", "world"]);
    }

    #[test]
    fn test_consume_sse_payloads_done_marker() {
        let buffer = "data: [DONE]\n\n";
        let result = consume_sse_payloads(buffer);
        assert_eq!(result.payloads, vec!["[DONE]"]);
    }

    #[test]
    fn test_decode_loose_json_string_basic() {
        assert_eq!(decode_loose_json_string("hello"), "hello");
    }

    #[test]
    fn test_decode_loose_json_string_with_newlines() {
        assert_eq!(decode_loose_json_string("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn test_decode_loose_json_string_with_escapes() {
        assert_eq!(decode_loose_json_string("path\\nto\\rfile"), "path\nto\rfile");
    }

    #[test]
    fn test_extract_loose_json_string_field_basic() {
        let raw = r#"{"file_path": "/tmp/test.txt", "content": "hello"}"#;
        assert_eq!(
            extract_loose_json_string_field(raw, "file_path", false),
            Some("/tmp/test.txt".to_string())
        );
        assert_eq!(
            extract_loose_json_string_field(raw, "content", false),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_extract_loose_json_string_field_truncated() {
        let raw = r#"{"file_path": "/tmp/test.txt", "content": "some very long con"#;
        assert_eq!(
            extract_loose_json_string_field(raw, "file_path", false),
            Some("/tmp/test.txt".to_string())
        );
        assert_eq!(
            extract_loose_json_string_field(raw, "content", true),
            Some("some very long con".to_string())
        );
    }

    #[test]
    fn test_extract_loose_json_boolean_field() {
        let raw = r#"{"replace_all": true}"#;
        assert_eq!(extract_loose_json_boolean_field(raw, "replace_all"), Some(true));
        assert_eq!(extract_loose_json_boolean_field(raw, "missing"), None);
    }

    #[test]
    fn test_extract_loose_json_number_field() {
        let raw = r#"{"offset": 10, "limit": 50}"#;
        assert_eq!(extract_loose_json_number_field(raw, "offset"), Some(10.0));
        assert_eq!(extract_loose_json_number_field(raw, "limit"), Some(50.0));
    }

    #[test]
    fn test_recover_malformed_tool_input_write() {
        let raw = r#"{"file_path": "/tmp/test.txt", "content": "hello world"#;
        let result = recover_malformed_tool_input("Write", raw);
        assert!(result.is_some());
        let obj = result.unwrap();
        assert_eq!(obj["file_path"], "/tmp/test.txt");
        assert_eq!(obj["content"], "hello world");
    }

    #[test]
    fn test_recover_malformed_tool_input_edit() {
        let raw = r#"{"file_path": "/tmp/test.txt", "old_string": "foo", "new_string": "bar", "replace_all": true"#;
        let result = recover_malformed_tool_input("Edit", raw);
        assert!(result.is_some());
        let obj = result.unwrap();
        assert_eq!(obj["file_path"], "/tmp/test.txt");
        assert_eq!(obj["old_string"], "foo");
        assert_eq!(obj["new_string"], "bar");
        assert_eq!(obj["replace_all"], true);
    }

    #[test]
    fn test_recover_malformed_tool_input_bash() {
        let raw = r#"{"command": "ls -la /tmp"#;
        let result = recover_malformed_tool_input("Bash", raw);
        assert!(result.is_some());
        let obj = result.unwrap();
        assert_eq!(obj["command"], "ls -la /tmp");
    }

    #[test]
    fn test_recover_malformed_tool_input_valid_json() {
        let raw = r#"{"file_path": "/tmp/test.txt", "content": "hello"}"#;
        let result = recover_malformed_tool_input("Write", raw);
        assert!(result.is_some());
        let obj = result.unwrap();
        assert_eq!(obj["file_path"], "/tmp/test.txt");
    }

    #[test]
    fn test_recover_malformed_tool_input_unknown_tool() {
        let raw = r#"{"some": "data"}"#;
        let result = recover_malformed_tool_input("UnknownTool", raw);
        assert!(result.is_some());
    }

    #[test]
    fn test_try_parse_tool_input_valid() {
        let result = try_parse_tool_input("Write", r#"{"file_path": "/tmp/test.txt"}"#);
        assert_eq!(result["file_path"], "/tmp/test.txt");
    }

    #[test]
    fn test_try_parse_tool_input_malformed() {
        let result = try_parse_tool_input("Bash", r#"{"command": "ls -la"#);
        assert_eq!(result["command"], "ls -la");
    }

    #[test]
    fn test_try_parse_tool_input_empty() {
        let result = try_parse_tool_input("Bash", "");
        assert_eq!(result, serde_json::json!({}));
    }

    #[test]
    fn test_merge_tool_args() {
        assert_eq!(merge_tool_args("", "hello"), "hello");
        assert_eq!(merge_tool_args("hel", "hello"), "hello");
        assert_eq!(merge_tool_args("hello", "hel"), "hello");
        assert_eq!(merge_tool_args("abc", "def"), "abcdef");
    }

    #[test]
    fn test_strip_trailing_incomplete_escape() {
        assert_eq!(strip_trailing_incomplete_escape("hello"), "hello");
        assert_eq!(strip_trailing_incomplete_escape("hello\\"), "hello");
        assert_eq!(strip_trailing_incomplete_escape("hello\\\\"), "hello\\\\");
        assert_eq!(strip_trailing_incomplete_escape("hello\\\\\\"), "hello\\\\");
    }

    #[test]
    fn test_consume_sse_payloads_empty() {
        let result = consume_sse_payloads("");
        assert!(result.payloads.is_empty());
        assert_eq!(result.remainder, "");
    }

    #[test]
    fn test_extract_loose_json_string_field_with_special_chars() {
        let raw = r#"{"content": "line1\\nline2\\nline3"}"#;
        let result = extract_loose_json_string_field(raw, "content", false);
        assert!(result.is_some());
        let val = result.unwrap();
        assert!(val.contains("line1"));
        assert!(val.contains("line2"));
    }
}
