use colored::*;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn get_git_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .map_err(|e| format!("Failed to execute git command: {e}"))?;

    if !output.status.success() {
        return Err("Not in a git repository".to_string());
    }

    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

pub fn pull_latest() -> Result<(), String> {
    let output = Command::new("git")
        .arg("pull")
        .output()
        .map_err(|e| format!("Failed to execute git pull: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("There is no tracking information for the current branch.") {
            eprintln!(
                "{} Unable to run git pull, there may not be an upstream",
                "Warning:".yellow()
            );
        }
    }
    Ok(())
}

pub fn create_worktree(branch_name: &str, dirname: &str) -> Result<(), String> {
    let worktree_path = format!("../{dirname}");
    let local_branch_exists_output = Command::new("git")
        .arg("for-each-ref")
        .arg("--format=%(refname:lstrip=2)")
        .arg("refs/heads")
        .output()
        .map_err(|e| format!("Failed to check for local branches: {e}"))?;
    let local_branch_exists = local_branch_exists_output.stdout;

    let remote_branch_exists_output = Command::new("git")
        .arg("for-each-ref")
        .arg("--format=%(refname:lstrip=3)")
        .arg("refs/remotes/origin")
        .output()
        .map_err(|e| format!("Failed to check for remote branches: {e}"))?;
    let remote_branch_exists = remote_branch_exists_output.stdout;

    let mut cmd = Command::new("git");
    cmd.arg("worktree").arg("add");

    if String::from_utf8_lossy(&local_branch_exists).contains(branch_name)
        || String::from_utf8_lossy(&remote_branch_exists).contains(branch_name)
    {
        cmd.arg(&worktree_path).arg(branch_name);
    } else {
        cmd.arg("-b").arg(branch_name).arg(&worktree_path);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to create git worktree: {e}"))?;

    if !status.success() {
        return Err(format!(
            "Failed to create git worktree for branch '{branch_name}'"
        ));
    }

    Ok(())
}

pub fn get_worktrees() -> Result<Vec<PathBuf>, String> {
    let output = Command::new("git")
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .output()
        .map_err(|e| format!("Failed to list worktrees: {e}"))?;

    if !output.status.success() {
        return Err("Failed to list git worktrees".to_string());
    }

    let mut worktrees = Vec::new();
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.starts_with("worktree ") {
            worktrees.push(PathBuf::from(line.split_at(9).1));
        }
    }
    Ok(worktrees)
}

pub fn get_default_branch() -> Result<String, String> {
    // Try to get the HEAD branch from `git remote show origin`
    let output = Command::new("git")
        .arg("remote")
        .arg("show")
        .arg("origin")
        .output()
        .map_err(|e| format!("Failed to execute git remote show origin: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim().starts_with("HEAD branch:") {
                return Ok(line.split(':').nth(1).unwrap().trim().to_string());
            }
        }
    }

    // Fallback to common branch names
    let common_branches = vec!["main", "master"];
    for branch in common_branches {
        let output = Command::new("git")
            .arg("show-ref")
            .arg(format!("refs/remotes/origin/{branch}"))
            .output()
            .map_err(|e| format!("Failed to execute git show-ref: {e}"))?;
        if output.status.success() {
            return Ok(branch.to_string());
        }
    }

    Err("Could not determine default branch.".to_string())
}

pub fn branch_has_changes() -> Result<bool, String> {
    let output = Command::new("git")
        .arg("status")
        .arg("-s")
        .output()
        .map_err(|e| format!("Failed to execute git status -s: {e}"))?;

    let has_changes = !output.stdout.is_empty();

    Ok(has_changes)
}

pub fn remove_worktree(dirname: &str) -> Result<(), String> {
    Command::new("git")
        .arg("worktree")
        .arg("remove")
        .arg(dirname)
        .output()
        .map_err(|e| format!("Failed to execute git worktree remove: {e}"))?;

    Ok(())
}

pub fn delete_branch(branch_name: &str) -> Result<(), String> {
    let output = Command::new("git")
        .arg("branch")
        .arg("-d")
        .arg(branch_name)
        .output()
        .map_err(|e| format!("Failed to execute git branch -d: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("is not fully merged") {
            return Err(format!(
                "The branch {} is not fully merged and will not be deleted",
                branch_name.green(),
            ));
        }
        return Err(format!(
            "Could not delete branch {}: {}",
            branch_name.green(),
            stderr,
        ));
    }

    Ok(())
}
