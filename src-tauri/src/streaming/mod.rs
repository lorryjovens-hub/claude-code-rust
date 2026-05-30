pub mod sse_parser;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

pub const SSE_IDLE_TIMEOUT_SECS: u64 = 300;
pub const SSE_MAX_DURATION_SECS: u64 = 1800;
pub const SSE_MAX_BUFFER_SIZE: usize = 10000;

pub use sse_parser::{
    consume_sse_payloads, merge_tool_args,
    try_parse_tool_input,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct ActiveStream {
    pub conversation_id: String,
    pub events: Vec<StreamEvent>,
    pub listeners: Vec<broadcast::Sender<StreamEvent>>,
    pub done: bool,
}

pub struct StreamManager {
    streams: HashMap<String, ActiveStream>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }

    pub fn create_stream(&mut self, conversation_id: &str) -> broadcast::Sender<StreamEvent> {
        let stream = ActiveStream {
            conversation_id: conversation_id.to_string(),
            events: Vec::new(),
            listeners: Vec::new(),
            done: false,
        };

        let (tx, _rx) = broadcast::channel(SSE_MAX_BUFFER_SIZE);
        let mut stream_entry = stream;
        stream_entry.listeners.push(tx.clone());
        self.streams.insert(conversation_id.to_string(), stream_entry);

        tx
    }

    pub fn broadcast(&mut self, conversation_id: &str, event: StreamEvent) {
        if let Some(stream) = self.streams.get_mut(conversation_id) {
            if stream.events.len() >= SSE_MAX_BUFFER_SIZE {
                let removed = stream.events.len() - SSE_MAX_BUFFER_SIZE / 2;
                stream.events.drain(..removed);
            }

            stream.events.push(event.clone());

            for listener in &stream.listeners {
                let _ = listener.send(event.clone());
            }
        }
    }

    pub fn end_stream(&mut self, conversation_id: &str) {
        if let Some(stream) = self.streams.get_mut(conversation_id) {
            stream.done = true;

            let done_event = StreamEvent {
                event_type: "stream_done".to_string(),
                data: serde_json::json!({}),
                timestamp: chrono::Utc::now().timestamp_millis(),
            };

            for listener in &stream.listeners {
                let _ = listener.send(done_event.clone());
            }
        }
    }

    pub fn add_listener(&mut self, conversation_id: &str) -> Option<broadcast::Receiver<StreamEvent>> {
        self.streams.get_mut(conversation_id).map(|stream| {
            let (tx, rx) = broadcast::channel(SSE_MAX_BUFFER_SIZE);
            stream.listeners.push(tx);
            rx
        })
    }

    pub fn remove_listener(&mut self, conversation_id: &str, sender: &broadcast::Sender<StreamEvent>) {
        if let Some(stream) = self.streams.get_mut(conversation_id) {
            stream.listeners.retain(|s| !std::ptr::eq(s, sender));
        }
    }

    pub fn get_events(&self, conversation_id: &str) -> Option<Vec<StreamEvent>> {
        self.streams.get(conversation_id).map(|stream| stream.events.clone())
    }

    pub fn is_done(&self, conversation_id: &str) -> bool {
        self.streams.get(conversation_id).map(|s| s.done).unwrap_or(true)
    }

    pub fn cleanup(&mut self, conversation_id: &str) {
        self.streams.remove(conversation_id);
    }

    pub fn cleanup_done_streams(&mut self) {
        self.streams.retain(|_id, stream| !stream.done || !stream.listeners.is_empty());
    }

    pub fn list_active_streams(&self) -> Vec<String> {
        self.streams.iter()
            .filter(|(_, s)| !s.done)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn has_listeners(&self, conversation_id: &str) -> bool {
        self.streams.get(conversation_id)
            .map(|s| !s.listeners.is_empty() && !s.done)
            .unwrap_or(false)
    }

    pub fn is_active(&self, conversation_id: &str) -> bool {
        self.streams.get(conversation_id)
            .map(|s| !s.done)
            .unwrap_or(false)
    }

    pub fn get_historical_events(&self, conversation_id: &str) -> Vec<StreamEvent> {
        self.streams.get(conversation_id)
            .map(|s| s.events.clone())
            .unwrap_or_default()
    }

    pub fn add_listener_with_replay(&mut self, conversation_id: &str) -> Option<(broadcast::Receiver<StreamEvent>, Vec<StreamEvent>)> {
        self.streams.get_mut(conversation_id).map(|stream| {
            let (tx, rx) = broadcast::channel(SSE_MAX_BUFFER_SIZE);
            stream.listeners.push(tx);
            let history = stream.events.clone();
            (rx, history)
        })
    }

    pub fn remove_all_listeners(&mut self, conversation_id: &str) {
        if let Some(stream) = self.streams.get_mut(conversation_id) {
            stream.listeners.clear();
        }
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_sse_event(event_type: &str, data: serde_json::Value) -> String {
    format!("event: {}\ndata: {}\n\n", event_type, data.to_string())
}

pub fn create_sse_data(data: serde_json::Value) -> String {
    format!("data: {}\n\n", data.to_string())
}
