use std::process::Command;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=.git/HEAD");
    if let Ok(head) = std::fs::read_to_string(".git/HEAD") {
        if let Some(ref_path) = head.strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed=.git/{}", ref_path.trim());
        }
    }

    let major: u16 = std::env::var("CARGO_PKG_VERSION_MAJOR")
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);
    let minor: u16 = std::env::var("CARGO_PKG_VERSION_MINOR")
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);
    let patch: u16 = std::env::var("CARGO_PKG_VERSION_PATCH")
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);

    let commit_count: u16 = git_cmd(&["rev-list", "--count", "HEAD"])
        .and_then(|s| s.parse::<u64>().ok())
        .map(|c| c.min(u16::MAX as u64) as u16)
        .unwrap_or(0);

    let git_hash =
        git_cmd(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "nogit".to_string());

    let is_dirty = git_cmd(&["status", "--porcelain"])
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    let packed_version = ((major as u64) << 48)
        | ((minor as u64) << 32)
        | ((patch as u64) << 16)
        | (commit_count as u64);

    let dirty_flag = if is_dirty { "-dirty" } else { "" };
    let display_version =
        format!("{major}.{minor}.{patch}.{commit_count} ({git_hash}{dirty_flag})");

    println!("cargo:rustc-env=FULL_DLL_VERSION={display_version}");

    let mut res = winresource::WindowsResource::new();
    res.set_version_info(winresource::VersionInfo::FILEVERSION, packed_version)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, packed_version)
        .set("FileVersion", &display_version)
        .set("ProductVersion", &display_version);

    res.compile().expect("Failed to compile version resource.");
}

fn git_cmd(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
