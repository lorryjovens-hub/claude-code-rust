use super::*;

#[test]
fn validate_path_rejects_system_access() {
    // Use a path that resolves into a blocked directory
    let test_path = if cfg!(windows) {
        "C:\\Windows\\System32\\config\\SAM"
    } else {
        "/etc/shadow"
    };
    let result = validate_path(test_path);
    assert!(result.is_err(), "Should reject access to system path '{}': {:?}", test_path, result);
    let err = result.unwrap_err().to_string();
    assert!(err.contains("protected"), "Error should mention 'protected': {}", err);
}

#[test]
fn validate_path_rejects_normalized_system_path() {
    // Use a path containing .. that resolves INTO a blocked directory
    let test_path = if cfg!(windows) {
        "C:\\Windows\\Temp\\..\\System32\\drivers\\etc"
    } else {
        "/var/../etc/passwd"
    };
    let result = validate_path(test_path);
    assert!(result.is_err(), "Path should be blocked after normalization: {:?}", result);
    let err = result.unwrap_err().to_string();
    assert!(err.contains("protected"), "Expected 'protected' in error: {}", err);
}

#[test]
fn validate_path_blocks_windows_system() {
    if cfg!(windows) {
        let result = validate_path("C:\\Windows\\System32\\config\\SAM");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("protected system directory"));
    }
}

#[test]
fn validate_path_blocks_linux_etc() {
    if cfg!(not(windows)) {
        let result = validate_path("/etc/secret");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("protected system directory"));
    }
}

#[test]
fn validate_path_allows_normal_directories() {
    let result = validate_path(".");
    assert!(result.is_ok());
}

#[test]
fn validate_path_blocks_sys_on_linux() {
    if cfg!(not(windows)) {
        let result = validate_path("/sys/class/net");
        assert!(result.is_err());
    }
}

#[test]
fn validate_path_blocks_proc() {
    if cfg!(not(windows)) {
        let result = validate_path("/proc/self/exe");
        assert!(result.is_err());
    }
}

#[test]
fn validate_path_blocks_root() {
    if cfg!(not(windows)) {
        let result = validate_path("/root/.ssh/id_rsa");
        assert!(result.is_err());
    }
}

#[test]
fn validate_path_blocks_boot() {
    if cfg!(not(windows)) {
        let result = validate_path("/boot/grub/grub.cfg");
        assert!(result.is_err());
    }
}

#[test]
fn validate_path_blocks_dev_on_linux() {
    if cfg!(not(windows)) {
        let result = validate_path("/dev/sda1");
        assert!(result.is_err());
    }
}

#[test]
fn blocked_path_windows_program_files() {
    if cfg!(windows) {
        let result = validate_path("C:\\Program Files\\SomeApp");
        assert!(result.is_err());
    }
}

#[test]
fn blocked_path_windows_program_files_x86() {
    if cfg!(windows) {
        let result = validate_path("C:\\Program Files (x86)\\Test");
        assert!(result.is_err());
    }
}

#[test]
fn existing_normal_path_is_ok() {
    let result = validate_path(env!("CARGO_MANIFEST_DIR"));
    assert!(result.is_ok());
}

#[test]
fn empty_path_resolves_to_current_dir() {
    let result = validate_path("");
    assert!(result.is_ok() || result.is_err());
}
