use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserQuestion {
    pub question: String,
    pub description: Option<String>,
    pub options: Vec<QuestionOption>,
    pub multi_select: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserAnswer {
    pub selected_options: Vec<usize>,
    pub custom_input: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PendingQuestion {
    pub question: AskUserQuestion,
    pub request_id: String,
    pub tool_use_id: String,
    pub original_input: serde_json::Value,
    pub answered: bool,
    pub answer: Option<AskUserAnswer>,
}

pub struct AskUserManager {
    pending_questions: Arc<RwLock<HashMap<String, PendingQuestion>>>,
    answer_notifications: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<AskUserAnswer>>>>,
}

impl AskUserManager {
    pub fn new() -> Self {
        Self {
            pending_questions: Arc::new(RwLock::new(HashMap::new())),
            answer_notifications: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_pending_question(
        &self,
        conversation_id: String,
        question: AskUserQuestion,
        request_id: String,
        tool_use_id: String,
        original_input: serde_json::Value,
    ) {
        let pending = PendingQuestion {
            question,
            request_id: request_id.clone(),
            tool_use_id,
            original_input,
            answered: false,
            answer: None,
        };

        self.pending_questions
            .write()
            .await
            .insert(conversation_id, pending);
    }

    pub async fn get_pending_question(
        &self,
        conversation_id: &str,
    ) -> Option<PendingQuestion> {
        self.pending_questions.read().await.get(conversation_id).cloned()
    }

    pub async fn submit_answer(
        &self,
        conversation_id: &str,
        answer: AskUserAnswer,
    ) -> Option<serde_json::Value> {
        let mut questions = self.pending_questions.write().await;
        if let Some(pending) = questions.get_mut(conversation_id) {
            pending.answered = true;
            pending.answer = Some(answer.clone());

            let mut merged_input = pending.original_input.clone();
            if let Some(input_map) = merged_input.as_object_mut() {
                let answers_json = serde_json::json!({
                    "selected_options": answer.selected_options,
                    "custom_input": answer.custom_input,
                });
                input_map.insert("answers".to_string(), answers_json);
            }

            Some(merged_input)
        } else {
            None
        }
    }

    pub async fn remove_pending_question(&self, conversation_id: &str) {
        self.pending_questions.write().await.remove(conversation_id);
    }

    pub async fn register_answer_waiter(
        &self,
        conversation_id: String,
        tx: tokio::sync::oneshot::Sender<AskUserAnswer>,
    ) {
        self.answer_notifications
            .lock()
            .await
            .insert(conversation_id, tx);
    }

    pub async fn notify_answer_received(&self, conversation_id: &str, answer: AskUserAnswer) {
        let mut notifications = self.answer_notifications.lock().await;
        if let Some(tx) = notifications.remove(conversation_id) {
            let _ = tx.send(answer);
        }
    }

    pub async fn wait_for_answer(
        &self,
        conversation_id: &str,
    ) -> Option<AskUserAnswer> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.register_answer_waiter(conversation_id.to_string(), tx).await;
        rx.await.ok()
    }
}
