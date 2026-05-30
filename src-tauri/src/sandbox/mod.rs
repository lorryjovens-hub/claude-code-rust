use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub max_execution_time: u64,
    pub max_memory_mb: u64,
    pub allowed_languages: Vec<String>,
    pub max_output_size: usize,
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub allowed_paths: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_execution_time: 30,
            max_memory_mb: 512,
            allowed_languages: vec![
                "python".to_string(),
                "javascript".to_string(),
                "rust".to_string(),
                "go".to_string(),
                "bash".to_string(),
                "powershell".to_string(),
            ],
            max_output_size: 10000,
            allow_network: true,
            allow_filesystem: true,
            allowed_paths: vec![
                "/tmp".to_string(),
                "/sandbox".to_string(),
                "/home".to_string(),
                "/Users".to_string(),
                "D:\\".to_string(),
                "C:\\".to_string(),
                "C:\\Users".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub language: String,
    pub code: String,
    pub input: Option<String>,
    pub timeout: Option<u64>,
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time_ms: u128,
    pub error: Option<String>,
}

pub struct CodeSandbox {
    config: SandboxConfig,
    execution_history: Arc<RwLock<Vec<ExecutionResult>>>,
    temp_dir: PathBuf,
}

/// Windows Job Object with auto-closing handle (RAII guard).
#[cfg(windows)]
struct MemoryLimitedJob {
    handle: *mut std::ffi::c_void,
}

#[cfg(windows)]
impl Drop for MemoryLimitedJob {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            extern "system" {
                fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
            }
            unsafe { CloseHandle(self.handle); }
        }
    }
}

impl CodeSandbox {
    pub fn new(temp_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&temp_dir).ok();
        Self {
            config: SandboxConfig::default(),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            temp_dir,
        }
    }

    pub async fn execute(&self, req: ExecutionRequest) -> ExecutionResult {
        let id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        if !self.config.allowed_languages.contains(&req.language) {
            return ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: format!("Language '{}' is not allowed", req.language),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(format!("Unsupported language: {}", req.language)),
            };
        }

        crate::metrics::TOOL_CALLS_TOTAL.inc();

        let timeout = req.timeout.unwrap_or(self.config.max_execution_time);

        let result = match req.language.as_str() {
            "python" => self.execute_python(&req.code, timeout, &req.environment).await,
            "javascript" | "node" => self.execute_javascript(&req.code, timeout, &req.environment).await,
            "bash" => self.execute_bash(&req.code, timeout, &req.environment).await,
            "powershell" => self.execute_powershell(&req.code, timeout, &req.environment).await,
            "rust" => self.execute_rust(&req.code, timeout, &req.environment).await,
            "go" => self.execute_go(&req.code, timeout, &req.environment).await,
            _ => ExecutionResult {
                id: id.clone(),
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(format!("Unsupported language: {}", req.language)),
            },
        };

        let mut history = self.execution_history.write().await;
        history.push(result.clone());

        result
    }

    async fn run_with_timeout(
        &self,
        mut cmd: tokio::process::Command,
        timeout_secs: u64,
    ) -> Result<std::process::Output, String> {
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let child = cmd.spawn().map_err(|e| e.to_string())?;
        let pid = child.id();

        let effective_timeout = if timeout_secs == 0 {
            self.config.max_execution_time
        } else {
            timeout_secs
        };
        let duration = std::time::Duration::from_secs(effective_timeout);

        let handle = tokio::spawn(child.wait_with_output());

        tokio::select! {
            result = handle => {
                match result {
                    Ok(Ok(output)) => Ok(output),
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(e) => Err(e.to_string()),
                }
            }
            _ = tokio::time::sleep(duration) => {
                if let Some(pid_val) = pid {
                    #[cfg(windows)]
                    {
                        let _ = std::process::Command::new("taskkill")
                            .args(["/F", "/T", "/PID", &pid_val.to_string()])
                            .output();
                    }
                    #[cfg(not(windows))]
                    {
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &pid_val.to_string()])
                            .output();
                    }
                }
                Err(format!("Execution timed out after {} seconds", effective_timeout))
            }
        }
    }

    async fn execute_python(
        &self,
        code: &str,
        timeout_secs: u64,
        env: &Option<HashMap<String, String>>,
    ) -> ExecutionResult {
        let id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let script_path = self.temp_dir.join(format!("{}.py", id));
        if let Err(e) = std::fs::write(&script_path, code) {
            return ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to write script: {}", e),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e.to_string()),
            };
        }

        let mut cmd = tokio::process::Command::new("python");
        cmd.arg(&script_path);
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let result = self.run_with_timeout(cmd, timeout_secs).await;
        let _ = tokio::fs::remove_file(&script_path).await;

        match result {
            Ok(output) => ExecutionResult {
                id,
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).chars().take(self.config.max_output_size).collect(),
                stderr: String::from_utf8_lossy(&output.stderr).chars().take(self.config.max_output_size).collect(),
                exit_code: output.status.code().unwrap_or(-1),
                execution_time_ms: start_time.elapsed().as_millis(),
                error: None,
            },
            Err(e) => ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e),
            },
        }
    }

    async fn execute_javascript(
        &self,
        code: &str,
        timeout_secs: u64,
        env: &Option<HashMap<String, String>>,
    ) -> ExecutionResult {
        let id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let script_path = self.temp_dir.join(format!("{}.js", id));
        if let Err(e) = std::fs::write(&script_path, code) {
            return ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to write script: {}", e),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e.to_string()),
            };
        }

        let mut cmd = tokio::process::Command::new("node");
        cmd.arg(&script_path);
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let result = self.run_with_timeout(cmd, timeout_secs).await;
        let _ = tokio::fs::remove_file(&script_path).await;

        match result {
            Ok(output) => ExecutionResult {
                id,
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).chars().take(self.config.max_output_size).collect(),
                stderr: String::from_utf8_lossy(&output.stderr).chars().take(self.config.max_output_size).collect(),
                exit_code: output.status.code().unwrap_or(-1),
                execution_time_ms: start_time.elapsed().as_millis(),
                error: None,
            },
            Err(e) => ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e),
            },
        }
    }

    async fn execute_bash(
        &self,
        code: &str,
        timeout_secs: u64,
        env: &Option<HashMap<String, String>>,
    ) -> ExecutionResult {
        let id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let mut cmd = tokio::process::Command::new("bash");
        cmd.arg("-c").arg(code);
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let result = self.run_with_timeout(cmd, timeout_secs).await;

        match result {
            Ok(output) => ExecutionResult {
                id,
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).chars().take(self.config.max_output_size).collect(),
                stderr: String::from_utf8_lossy(&output.stderr).chars().take(self.config.max_output_size).collect(),
                exit_code: output.status.code().unwrap_or(-1),
                execution_time_ms: start_time.elapsed().as_millis(),
                error: None,
            },
            Err(e) => ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e),
            },
        }
    }

    async fn execute_powershell(
        &self,
        code: &str,
        timeout_secs: u64,
        env: &Option<HashMap<String, String>>,
    ) -> ExecutionResult {
        let id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let mut cmd = tokio::process::Command::new("powershell");
        cmd.arg("-Command").arg(code);
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let result = self.run_with_timeout(cmd, timeout_secs).await;

        match result {
            Ok(output) => ExecutionResult {
                id,
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).chars().take(self.config.max_output_size).collect(),
                stderr: String::from_utf8_lossy(&output.stderr).chars().take(self.config.max_output_size).collect(),
                exit_code: output.status.code().unwrap_or(-1),
                execution_time_ms: start_time.elapsed().as_millis(),
                error: None,
            },
            Err(e) => ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e),
            },
        }
    }

    async fn execute_rust(
        &self,
        code: &str,
        timeout_secs: u64,
        env: &Option<HashMap<String, String>>,
    ) -> ExecutionResult {
        let id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let source_path = self.temp_dir.join(format!("{}.rs", id));
        let exe_path = self.temp_dir.join(format!("sandbox_{}", id));

        if let Err(e) = std::fs::write(&source_path, code) {
            return ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to write source: {}", e),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e.to_string()),
            };
        }

        let mut compile_cmd = tokio::process::Command::new("rustc");
        compile_cmd.arg(&source_path).arg("-o").arg(&exe_path);
        compile_cmd.stdout(std::process::Stdio::piped());
        compile_cmd.stderr(std::process::Stdio::piped());

        let compile_result = self.run_with_timeout(compile_cmd, timeout_secs).await;

        match compile_result {
            Ok(compile_output) => {
                if !compile_output.status.success() {
                    let stderr = String::from_utf8_lossy(&compile_output.stderr)
                        .chars()
                        .take(self.config.max_output_size)
                        .collect();

                    let _ = tokio::fs::remove_file(&source_path).await;

                    return ExecutionResult {
                        id,
                        success: false,
                        stdout: String::new(),
                        stderr,
                        exit_code: compile_output.status.code().unwrap_or(-1),
                        execution_time_ms: start_time.elapsed().as_millis(),
                        error: Some("Compilation failed".to_string()),
                    };
                }

                let mut run_cmd = tokio::process::Command::new(&exe_path);
                if let Some(env_vars) = env {
                    for (key, value) in env_vars {
                        run_cmd.env(key, value);
                    }
                }

                let run_result = self.run_with_timeout(run_cmd, timeout_secs).await;

                let _ = tokio::fs::remove_file(&source_path).await;
                let _ = tokio::fs::remove_file(&exe_path).await;

                match run_result {
                    Ok(output) => ExecutionResult {
                        id,
                        success: output.status.success(),
                        stdout: String::from_utf8_lossy(&output.stdout).chars().take(self.config.max_output_size).collect(),
                        stderr: String::from_utf8_lossy(&output.stderr).chars().take(self.config.max_output_size).collect(),
                        exit_code: output.status.code().unwrap_or(-1),
                        execution_time_ms: start_time.elapsed().as_millis(),
                        error: None,
                    },
                    Err(e) => ExecutionResult {
                        id,
                        success: false,
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: -1,
                        execution_time_ms: start_time.elapsed().as_millis(),
                        error: Some(e),
                    },
                }
            }
            Err(e) => {
                let _ = tokio::fs::remove_file(&source_path).await;

                ExecutionResult {
                    id,
                    success: false,
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: -1,
                    execution_time_ms: start_time.elapsed().as_millis(),
                    error: Some(e),
                }
            }
        }
    }

    async fn execute_go(
        &self,
        code: &str,
        timeout_secs: u64,
        env: &Option<HashMap<String, String>>,
    ) -> ExecutionResult {
        let id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let source_path = self.temp_dir.join(format!("{}.go", id));

        if let Err(e) = std::fs::write(&source_path, code) {
            return ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to write source: {}", e),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e.to_string()),
            };
        }

        let mut cmd = tokio::process::Command::new("go");
        cmd.arg("run").arg(&source_path);
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let result = self.run_with_timeout(cmd, timeout_secs).await;
        let _ = tokio::fs::remove_file(&source_path).await;

        match result {
            Ok(output) => ExecutionResult {
                id,
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).chars().take(self.config.max_output_size).collect(),
                stderr: String::from_utf8_lossy(&output.stderr).chars().take(self.config.max_output_size).collect(),
                exit_code: output.status.code().unwrap_or(-1),
                execution_time_ms: start_time.elapsed().as_millis(),
                error: None,
            },
            Err(e) => ExecutionResult {
                id,
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: -1,
                execution_time_ms: start_time.elapsed().as_millis(),
                error: Some(e),
            },
        }
    }

    #[cfg(windows)]
    fn create_job_object_with_memory_limit(memory_limit_mb: u64) -> Result<*mut std::ffi::c_void, String> {
        #[repr(C)]
        struct JobObjectBasicLimitInformation {
            per_process_user_time_limit: i64,
            per_job_user_time_limit: i64,
            limit_flags: u32,
            minimum_working_set_size: usize,
            maximum_working_set_size: usize,
            active_process_limit: u32,
            affinity: usize,
            priority_class: u32,
            scheduling_class: u32,
        }

        #[repr(C)]
        struct IoCounters {
            read_operation_count: u64,
            write_operation_count: u64,
            other_operation_count: u64,
            read_transfer_count: u64,
            write_transfer_count: u64,
            other_transfer_count: u64,
        }

        #[repr(C)]
        struct JobObjectExtendedLimitInformation {
            basic_limit_information: JobObjectBasicLimitInformation,
            io_info: IoCounters,
            process_memory_limit: usize,
            job_memory_limit: usize,
            peak_process_memory_used: usize,
            peak_job_memory_used: usize,
        }

        const JOB_OBJECT_LIMIT_JOB_MEMORY: u32 = 0x00000200;
        const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: u32 = 9;

        #[link(name = "kernel32")]
        extern "system" {
            fn CreateJobObjectW(
                lp_job_attributes: *mut std::ffi::c_void,
                lp_name: *const u16,
            ) -> *mut std::ffi::c_void;

            fn SetInformationJobObject(
                h_job: *mut std::ffi::c_void,
                job_object_information_class: u32,
                lp_job_object_information: *const std::ffi::c_void,
                cb_job_object_information_length: u32,
            ) -> i32;
        }

        unsafe {
            let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
            if job.is_null() {
                return Err("Failed to create job object".to_string());
            }

            let mut info: JobObjectExtendedLimitInformation = std::mem::zeroed();
            info.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_JOB_MEMORY;
            info.job_memory_limit = (memory_limit_mb * 1024 * 1024) as usize;

            let result = SetInformationJobObject(
                job,
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
                &info as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
            );

            if result == 0 {
                return Err("Failed to set job object limits".to_string());
            }

            Ok(job)
        }
    }

    #[cfg(windows)]
    fn assign_process_to_job_object(job: *mut std::ffi::c_void, pid: u32) -> Result<(), String> {
        const PROCESS_SET_QUOTA: u32 = 0x0100;
        const PROCESS_TERMINATE: u32 = 0x0001;

        #[link(name = "kernel32")]
        extern "system" {
            fn OpenProcess(
                dw_desired_access: u32,
                b_inherit_handles: i32,
                dw_process_id: u32,
            ) -> *mut std::ffi::c_void;

            fn AssignProcessToJobObject(
                h_job: *mut std::ffi::c_void,
                h_process: *mut std::ffi::c_void,
            ) -> i32;

            fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
        }

        unsafe {
            let process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
            if process.is_null() {
                return Err("Failed to open process for job assignment".to_string());
            }

            let result = AssignProcessToJobObject(job, process);
            CloseHandle(process);

            if result == 0 {
                return Err("Failed to assign process to job object".to_string());
            }

            Ok(())
        }
    }

    pub async fn get_history(&self) -> Vec<ExecutionResult> {
        self.execution_history.read().await.clone()
    }

    pub async fn clear_history(&self) {
        self.execution_history.write().await.clear();
    }
}
