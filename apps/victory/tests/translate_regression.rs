use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_victory")
}

#[test]
fn translate_empty_dir_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(bin())
        .args(["translate", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "translate failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("SPIR-V"), "missing SPIR-V summary: {stdout}");
}

#[test]
fn translate_cache_empty_dir_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(bin())
        .args(["translate", "--cache", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "translate --cache failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn translate_validate_reports_spirv_val() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(bin())
        .args(["translate", "--cache", "--validate", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("SPIRV-Tools") || stdout.contains("spirv-val"),
        "missing validator line: {stdout}"
    );
}

#[test]
fn translate_list_empty_dir_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(bin())
        .args(["translate", "--list", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
}