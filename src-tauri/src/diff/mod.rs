use anyhow::Result;
use diffy::{create_patch, Patch, Line};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub diff_text: String,
    pub hunks: Vec<DiffHunk>,
    pub added_lines: usize,
    pub removed_lines: usize,
}

pub fn generate_diff(original: &str, modified: &str) -> DiffResult {
    let patch = create_patch(original, modified);
    let diff_text = patch.to_string();

    let mut hunks = Vec::new();
    let mut total_added = 0;
    let mut total_removed = 0;

    for hunk in patch.hunks() {
        let mut hunk_lines = Vec::new();
        let mut hunk_added = 0;
        let mut hunk_removed = 0;

        for line in hunk.lines() {
            match line {
                Line::Context(s) => {
                    hunk_lines.push(DiffLine {
                        kind: DiffLineKind::Context,
                        content: s.to_string(),
                    });
                }
                Line::Insert(s) => {
                    hunk_added += 1;
                    hunk_lines.push(DiffLine {
                        kind: DiffLineKind::Added,
                        content: s.to_string(),
                    });
                }
                Line::Delete(s) => {
                    hunk_removed += 1;
                    hunk_lines.push(DiffLine {
                        kind: DiffLineKind::Removed,
                        content: s.to_string(),
                    });
                }
            }
        }

        total_added += hunk_added;
        total_removed += hunk_removed;

        let old_range = hunk.old_range();
        let new_range = hunk.new_range();

        hunks.push(DiffHunk {
            old_start: old_range.start(),
            old_count: old_range.len(),
            new_start: new_range.start(),
            new_count: new_range.len(),
            lines: hunk_lines,
        });
    }

    DiffResult {
        diff_text,
        hunks,
        added_lines: total_added,
        removed_lines: total_removed,
    }
}

pub fn generate_file_diff(original_path: &str, modified_path: &str) -> Result<DiffResult> {
    let original = std::fs::read_to_string(original_path)?;
    let modified = std::fs::read_to_string(modified_path)?;
    Ok(generate_diff(&original, &modified))
}

pub fn apply_diff_to_content(content: &str, diff_text: &str) -> Result<String> {
    let patch = Patch::from_str(diff_text)?;
    Ok(diffy::apply(content, &patch)?)
}

pub fn apply_diff_to_file(file_path: &str, diff_text: &str) -> Result<()> {
    let content = std::fs::read_to_string(file_path)?;
    let new_content = apply_diff_to_content(&content, diff_text)?;
    std::fs::write(file_path, new_content)?;
    Ok(())
}

pub fn parse_unified_diff(diff_text: &str) -> Result<DiffResult> {
    let patch = Patch::from_str(diff_text)?;
    let mut hunks = Vec::new();
    let mut total_added = 0;
    let mut total_removed = 0;

    for hunk in patch.hunks() {
        let mut hunk_lines = Vec::new();
        let mut hunk_added = 0;
        let mut hunk_removed = 0;

        for line in hunk.lines() {
            match line {
                Line::Context(s) => {
                    hunk_lines.push(DiffLine {
                        kind: DiffLineKind::Context,
                        content: s.to_string(),
                    });
                }
                Line::Insert(s) => {
                    hunk_added += 1;
                    hunk_lines.push(DiffLine {
                        kind: DiffLineKind::Added,
                        content: s.to_string(),
                    });
                }
                Line::Delete(s) => {
                    hunk_removed += 1;
                    hunk_lines.push(DiffLine {
                        kind: DiffLineKind::Removed,
                        content: s.to_string(),
                    });
                }
            }
        }

        total_added += hunk_added;
        total_removed += hunk_removed;

        let old_range = hunk.old_range();
        let new_range = hunk.new_range();

        hunks.push(DiffHunk {
            old_start: old_range.start(),
            old_count: old_range.len(),
            new_start: new_range.start(),
            new_count: new_range.len(),
            lines: hunk_lines,
        });
    }

    Ok(DiffResult {
        diff_text: diff_text.to_string(),
        hunks,
        added_lines: total_added,
        removed_lines: total_removed,
    })
}
