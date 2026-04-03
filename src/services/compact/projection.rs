//! 投影压缩算法
//! 
//! 关键信息提取与摘要生成

use serde::{Deserialize, Serialize};

/// 投影结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedContent {
    pub essential: String,
    pub key_points: Vec<String>,
    pub tool_calls: Vec<ToolCallSummary>,
    pub token_count: usize,
}

/// 工具调用摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallSummary {
    pub tool_name: String,
    pub input_summary: String,
    pub result_summary: String,
}

/// 投影到核心内容
pub fn project_to_essential(messages: &[String]) -> String {
    if messages.is_empty() {
        return String::new();
    }

    let mut essential_parts: Vec<String> = Vec::new();

    let key_points = extract_key_points(messages);
    if !key_points.is_empty() {
        essential_parts.push("### Key Points".to_string());
        for (i, point) in key_points.iter().enumerate() {
            essential_parts.push(format!("{}. {}", i + 1, point));
        }
    }

    let tool_summaries = extract_tool_summaries(messages);
    if !tool_summaries.is_empty() {
        essential_parts.push(String::new());
        essential_parts.push("### Tool Calls".to_string());
        for tool in &tool_summaries {
            essential_parts.push(format!("- {}: {}", tool.tool_name, tool.result_summary));
        }
    }

    let decisions = extract_decisions(messages);
    if !decisions.is_empty() {
        essential_parts.push(String::new());
        essential_parts.push("### Decisions".to_string());
        for decision in &decisions {
            essential_parts.push(format!("- {}", decision));
        }
    }

    essential_parts.join("\n")
}

fn extract_key_points(messages: &[String]) -> Vec<String> {
    let mut points = Vec::new();
    
    for message in messages {
        let lines: Vec<&str> = message.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let point = trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == ' ');
                if !point.is_empty() && point.len() > 10 {
                    points.push(point.to_string());
                }
            }
        }
    }

    points.into_iter().take(5).collect()
}

fn extract_tool_summaries(messages: &[String]) -> Vec<ToolCallSummary> {
    let mut summaries = Vec::new();

    for message in messages {
        if message.contains("tool_use") || message.contains("tool_result") {
            if let Some(summary) = parse_tool_summary(message) {
                summaries.push(summary);
            }
        }
    }

    summaries
}

fn parse_tool_summary(message: &str) -> Option<ToolCallSummary> {
    let tool_name = extract_tool_name(message)?;
    
    let result_summary = if message.contains("success") || message.contains("completed") {
        "Success".to_string()
    } else if message.contains("error") || message.contains("failed") {
        "Failed".to_string()
    } else {
        "Executed".to_string()
    };

    Some(ToolCallSummary {
        tool_name,
        input_summary: String::new(),
        result_summary,
    })
}

fn extract_tool_name(message: &str) -> Option<String> {
    let patterns = ["Read", "Write", "Edit", "Bash", "Glob", "Grep"];
    
    for pattern in patterns {
        if message.contains(pattern) {
            return Some(pattern.to_string());
        }
    }
    
    None
}

fn extract_decisions(messages: &[String]) -> Vec<String> {
    let mut decisions = Vec::new();
    
    let decision_keywords = ["decided", "chose", "selected", "will use", "going to"];
    
    for message in messages {
        let lower = message.to_lowercase();
        for keyword in &decision_keywords {
            if lower.contains(keyword) {
                if let Some(sentence) = find_sentence_with_keyword(message, keyword) {
                    decisions.push(sentence);
                    break;
                }
            }
        }
    }

    decisions.into_iter().take(3).collect()
}

fn find_sentence_with_keyword(text: &str, keyword: &str) -> Option<String> {
    let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();
    
    for sentence in sentences {
        if sentence.to_lowercase().contains(keyword) {
            return Some(sentence.trim().to_string());
        }
    }
    
    None
}

/// 估算 token 数
pub fn estimate_tokens(text: &str) -> usize {
    text.split_whitespace().count() / 4 * 3 + text.len() / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_empty() {
        let result = project_to_essential(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_project_with_key_points() {
        let messages = vec![
            "Here are some points:\n- First important point\n- Second key point".to_string(),
        ];
        let result = project_to_essential(&messages);
        assert!(result.contains("Key Points"));
        assert!(result.contains("First important point"));
    }

    #[test]
    fn test_project_with_tools() {
        let messages = vec![
            "Used Read tool successfully".to_string(),
            "Used Write tool with error".to_string(),
        ];
        let result = project_to_essential(&messages);
        assert!(result.contains("Tool Calls"));
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "This is a test message with multiple words";
        let tokens = estimate_tokens(text);
        assert!(tokens > 0);
    }
}
