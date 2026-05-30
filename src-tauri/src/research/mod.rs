use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRequest {
    pub query: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    pub title: String,
    pub sub_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSource {
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentResult {
    pub sub_question: String,
    pub findings: String,
    pub sources: Vec<ResearchSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub plan: ResearchPlan,
    pub sub_results: Vec<SubAgentResult>,
    pub sources: Vec<ResearchSource>,
    pub report: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResearchEvent {
    ResearchPhase { phase: String, label: String },
    ResearchPlan { title: String, sub_questions: Vec<String> },
    ResearchSubagentStarted { sub_agent_id: String, index: usize, sub_question: String },
    ResearchSource { sub_agent_id: String, source: ResearchSource },
    ResearchFinding { sub_agent_id: String, sub_question: String, markdown: String },
    ResearchSubagentDone { sub_agent_id: String, sources_count: usize },
    ResearchReportDelta { text: String },
    ResearchReport { markdown: String },
    ResearchDone { sources_count: usize, sub_agents_count: usize, duration_ms: u64 },
    ResearchError { error: String },
}

const MAX_SUB_QUESTIONS: usize = 5;
const MAX_WEB_SEARCHES_PER_SUBAGENT: u32 = 8;
const SUB_RESEARCHER_MAX_TOKENS: u32 = 8192;
const SYNTHESIS_MAX_TOKENS: u32 = 32768;

const PLANNING_SYSTEM_PROMPT: &str = r#"You are a research planner. Your job is to decompose a user's research question into a structured research plan.

Given a research question, you must:
1. Identify the core subject and scope
2. Break it into 3-5 focused, non-overlapping sub-questions that together cover the full scope
3. Each sub-question should be specific enough to be answerable by 3-5 high-quality web sources
4. Order sub-questions logically (foundational context first, specifics later)
5. Generate a concise title for the final research report

IMPORTANT:
- Keep each sub-question to ONE concise sentence (max 30 words)
- The title must be under 15 words
- Use English for JSON keys and structure; sub-question text should match the language of the user's query

Respond in strict JSON with this shape:
{
  "title": "string",
  "sub_questions": ["string", "string", ...]
}

Do NOT include markdown code fences. Output only the raw JSON object."#;

const SUB_RESEARCHER_SYSTEM_PROMPT: &str = r#"You are a research specialist. You will be given ONE specific sub-question as part of a larger research project. Your job:

1. Use the web_search tool aggressively to find 3-8 high-quality, recent, authoritative sources
2. Prefer primary sources, academic papers, official documentation, and reputable publications
3. After gathering sources, write a detailed markdown-formatted findings report on this sub-question ONLY
4. Include inline markdown hyperlinks to each source you cite: ([source name](url))
5. Use markdown tables for structured data or comparisons
6. Be precise with facts, numbers, dates, and technical details
7. Do NOT write an introduction or conclusion — just the findings body
8. Aim for 500-1500 words of dense, well-cited content

IMPORTANT:
- Stay strictly focused on your assigned sub-question
- If sources disagree, note the disagreement and cite both
- Prefer quantitative data when available
- Do not speculate beyond what the sources say"#;

const SYNTHESIS_SYSTEM_PROMPT: &str = r#"You are a senior research writer. You will receive:
1. The original research question
2. A collection of findings from multiple sub-researchers, each covering a different sub-question
3. The list of all sources gathered

Your job is to synthesize these findings into a single, comprehensive, long-form research report. Requirements:

1. Write a clear H1 title
2. Open with an executive summary (2-3 paragraphs) stating the main conclusions
3. Organize the body into logical ## sections (not necessarily mirroring the sub-questions — reorganize as needed for coherence)
4. Use ### sub-sections where helpful
5. Use markdown tables for any structured comparisons or data
6. Include inline citations as markdown hyperlinks: ([source name](url))
7. Every non-trivial factual claim MUST have an inline citation
8. End with a ## Conclusion section summarizing key takeaways and open questions
9. End with a ## References section listing all cited sources as a numbered markdown list with hyperlinks

Style:
- Objective, analytical tone
- Precise with numbers, dates, and technical terminology
- Acknowledge uncertainty and disagreements between sources
- Do not speculate beyond what the findings support
- Target 2000-4000 words for a substantive report
- Prefer prose over bullet lists for the main body

Do NOT wrap the output in markdown code fences. Output the report directly as markdown."#;

#[derive(Clone)]
pub struct ResearchOrchestrator {
    http_client: reqwest::Client,
}

impl ResearchOrchestrator {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self { http_client }
    }

    pub async fn run_pipeline(
        &self,
        request: ResearchRequest,
        event_tx: mpsc::UnboundedSender<ResearchEvent>,
    ) -> Result<ResearchResult> {
        let started_at = std::time::Instant::now();

        let emit = |event: ResearchEvent| {
            let _ = event_tx.send(event);
        };

        // Phase 1: Planning
        emit(ResearchEvent::ResearchPhase {
            phase: "planning".to_string(),
            label: "Planning research...".to_string(),
        });

        let plan = self.run_planner(&request).await?;
        emit(ResearchEvent::ResearchPlan {
            title: plan.title.clone(),
            sub_questions: plan.sub_questions.clone(),
        });

        // Phase 2: Parallel sub-researchers
        let sub_count = plan.sub_questions.len();
        emit(ResearchEvent::ResearchPhase {
            phase: "gathering".to_string(),
            label: format!("Researching {} sub-topics in parallel...", sub_count),
        });

        let mut sub_results = Vec::new();
        let mut handles: Vec<tokio::task::JoinHandle<Result<SubAgentResult, anyhow::Error>>> = Vec::new();
        let sub_questions: Vec<String> = plan.sub_questions.clone();

        for (idx, sub_question) in sub_questions.into_iter().enumerate() {
            let sub_agent_id = format!("sub-{}", idx);
            let request_clone = request.clone();
            let event_tx_clone = event_tx.clone();
            let http_client = self.http_client.clone();
            let sub_question_clone = sub_question.clone();

            emit(ResearchEvent::ResearchSubagentStarted {
                sub_agent_id: sub_agent_id.clone(),
                index: idx,
                sub_question: sub_question_clone,
            });

            let handle = tokio::spawn(async move {
                let sub_q = sub_question.clone();
                let orchestrator = ResearchOrchestrator::new(http_client);
                match orchestrator.run_sub_researcher(&request_clone, sub_question).await {
                    Ok((findings, sources)) => {
                        for source in &sources {
                            let _ = event_tx_clone.send(ResearchEvent::ResearchSource {
                                sub_agent_id: sub_agent_id.clone(),
                                source: source.clone(),
                            });
                        }
                        let _ = event_tx_clone.send(ResearchEvent::ResearchFinding {
                            sub_agent_id: sub_agent_id.clone(),
                            sub_question: sub_q.clone(),
                            markdown: findings.clone(),
                        });
                        let _ = event_tx_clone.send(ResearchEvent::ResearchSubagentDone {
                            sub_agent_id: sub_agent_id.clone(),
                            sources_count: sources.len(),
                        });
                        Ok(SubAgentResult {
                            sub_question: sub_q.clone(),
                            findings,
                            sources,
                        })
                    }
                    Err(e) => {
                        tracing::error!(module = "Research", "Sub-agent {} failed: {}", sub_agent_id, e);
                        let _ = event_tx_clone.send(ResearchEvent::ResearchSubagentDone {
                            sub_agent_id: sub_agent_id.clone(),
                            sources_count: 0,
                        });
                        Ok(SubAgentResult {
                            sub_question: sub_q.clone(),
                            findings: String::new(),
                            sources: Vec::new(),
                        })
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Ok(result) = handle.await? {
                sub_results.push(result);
            }
        }

        // Aggregate and dedupe sources
        let mut all_sources_map: HashMap<String, ResearchSource> = HashMap::new();
        for r in &sub_results {
            for s in &r.sources {
                all_sources_map.entry(s.url.clone()).or_insert_with(|| s.clone());
            }
        }
        let all_sources: Vec<ResearchSource> = all_sources_map.into_values().collect();

        // Phase 3: Synthesis
        emit(ResearchEvent::ResearchPhase {
            phase: "writing".to_string(),
            label: "Writing final report...".to_string(),
        });

        let findings_blocks: Vec<(String, String)> = sub_results
            .iter()
            .filter(|r| !r.findings.trim().is_empty())
            .map(|r| (r.sub_question.clone(), r.findings.clone()))
            .collect();

        let final_report = self.run_synthesis(
            &request,
            &request.query,
            &findings_blocks,
            &all_sources,
            event_tx.clone(),
        ).await?;

        let duration_ms = started_at.elapsed().as_millis() as u64;

        emit(ResearchEvent::ResearchReport {
            markdown: final_report.clone(),
        });
        emit(ResearchEvent::ResearchDone {
            sources_count: all_sources.len(),
            sub_agents_count: sub_count,
            duration_ms,
        });

        Ok(ResearchResult {
            plan,
            sub_results,
            sources: all_sources,
            report: final_report,
        })
    }

    async fn run_planner(&self, request: &ResearchRequest) -> Result<ResearchPlan> {
        let today = chrono::Utc::now().format("%Y-%m-%d");
        let user_prompt = format!(
            r#"Research question: "{}"

Today's date: {}

Produce the research plan now."#,
            request.query, today
        );

        let body = serde_json::json!({
            "model": request.model,
            "max_tokens": 4096,
            "system": PLANNING_SYSTEM_PROMPT,
            "messages": [{
                "role": "user",
                "content": user_prompt
            }]
        });

        let response = self.call_anthropic(request, &body).await?;
        let content = response["content"].as_array().ok_or_else(|| anyhow::anyhow!("Planner returned no content"))?;
        let text_block = content.iter().find(|b| b["type"] == "text").ok_or_else(|| anyhow::anyhow!("Planner returned no text content"))?;
        let raw_text = text_block["text"].as_str().ok_or_else(|| anyhow::anyhow!("Planner text is invalid"))?;

        let cleaned = raw_text
            .replace("```json", "")
            .replace("```", "")
            .trim()
            .to_string();

        let parsed: serde_json::Value = serde_json::from_str(&cleaned)
            .map_err(|e| anyhow::anyhow!("Planner returned invalid JSON: {}", e))?;

        let title = parsed["title"].as_str().unwrap_or(&request.query).to_string();
        let sub_questions: Vec<String> = parsed["sub_questions"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Planner returned no sub_questions"))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .take(MAX_SUB_QUESTIONS)
            .collect();

        if sub_questions.is_empty() {
            return Err(anyhow::anyhow!("Planner returned no sub_questions"));
        }

        Ok(ResearchPlan { title, sub_questions })
    }

    async fn run_sub_researcher(
        &self,
        request: &ResearchRequest,
        sub_question: String,
    ) -> Result<(String, Vec<ResearchSource>)> {
        let today = chrono::Utc::now().format("%Y-%m-%d");
        let user_prompt = format!(
            r#"Main research topic: "{}"

YOUR ASSIGNED SUB-QUESTION: "{}"

Today's date: {}

Research this sub-question now. Use web_search to find high-quality sources, then write your markdown findings report. Cite every factual claim with an inline markdown link."#,
            request.query, sub_question, today
        );

        let body = serde_json::json!({
            "model": request.model,
            "max_tokens": SUB_RESEARCHER_MAX_TOKENS,
            "system": SUB_RESEARCHER_SYSTEM_PROMPT,
            "messages": [{
                "role": "user",
                "content": user_prompt
            }],
            "tools": [{
                "type": "web_search_20250305",
                "name": "web_search",
                "max_uses": MAX_WEB_SEARCHES_PER_SUBAGENT,
            }]
        });

        let response = self.call_anthropic(request, &body).await?;
        let (text, sources) = self.extract_sub_agent_result(&response);
        Ok((text, sources))
    }

    async fn run_synthesis(
        &self,
        request: &ResearchRequest,
        query: &str,
        findings_blocks: &[(String, String)],
        all_sources: &[ResearchSource],
        event_tx: mpsc::UnboundedSender<ResearchEvent>,
    ) -> Result<String> {
        let today = chrono::Utc::now().format("%Y-%m-%d");

        let sources_list = all_sources
            .iter()
            .enumerate()
            .map(|(i, s)| format!("[{}] {} — {}", i + 1, s.title, s.url))
            .collect::<Vec<_>>()
            .join("\n");

        let findings_text = findings_blocks
            .iter()
            .enumerate()
            .map(|(i, (sub_q, markdown))| {
                format!("### Sub-research {}: {}\n\n{}", i + 1, sub_q, markdown)
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let user_prompt = format!(
            r#"Original research question: "{}"

Today's date: {}

## All Sources Gathered ({} total)
{}

## Findings from Sub-Researchers

{}

---

Now synthesize all of the above into a comprehensive research report following the structure and style described in the system prompt."#,
            query, today, all_sources.len(), sources_list, findings_text
        );

        let body = serde_json::json!({
            "model": request.model,
            "max_tokens": SYNTHESIS_MAX_TOKENS,
            "system": SYNTHESIS_SYSTEM_PROMPT,
            "messages": [{
                "role": "user",
                "content": user_prompt
            }],
            "stream": true
        });

        self.call_anthropic_streaming(request, &body, event_tx).await
    }

    async fn call_anthropic(
        &self,
        request: &ResearchRequest,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let endpoint = self.normalize_endpoint(&request.base_url);

        let response = self.http_client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .header("x-api-key", &request.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "web-search-20250305")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error {}: {}", status, &err_text[..err_text.len().min(500)]));
        }

        let json: serde_json::Value = response.json().await?;
        Ok(json)
    }

    async fn call_anthropic_streaming(
        &self,
        request: &ResearchRequest,
        body: &serde_json::Value,
        event_tx: mpsc::UnboundedSender<ResearchEvent>,
    ) -> Result<String> {
        let endpoint = self.normalize_endpoint(&request.base_url);

        let response = self.http_client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .header("x-api-key", &request.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "web-search-20250305")
            .header("accept", "text/event-stream")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error {}: {}", status, &err_text[..err_text.len().min(500)]));
        }

        let mut stream = response.bytes_stream();
        use futures::StreamExt;
        let mut buf = String::new();
        let mut full_text = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buf.push_str(&String::from_utf8_lossy(&chunk));

            let lines: Vec<String> = buf.split('\n').map(String::from).collect();
            buf = lines.last().cloned().unwrap_or_default();

            for line in &lines[..lines.len() - 1] {
                if !line.starts_with("data: ") {
                    continue;
                }
                let payload = line[6..].trim();
                if payload.is_empty() {
                    continue;
                }

                if let Ok(evt) = serde_json::from_str::<serde_json::Value>(payload) {
                    if evt["type"] == "content_block_delta" {
                        if let Some(text) = evt["delta"]["text"].as_str() {
                            full_text.push_str(text);
                            let _ = event_tx.send(ResearchEvent::ResearchReportDelta {
                                text: text.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(full_text)
    }

    fn extract_sub_agent_result(&self, response: &serde_json::Value) -> (String, Vec<ResearchSource>) {
        let mut sources = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();
        let mut text = String::new();

        if let Some(content) = response["content"].as_array() {
            for block in content {
                if block["type"] == "text" {
                    if let Some(t) = block["text"].as_str() {
                        text.push_str(t);
                    }
                } else if block["type"] == "web_search_tool_result" {
                    if let Some(items) = block["content"].as_array() {
                        for item in items {
                            if item["type"] == "web_search_result" {
                                if let Some(url) = item["url"].as_str() {
                                    if !seen_urls.contains(url) {
                                        seen_urls.insert(url.to_string());
                                        sources.push(ResearchSource {
                                            url: url.to_string(),
                                            title: item["title"].as_str().unwrap_or(url).to_string(),
                                            snippet: item["page_age"].as_str().map(|age| format!("Last updated: {}", age)),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        (text, sources)
    }

    fn normalize_endpoint(&self, base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');
        if base.ends_with("/v1") {
            format!("{}/messages", base)
        } else {
            format!("{}/v1/messages", base)
        }
    }
}
