//! 消息分组算法
//! 
//! 按主题、时间、工具调用多维度分组

use serde::{Deserialize, Serialize};

/// 消息分组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageGroup {
    pub id: String,
    pub messages: Vec<String>,
    pub topic: Option<String>,
    pub tool_calls: Vec<String>,
    pub timestamp: i64,
}

/// 按多维度分组消息
pub fn group_messages(messages: &[String]) -> Vec<MessageGroup> {
    if messages.is_empty() {
        return Vec::new();
    }

    let mut groups: Vec<MessageGroup> = Vec::new();
    let mut current_group = MessageGroup {
        id: uuid::Uuid::new_v4().to_string(),
        messages: Vec::new(),
        topic: None,
        tool_calls: Vec::new(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    for message in messages {
        let is_boundary = is_group_boundary(message);
        
        if is_boundary && !current_group.messages.is_empty() {
            groups.push(current_group.clone());
            current_group = MessageGroup {
                id: uuid::Uuid::new_v4().to_string(),
                messages: Vec::new(),
                topic: extract_topic(message),
                tool_calls: Vec::new(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            };
        }

        current_group.messages.push(message.clone());
        
        if let Some(tool) = extract_tool_call(message) {
            current_group.tool_calls.push(tool);
        }
    }

    if !current_group.messages.is_empty() {
        groups.push(current_group);
    }

    groups
}

fn is_group_boundary(message: &str) -> bool {
    let boundary_markers = ["## ", "# ", "User:", "Assistant:", "---"];
    boundary_markers.iter().any(|marker| message.starts_with(marker))
}

fn extract_topic(message: &str) -> Option<String> {
    if message.starts_with("# ") {
        Some(message.trim_start_matches('#').trim().to_string())
    } else if message.starts_with("## ") {
        Some(message.trim_start_matches(|c| c == '#').trim().to_string())
    } else {
        None
    }
}

fn extract_tool_call(message: &str) -> Option<String> {
    if message.contains("tool_use") || message.contains("tool_call") {
        let start = message.find("name=\"")? + 6;
        let end = message[start..].find("\"")?;
        Some(message[start..start + end].to_string())
    } else {
        None
    }
}

/// 合并相似分组
pub fn merge_similar_groups(groups: Vec<MessageGroup>) -> Vec<MessageGroup> {
    if groups.len() <= 1 {
        return groups;
    }

    let mut merged: Vec<MessageGroup> = Vec::new();
    let mut current = groups[0].clone();

    for group in groups.into_iter().skip(1) {
        if should_merge(&current, &group) {
            current.messages.extend(group.messages);
            current.tool_calls.extend(group.tool_calls);
        } else {
            merged.push(current);
            current = group;
        }
    }

    merged.push(current);
    merged
}

fn should_merge(a: &MessageGroup, b: &MessageGroup) -> bool {
    if a.topic.is_some() && b.topic.is_some() {
        return a.topic == b.topic;
    }
    
    let tool_overlap = a.tool_calls.iter()
        .filter(|t| b.tool_calls.contains(t))
        .count();
    
    tool_overlap > 0 && tool_overlap == a.tool_calls.len().min(b.tool_calls.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_messages_empty() {
        let groups = group_messages(&[]);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_group_messages_single() {
        let messages = vec!["Hello world".to_string()];
        let groups = group_messages(&messages);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].messages.len(), 1);
    }

    #[test]
    fn test_group_messages_with_boundaries() {
        let messages = vec![
            "# Topic 1".to_string(),
            "Message 1".to_string(),
            "Message 2".to_string(),
            "# Topic 2".to_string(),
            "Message 3".to_string(),
        ];
        let groups = group_messages(&messages);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_extract_topic() {
        let topic = extract_topic("# My Topic");
        assert_eq!(topic, Some("My Topic".to_string()));
    }

    #[test]
    fn test_merge_similar_groups() {
        let groups = vec![
            MessageGroup {
                id: "1".to_string(),
                messages: vec!["a".to_string()],
                topic: Some("topic".to_string()),
                tool_calls: vec!["read".to_string()],
                timestamp: 0,
            },
            MessageGroup {
                id: "2".to_string(),
                messages: vec!["b".to_string()],
                topic: Some("topic".to_string()),
                tool_calls: vec!["read".to_string()],
                timestamp: 0,
            },
        ];

        let merged = merge_similar_groups(groups);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].messages.len(), 2);
    }
}
