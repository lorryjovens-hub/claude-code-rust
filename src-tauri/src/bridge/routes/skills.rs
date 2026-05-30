use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;

use super::super::AppState;
use crate::skills::{Skill, SkillsManager, SkillExecutionContext, SuperpowersCategory};

async fn skills_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.load_skills().await {
        Ok(skills) => Json(serde_json::json!({ "skills": skills })),
        Err(e) => Json(serde_json::json!({ "skills": [], "error": format!("{}", e) })),
    }
}

async fn skills_create(State(state): State<AppState>, Json(skill): Json<Skill>) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.create_skill(&skill.name, &skill.description, &skill.content.unwrap_or_default()) {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn skill_get(Path(name): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.get_skill_by_id(&name).await {
        Ok(Some(skill)) => Json(serde_json::to_value(skill).unwrap_or_default()),
        Ok(None) => Json(serde_json::json!({ "error": "Skill not found" })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn skill_update(Path(name): Path<String>, State(state): State<AppState>, Json(updates): Json<HashMap<String, serde_json::Value>>) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.update_skill(&name, updates) {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn skill_delete(Path(name): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.delete_skill(&name) {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct SkillEnableRequest {
    pub enabled: bool,
}

async fn skill_enable(Path(name): Path<String>, State(state): State<AppState>, Json(_req): Json<SkillEnableRequest>) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.toggle_skill(&name).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct SkillExecuteRequest {
    pub input: String,
    pub conversation_id: Option<String>,
    pub workspace_path: Option<String>,
    pub variables: Option<serde_json::Map<String, serde_json::Value>>,
}

async fn skill_execute(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<SkillExecuteRequest>,
) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let mcp_server_manager = state.mcp_manager.clone();

    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;

    let input = req.input.clone();

    let mut context = SkillExecutionContext::default();
    context.current_input = input.clone();
    context.conversation_id = req.conversation_id.unwrap_or_default();
    context.workspace_path = req.workspace_path;

    if let Some(vars) = req.variables {
        for (key, value) in vars {
            if let Some(s) = value.as_str() {
                context.variables.insert(key, s.to_string());
            }
        }
    }

    context = context.with_mcp_manager(mcp_server_manager.clone());

    let mcp_tools = mcp_server_manager.get_all_tools().await;
    context.available_mcp_tools = mcp_tools;

    match manager.execute_skill(&name, &input, Some(context)).await {
        Ok(result) => Json(serde_json::json!({ "success": true, "result": result })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct SkillMatchRequest {
    pub input: String,
}

async fn skills_match(State(state): State<AppState>, Json(req): Json<SkillMatchRequest>) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.execute_skill("match", &req.input, None).await {
        Ok(result) => Json(serde_json::json!({ "matched": true, "result": result })),
        Err(_) => Json(serde_json::json!({ "matched": false })),
    }
}

async fn design_skills_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    tracing::info!(module = "Bridge::design_skills_list", "➡️ 收到请求 /api/skills/design");
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    let start = std::time::Instant::now();
    match manager.load_skills().await {
        Ok(skills) => {
            let total = skills.len();
            let design_skills: Vec<_> = skills
                .into_iter()
                .filter(|s| s.od_metadata.is_some())
                .collect();
            let elapsed = start.elapsed();
            tracing::info!(
                module = "Bridge::design_skills_list",
                "✅ 返回 {} 个设计技能 / {} 个总技能 ({}ms)",
                design_skills.len(), total,
                elapsed.as_millis()
            );
            if design_skills.is_empty() {
                tracing::warn!(
                    module = "Bridge::design_skills_list",
                    "⚠️ 没有找到任何包含 od_metadata 的技能（共扫描 {} 个 SKILL.md）",
                    total
                );
            }
            Json(serde_json::json!({ "skills": design_skills }))
        }
        Err(e) => {
            tracing::error!(module = "Bridge::design_skills_list", "❌ load_skills 失败: {}", e);
            Json(serde_json::json!({ "skills": [], "error": format!("{}", e) }))
        }
    }
}

async fn design_skill_detail(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    tracing::info!(module = "Bridge::design_skill_detail", "➡️ 收到请求 /api/skills/design/{}", id);
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.load_skills().await {
        Ok(skills) => {
            if let Some(skill) = skills.into_iter().find(|s| s.id == id) {
                tracing::info!(
                    module = "Bridge::design_skill_detail",
                    "✅ 找到技能: {} (mode={:?}, scenario={:?})",
                    skill.name,
                    skill.od_metadata.as_ref().and_then(|od| od.mode.as_ref()),
                    skill.od_metadata.as_ref().and_then(|od| od.scenario.as_ref()),
                );
                Json(serde_json::to_value(skill).unwrap_or_default())
            } else {
                tracing::warn!(module = "Bridge::design_skill_detail", "⚠️ 未找到技能 id: {}", id);
                Json(serde_json::json!({ "error": "Design skill not found" }))
            }
        }
        Err(e) => {
            tracing::error!(module = "Bridge::design_skill_detail", "❌ load_skills 失败: {}", e);
            Json(serde_json::json!({ "error": format!("{}", e) }))
        }
    }
}

async fn design_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    tracing::info!(module = "Bridge::design_stats", "➡️ 收到请求 /api/skills/design/stats");
    let skill_manager = state.skills_manager.clone();
    let analytics_store = state.analytics_store.clone();
    let context_manager = state.context_manager.clone();

    let (total_skills, by_mode, by_scenario, by_fidelity, by_platform, featured_count) = {
        let manager = skill_manager.lock().await;
        match manager.load_skills().await {
            Ok(skills) => {
                let design_skills: Vec<_> = skills
                    .into_iter()
                    .filter(|s| s.od_metadata.is_some())
                    .collect();

                let mut by_mode: HashMap<String, usize> = HashMap::new();
                let mut by_scenario: HashMap<String, usize> = HashMap::new();
                let mut by_fidelity: HashMap<String, usize> = HashMap::new();
                let mut by_platform: HashMap<String, usize> = HashMap::new();
                let mut featured_count = 0usize;

                for skill in &design_skills {
                    if let Some(ref od) = skill.od_metadata {
                        if let Some(ref mode) = od.mode {
                            *by_mode.entry(mode.clone()).or_default() += 1;
                        }
                        if let Some(ref scenario) = od.scenario {
                            *by_scenario.entry(scenario.clone()).or_default() += 1;
                        }
                        if let Some(ref mode) = od.mode {
                            if mode == "high" || mode == "medium" {
                                *by_fidelity.entry(mode.clone()).or_default() += 1;
                            }
                        }
                        if let Some(ref platform) = od.platform {
                            *by_platform.entry(platform.clone()).or_default() += 1;
                        }
                        if let Some(ref mode) = od.mode {
                            if mode == "featured" || mode == "production" {
                                featured_count += 1;
                            }
                        }
                    }
                }

                (design_skills.len(), by_mode, by_scenario, by_fidelity, by_platform, featured_count)
            }
            Err(_) => (0, HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(), 0),
        }
    };

    let design_events = {
        let today_key = analytics_store.today_key();
        let today = analytics_store.get_daily_stats(&today_key).await;
        let msgs = today.as_ref().map(|s| s.messages_sent).unwrap_or(0);
        let convs = today.as_ref().map(|s| s.conversations_created).unwrap_or(0);
        let tkns = today.as_ref().map(|s| s.tokens_input + s.tokens_output).unwrap_or(0);
        serde_json::json!({
            "today_messages": msgs,
            "today_conversations": convs,
            "today_tokens": tkns,
            "date": today_key,
        })
    };

    let caveman_stats = {
        let cm = context_manager.lock().await;
        let stats = cm.get_caveman_stats().await;
        serde_json::json!({
            "total_segments": stats.total_segments,
            "tokens_saved": stats.tokens_saved,
            "total_tokens_processed": stats.total_tokens_processed,
            "avg_compression_ratio": stats.avg_compression_ratio,
        })
    };

    Json(serde_json::json!({
        "total_design_skills": total_skills,
        "featured_skills": featured_count,
        "by_mode": by_mode,
        "by_scenario": by_scenario,
        "by_fidelity": by_fidelity,
        "by_platform": by_platform,
        "today_usage": design_events,
        "caveman": caveman_stats,
    }))
}

async fn superpowers_install(State(state): State<AppState>) -> Json<serde_json::Value> {
    let skill_manager = state.skills_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, SkillsManager> = skill_manager.lock().await;
    match manager.install_superpowers_skills().await {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "installed_count": result.installed_count,
            "failed_count": result.failed_count,
            "errors": result.errors
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": format!("{}", e)
        })),
    }
}

async fn superpowers_categories() -> Json<Vec<SuperpowersCategory>> {
    Json(SkillsManager::get_superpowers_categories())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/skills", get(skills_list))
        .route("/api/skills", post(skills_create))
        .route("/api/skills/{name}", get(skill_get))
        .route("/api/skills/{name}", put(skill_update))
        .route("/api/skills/{name}", delete(skill_delete))
        .route("/api/skills/{name}/enable", post(skill_enable))
        .route("/api/skills/{name}/execute", post(skill_execute))
        .route("/api/skills/match", post(skills_match))
        .route("/api/skills/design", get(design_skills_list))
        .route("/api/skills/design/{id}", get(design_skill_detail))
        .route("/api/skills/design/stats", get(design_stats))
        .route("/api/superpowers/install", post(superpowers_install))
        .route("/api/superpowers/categories", get(superpowers_categories))
}
