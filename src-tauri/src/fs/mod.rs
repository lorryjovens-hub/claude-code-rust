use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use base64::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file: String,
    pub line_number: usize,
    pub content: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size: u64,
    pub by_extension: HashMap<String, usize>,
}

pub struct FileOperations;

/// Sensitive system directories that filesystem operations should never touch.
fn blocked_path_prefixes() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
        &[
            "C:\\Windows",
            "C:\\Windows\\System32",
            "C:\\ProgramData",
            "C:\\Boot",
            "C:\\$Recycle.Bin",
        ]
    } else {
        &[
            "/etc",
            "/sys",
            "/proc",
            "/boot",
            "/root",
            "/dev",
            "/var/log",
            "/var/run",
            "/tmp/systemd-",
            "/run/systemd",
            "/usr/lib/systemd",
        ]
    }
}

/// Validate and canonicalize a path to prevent traversal attacks.
/// Returns the canonicalized path on success.
pub fn validate_path(path: &str) -> Result<PathBuf> {
    let p = Path::new(path);

    // Canonicalize resolves .. components and symlinks
    let canonical = match p.canonicalize() {
        Ok(c) => c,
        Err(_) => {
            // Path may not exist yet (e.g. write/create operations).
            // Normalize by resolving .. manually and prepending current dir.
            let abs = if p.is_absolute() {
                p.to_path_buf()
            } else {
                std::env::current_dir()?.join(p)
            };
            // Normalize: remove .. components
            let mut components = Vec::new();
            for component in abs.components() {
                match component {
                    std::path::Component::ParentDir => {
                        components.pop();
                    }
                    std::path::Component::CurDir => {}
                    c => components.push(c),
                }
            }
            let normalized: PathBuf = components.iter().collect();
            // If the path doesn't exist, we can't fully canonicalize,
            // but we can check the normalized form
            normalized
        }
    };

    // Strip Windows extended-path prefix (\\?\) before security check
    let path_for_check = canonical.to_string_lossy();
    let path_str = path_for_check
        .strip_prefix(r"\\?\")
        .unwrap_or(&path_for_check)
        .to_lowercase();

    // Block access to sensitive system directories
    for blocked in blocked_path_prefixes() {
        if path_str == blocked.to_lowercase() || path_str.starts_with(&format!("{}\\", blocked.to_lowercase())) || path_str.starts_with(&format!("{}/", blocked.to_lowercase())) {
            return Err(anyhow!(
                "Access denied: path '{}' is in a protected system directory",
                canonical.display()
            ));
        }
    }

    Ok(canonical)
}

impl FileOperations {
    pub fn read_file(path: &str, offset: Option<usize>, limit: Option<usize>) -> Result<String> {
        let validated = validate_path(path)?;
        let content = fs::read_to_string(&validated)?;
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(usize::MAX);

        let lines: Vec<&str> = content.lines().collect();
        if offset >= lines.len() {
            return Ok(String::new());
        }

        let end = (offset + limit).min(lines.len());
        Ok(lines[offset..end].join("\n"))
    }

    pub fn write_file(path: &str, content: &str) -> Result<()> {
        // Validate the path is safe before writing
        let p = Path::new(path);
        let abs = if p.is_absolute() {
            p.to_path_buf()
        } else {
            std::env::current_dir()?.join(p)
        };
        validate_path(&abs.to_string_lossy())?;

        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&abs, content)?;
        Ok(())
    }

    pub fn edit_file(path: &str, old_string: &str, new_string: &str, replace_all: bool) -> Result<()> {
        let validated = validate_path(path)?;
        let mut file_content = fs::read_to_string(&validated)?;

        if replace_all {
            file_content = file_content.replace(old_string, new_string);
        } else {
            if let Some(pos) = file_content.find(old_string) {
                file_content = format!("{}{}{}", &file_content[..pos], new_string, &file_content[pos + old_string.len()..]);
            } else {
                return Err(anyhow!("Could not find the specified string in the file"));
            }
        }

        fs::write(&validated, file_content)?;
        Ok(())
    }

    pub fn multi_edit_file(path: &str, edits: Vec<(String, String)>) -> Result<()> {
        let validated = validate_path(path)?;
        let mut file_content = fs::read_to_string(&validated)?;

        for (old_string, new_string) in edits {
            file_content = file_content.replace(&old_string, &new_string);
        }

        fs::write(&validated, file_content)?;
        Ok(())
    }

    pub fn list_directory(path: &str, recursive: bool) -> Result<Vec<FileEntry>> {
        let validated = validate_path(path)?;
        let mut entries = Vec::new();
        let base_path = &validated;

        if recursive {
            for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
                let metadata = entry.metadata()?;
                let file_path = entry.path();

                entries.push(FileEntry {
                    path: file_path.to_string_lossy().to_string(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                    modified: metadata.modified().ok().map(|t| {
                        chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
                    }),
                    extension: file_path.extension().map(|e| e.to_string_lossy().to_string()),
                });
            }
        } else {
            for entry in fs::read_dir(base_path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                let file_path = entry.path();

                entries.push(FileEntry {
                    path: file_path.to_string_lossy().to_string(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                    modified: metadata.modified().ok().map(|t| {
                        chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
                    }),
                    extension: file_path.extension().map(|e| e.to_string_lossy().to_string()),
                });
            }
        }

        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(entries)
    }

    pub fn glob(pattern: &str, path: Option<&str>) -> Result<Vec<String>> {
        let base = path.map(|p| validate_path(p).unwrap_or_else(|_| PathBuf::from(p)))
            .unwrap_or_else(|| PathBuf::from("."));
        let mut matches = Vec::new();

        for entry in glob::glob(&format!(
            "{}/{}",
            base.to_string_lossy(),
            pattern
        ))? {
            if let Ok(path) = entry {
                matches.push(path.to_string_lossy().to_string());
            }
        }

        matches.sort();
        Ok(matches)
    }

    pub fn grep(pattern: &str, path: &str, include: Option<&str>, context: Option<usize>) -> Result<Vec<SearchResult>> {
        let re = regex::Regex::new(pattern)?;
        let context_lines = context.unwrap_or(0);
        let mut results = Vec::new();

        let validated = validate_path(path)?;
        let validated_str = validated.to_string_lossy();

        let paths: Vec<PathBuf> = if validated.is_file() {
            vec![validated.clone()]
        } else if let Some(glob_pattern) = include {
            let full_pattern = format!("{}/{}", validated_str, glob_pattern);
            glob::glob(&full_pattern)?
                .filter_map(|p| p.ok())
                .collect()
        } else {
            WalkDir::new(&validated)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect()
        };

        for file_path in paths {
            if let Ok(content) = fs::read_to_string(&file_path) {
                let lines: Vec<&str> = content.lines().collect();

                for (idx, line) in lines.iter().enumerate() {
                    if re.is_match(line) {
                        let context_before: Vec<String> = lines[..idx]
                            .iter()
                            .rev()
                            .take(context_lines)
                            .map(|s| s.to_string())
                            .collect();

                        let context_after: Vec<String> = lines[idx + 1..]
                            .iter()
                            .take(context_lines)
                            .map(|s| s.to_string())
                            .collect();

                        results.push(SearchResult {
                            file: file_path.to_string_lossy().to_string(),
                            line_number: idx + 1,
                            content: line.to_string(),
                            context_before: context_before.into_iter().rev().collect(),
                            context_after,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn get_file_stats(path: &str) -> Result<FileStats> {
        let mut total_files = 0;
        let mut total_dirs = 0;
        let mut total_size = 0u64;
        let mut by_extension: HashMap<String, usize> = HashMap::new();

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_files += 1;
                total_size += metadata.len();

                if let Some(ext) = entry.path().extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    *by_extension.entry(ext_str).or_insert(0) += 1;
                }
            } else if metadata.is_dir() {
                total_dirs += 1;
            }
        }

        Ok(FileStats {
            total_files,
            total_dirs,
            total_size,
            by_extension,
        })
    }

    pub fn create_directory(path: &str) -> Result<()> {
        let validated = validate_path(path)?;
        fs::create_dir_all(&validated)?;
        Ok(())
    }

    pub fn delete_file(path: &str) -> Result<()> {
        let validated = validate_path(path)?;
        if validated.is_dir() {
            fs::remove_dir_all(&validated)?;
        } else {
            fs::remove_file(&validated)?;
        }
        Ok(())
    }

    pub fn exists(path: &str) -> bool {
        validate_path(path).map(|p| p.exists()).unwrap_or(false)
    }

    pub fn is_directory(path: &str) -> bool {
        validate_path(path).map(|p| p.is_dir()).unwrap_or(false)
    }

    pub fn file_size(path: &str) -> Result<u64> {
        let validated = validate_path(path)?;
        let metadata = fs::metadata(&validated)?;
        Ok(metadata.len())
    }

    pub fn get_modified_time(path: &str) -> Result<String> {
        let validated = validate_path(path)?;
        let metadata = fs::metadata(&validated)?;
        let modified = metadata.modified()?;
        Ok(chrono::DateTime::<chrono::Utc>::from(modified).to_rfc3339())
    }

    pub fn get_extension(path: &str) -> Option<String> {
        validate_path(path).ok()?
            .extension()
            .map(|e| e.to_string_lossy().to_string())
    }

    pub fn read_binary(path: &str) -> Result<Vec<u8>> {
        let validated = validate_path(path)?;
        Ok(fs::read(&validated)?)
    }

    pub fn write_binary(path: &str, data: &[u8]) -> Result<()> {
        let validated = validate_path(path)?;
        if let Some(parent) = validated.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&validated, data)?;
        Ok(())
    }

    pub fn copy_file(from: &str, to: &str) -> Result<()> {
        let validated_from = validate_path(from)?;
        let validated_to = validate_path(to)?;
        if let Some(parent) = validated_to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&validated_from, &validated_to)?;
        Ok(())
    }

    pub fn move_file(from: &str, to: &str) -> Result<()> {
        let validated_from = validate_path(from)?;
        let validated_to = validate_path(to)?;
        if let Some(parent) = validated_to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&validated_from, &validated_to)?;
        Ok(())
    }

    pub fn get_relative_path(path: &str, base: &str) -> Result<String> {
        let validated_path = validate_path(path)?;
        let validated_base = validate_path(base)?;

        Ok(validated_path.strip_prefix(&validated_base)?
            .to_string_lossy()
            .to_string())
    }

    pub fn get_absolute_path(path: &str) -> Result<String> {
        let validated = validate_path(path)?;
        Ok(validated.to_string_lossy().to_string())
    }

    pub fn find_files_by_extension(dir: &str, extension: &str) -> Result<Vec<String>> {
        let validated = validate_path(dir)?;
        let pattern = format!("{}/**/*.{}", validated.to_string_lossy(), extension);
        let mut results = Vec::new();

        for entry in glob::glob(&pattern)? {
            if let Ok(path) = entry {
                results.push(path.to_string_lossy().to_string());
            }
        }

        Ok(results)
    }

    pub fn get_directory_size(path: &str) -> Result<u64> {
        let validated = validate_path(path)?;
        let mut total_size = 0u64;

        for entry in WalkDir::new(&validated).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                total_size += entry.metadata()?.len();
            }
        }

        Ok(total_size)
    }

    pub fn get_line_count(path: &str) -> Result<usize> {
        let validated = validate_path(path)?;
        let content = fs::read_to_string(&validated)?;
        Ok(content.lines().count())
    }

    pub fn get_language_from_extension(path: &str) -> Option<String> {
        let ext = Path::new(path).extension()?.to_str()?.to_lowercase();

        let languages: HashMap<&str, &str> = [
            ("rs", "Rust"),
            ("js", "JavaScript"),
            ("ts", "TypeScript"),
            ("jsx", "JavaScript (JSX)"),
            ("tsx", "TypeScript (TSX)"),
            ("py", "Python"),
            ("rb", "Ruby"),
            ("go", "Go"),
            ("java", "Java"),
            ("c", "C"),
            ("cpp", "C++"),
            ("h", "C Header"),
            ("hpp", "C++ Header"),
            ("cs", "C#"),
            ("swift", "Swift"),
            ("kt", "Kotlin"),
            ("scala", "Scala"),
            ("php", "PHP"),
            ("html", "HTML"),
            ("css", "CSS"),
            ("scss", "SCSS"),
            ("sass", "Sass"),
            ("less", "Less"),
            ("json", "JSON"),
            ("xml", "XML"),
            ("yaml", "YAML"),
            ("yml", "YAML"),
            ("toml", "TOML"),
            ("md", "Markdown"),
            ("sql", "SQL"),
            ("sh", "Shell"),
            ("bash", "Bash"),
            ("zsh", "Zsh"),
            ("ps1", "PowerShell"),
            ("dockerfile", "Dockerfile"),
            ("makefile", "Makefile"),
            ("cmake", "CMake"),
        ].iter().cloned().collect();

        languages.get(ext.as_str()).map(|s| s.to_string())
    }

    fn decode_file(raw: &[u8]) -> Result<(String, String)> {
        if raw.len() >= 3 && raw[0] == 0xEF && raw[1] == 0xBB && raw[2] == 0xBF {
            return Ok((String::from_utf8(raw[3..].to_vec())
                .map_err(|_| anyhow!("Invalid UTF-8 despite BOM"))?, "UTF-8 BOM".into()));
        }
        if raw.len() >= 2 {
            if raw[0] == 0xFE && raw[1] == 0xFF {
                let (cow, _, _) = encoding_rs::UTF_16BE.decode(raw);
                return Ok((cow.into_owned(), "UTF-16 BE".into()));
            }
            if raw[0] == 0xFF && raw[1] == 0xFE {
                let (cow, _, _) = encoding_rs::UTF_16LE.decode(raw);
                return Ok((cow.into_owned(), "UTF-16 LE".into()));
            }
        }
        if let Ok(s) = std::str::from_utf8(raw) {
            return Ok((s.to_string(), "UTF-8".into()));
        }
        let candidates = [
            encoding_rs::GBK, encoding_rs::GB18030, encoding_rs::SHIFT_JIS,
            encoding_rs::EUC_JP, encoding_rs::EUC_KR, encoding_rs::BIG5,
            encoding_rs::WINDOWS_1252, encoding_rs::WINDOWS_1251,
        ];
        for enc in &candidates {
            let (cow, _, had_errors) = enc.decode(raw);
            if !had_errors && !cow.contains('\u{FFFD}') {
                return Ok((cow.into_owned(), enc.name().into()));
            }
        }
        let (cow, _, _) = encoding_rs::GBK.decode(raw);
        Ok((cow.into_owned(), "GBK (fallback)".into()))
    }

    pub fn read_file_with_encoding(path: &str) -> Result<serde_json::Value> {
        let validated = validate_path(path)?;
        let raw = fs::read(&validated)?;
        let (content, encoding) = Self::decode_file(&raw)?;
        tracing::info!(module = "fs", "Read {} ({}b, enc={})", validated.display(), raw.len(), encoding);
        Ok(serde_json::json!({"content": content, "encoding": encoding, "size": raw.len(), "path": path}))
    }

    pub fn read_file_base64(path: &str) -> Result<serde_json::Value> {
        use base64::Engine;
        let validated = validate_path(path)?;
        let raw = fs::read(&validated)?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&raw);
        let ext = validated.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let mime = crate::fs::mime_from_ext(&ext);
        let is_text = crate::fs::is_text_extension(&ext);
        let text_preview = if is_text {
            match Self::decode_file(&raw) {
                Ok((content, enc)) => Some(serde_json::json!({"content": content, "encoding": enc})),
                Err(_) => None,
            }
        } else { None };
        Ok(serde_json::json!({
            "base64": encoded, "size": raw.len(), "path": path,
            "name": validated.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "mime_type": mime, "extension": ext, "is_binary": !is_text,
            "text_preview": text_preview,
        }))
    }
}

pub fn is_text_extension(ext: &str) -> bool {
    matches!(ext,
        "txt" | "md" | "json" | "xml" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" |
        "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "rb" | "go" | "java" | "c" | "cpp" |
        "h" | "hpp" | "cs" | "swift" | "kt" | "kts" | "scala" | "php" | "pl" | "pm" |
        "html" | "htm" | "css" | "scss" | "sass" | "less" | "svg" | "vue" | "svelte" |
        "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" |
        "sql" | "graphql" | "gql" | "r" | "rmd" | "tex" | "bib" |
        "log" | "csv" | "tsv" | "diff" | "patch" |
        "dockerfile" | "makefile" | "cmake" | "gitignore" | "env" |
        "lock" | "editorconfig" | "eslintrc" | "prettierrc"
    )
}

pub fn mime_from_ext(ext: &str) -> &'static str {
    match ext {
        "html" | "htm" => "text/html", "css" => "text/css",
        "js" => "text/javascript", "ts" => "text/typescript",
        "json" => "application/json", "xml" => "application/xml",
        "pdf" => "application/pdf", "zip" => "application/zip",
        "gz" | "tar" => "application/gzip",
        "png" => "image/png", "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif", "svg" => "image/svg+xml",
        "ico" => "image/x-icon", "webp" => "image/webp",
        "mp3" => "audio/mpeg", "wav" => "audio/wav",
        "mp4" => "video/mp4", "webm" => "video/webm",
        "woff" => "font/woff", "woff2" => "font/woff2",
        "ttf" => "font/ttf", "otf" => "font/otf",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests;
