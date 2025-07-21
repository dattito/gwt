use std::path::Path;
use std::process::{Command, Stdio};

pub fn allow_direnv(worktree_path: &Path) -> Result<(), String> {
    Command::new("direnv")
        .arg("allow")
        .arg(worktree_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("Failed to run direnv allow: {e}"))?;
    Ok(())
}
