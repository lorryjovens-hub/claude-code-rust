use serde::{Serialize, Deserialize};
use std::process::Command;
use std::path::PathBuf;
use tauri::AppHandle;

/// Represents a Remotion project
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemotionProject {
    pub name: String,
    pub path: String,
    pub compositions: Vec<CompositionInfo>,
    pub has_node_modules: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompositionInfo {
    pub id: String,
    pub duration_in_frames: u32,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderRequest {
    pub project_path: String,
    pub composition_id: String,
    pub output_path: String,
    pub fps: Option<u32>,
    pub frames: Option<Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderResponse {
    pub success: bool,
    pub output_file: String,
    pub duration_secs: f64,
    pub error: Option<String>,
}

/// Create a new Remotion project using `npx create-video`
#[tauri::command]
pub async fn remotion_create_project(
    _app: AppHandle,
    name: String,
    target_dir: String,
    template: Option<String>,
) -> Result<RemotionProject, String> {
    let project_path = PathBuf::from(&target_dir).join(&name);
    
    // Check if directory exists
    if project_path.exists() {
        return Err(format!("Directory already exists: {}", project_path.display()));
    }

    // Run create-video (non-interactive: close stdin to avoid hanging on prompts)
    let template_arg = template.unwrap_or_else(|| "blank".to_string());

    let output = Command::new("npx")
        .args([
            "--yes",  // skip npx install prompt
            "create-video@latest",
            &name,
            "--template",
            &template_arg,
            "--package-manager=npm",
        ])
        .stdin(std::process::Stdio::null())
        .current_dir(&target_dir)
        .output()
        .map_err(|e| format!("Failed to run create-video: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("create-video failed: {}", stderr));
    }

    // Check for node_modules
    let has_node_modules = project_path.join("node_modules").exists();

    Ok(RemotionProject {
        name,
        path: project_path.to_string_lossy().to_string(),
        compositions: vec![],
        has_node_modules,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Install dependencies for a Remotion project
#[tauri::command]
pub async fn remotion_install_deps(project_path: String) -> Result<String, String> {
    let output = Command::new("npm")
        .args(["install"])
        .stdin(std::process::Stdio::null())
        .current_dir(&project_path)
        .output()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("npm install failed: {}", stderr));
    }

    Ok("Dependencies installed successfully".to_string())
}

/// Start the Remotion Studio dev server
#[tauri::command]
pub async fn remotion_start_studio(
    _app: AppHandle,
    project_path: String,
    port: Option<u16>,
) -> Result<String, String> {
    let port = port.unwrap_or(3000);
    let port_str = port.to_string();

    // Run npx remotion studio
    let _child = Command::new("npx")
        .args(["remotion", "studio", "--port", &port_str])
        .current_dir(&project_path)
        .spawn()
        .map_err(|e| format!("Failed to start Remotion Studio: {}", e))?;

    let url = format!("http://localhost:{}", port);
    
    // Open the URL in browser
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", &url])
            .spawn()
            .ok();
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .ok();
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .ok();
    }

    Ok(format!("Remotion Studio started at {}", url))
}

/// Render a Remotion composition to video
#[tauri::command]
pub async fn remotion_render(
    _app: AppHandle,
    request: RenderRequest,
) -> Result<RenderResponse, String> {
    let start = std::time::Instant::now();

    let mut args = vec![
        "remotion", "render",
        request.composition_id.as_str(),
        request.output_path.as_str(),
        "--log=verbose",
    ];

    let fps_arg;
    if let Some(fps) = request.fps {
        fps_arg = fps.to_string();
        args.push("--fps");
        args.push(&fps_arg);
    }

    let frame_range;
    if let Some(frames) = &request.frames {
        if frames.len() == 2 {
            frame_range = format!("{}-{}", frames[0], frames[1]);
            args.push("--frames");
            args.push(&frame_range);
        }
    }

    let output = Command::new("npx")
        .args(&args)
        .stdin(std::process::Stdio::null())
        .current_dir(&request.project_path)
        .output()
        .map_err(|e| format!("Failed to run remotion render: {}", e))?;

    let duration = start.elapsed().as_secs_f64();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(RenderResponse {
            success: false,
            output_file: request.output_path.clone(),
            duration_secs: duration,
            error: Some(stderr.to_string()),
        });
    }

    Ok(RenderResponse {
        success: true,
        output_file: request.output_path,
        duration_secs: duration,
        error: None,
    })
}

/// List compositions in a Remotion project
#[tauri::command]
pub async fn remotion_list_compositions(
    project_path: String,
) -> Result<Vec<CompositionInfo>, String> {
    let output = Command::new("npx")
        .args(["remotion", "compositions", "--json"])
        .current_dir(&project_path)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("Failed to list compositions: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to list compositions: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse JSON output
    #[derive(Deserialize)]
    struct RawComposition {
        id: String,
        #[serde(rename = "durationInFrames")]
        duration_in_frames: u32,
        fps: u32,
        width: u32,
        height: u32,
    }

    let raw: Vec<RawComposition> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse compositions JSON: {}", e))?;

    Ok(raw.into_iter().map(|c| CompositionInfo {
        id: c.id,
        duration_in_frames: c.duration_in_frames,
        fps: c.fps,
        width: c.width,
        height: c.height,
    }).collect())
}

/// Get all Remotion projects in a directory
#[tauri::command]
pub async fn remotion_scan_projects(scan_dir: String) -> Result<Vec<RemotionProject>, String> {
    let dir = PathBuf::from(&scan_dir);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", scan_dir));
    }

    let mut projects = vec![];
    
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let package_json = path.join("package.json");
                if package_json.exists() {
                    if let Ok(content) = std::fs::read_to_string(&package_json) {
                        // Check if it's a Remotion project
                        if content.contains("remotion") {
                            let name = path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());
                            
                            let has_node_modules = path.join("node_modules").exists();
                            
                            projects.push(RemotionProject {
                                name,
                                path: path.to_string_lossy().to_string(),
                                compositions: vec![],
                                has_node_modules,
                                created_at: String::new(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(projects)
}

/// Open a Remotion project in VS Code
#[tauri::command]
pub async fn remotion_open_in_editor(project_path: String) -> Result<String, String> {
    let _output = Command::new("code")
        .arg(&project_path)
        .output()
        .map_err(|_| "VS Code not found. Install it or use 'code' CLI.".to_string())?;

    Ok(format!("Opened {} in editor", project_path))
}

/// Still - render a single frame as image
#[tauri::command]
pub async fn remotion_still(
    project_path: String,
    composition_id: String,
    output_path: String,
    frame: Option<u32>,
) -> Result<String, String> {
    let frame_str = frame.unwrap_or(0).to_string();
    
    let output = Command::new("npx")
        .args([
            "remotion", "still",
            &composition_id,
            &output_path,
            "--frame", &frame_str,
        ])
        .current_dir(&project_path)
        .output()
        .map_err(|e| format!("Failed to render still: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Still render failed: {}", stderr));
    }

    Ok(output_path)
}
