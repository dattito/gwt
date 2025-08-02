use colored::*;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use crate::config::get_files_from_config;
use crate::direnv_utils::allow_direnv;
use crate::file_ops::{copy_files_from_config, cp_cow, link_files_from_config};
use crate::git_utils::{
    create_worktree, get_default_branch, get_git_root, get_worktrees, pull_latest,
};

pub fn add_worktree(
    branch_name: &str,
    copy: bool,
    verbose: bool,
    pull: bool,
) -> Result<(), String> {
    if verbose {
        println!("Verbose mode enabled");
    }

    let dirname = branch_name.replace('/', "_");

    let git_root = get_git_root()?;
    env::set_current_dir(&git_root)
        .map_err(|e| format!("Failed to change to git root directory: {e}"))?;

    if pull {
        pull_latest()?;
    }

    create_worktree(branch_name, &dirname)?;

    let worktree_path = git_root.join(format!("../{dirname}"));
    let worktree_path = fs::canonicalize(&worktree_path).map_err(|e| {
        format!(
            "Failed to canonicalize worktree path '{}': {e}",
            worktree_path.display()
        )
    })?;

    if copy {
        copy_files_from_config(&worktree_path)?;
    } else {
        link_files_from_config(&worktree_path, &git_root)?;
    }

    if worktree_path.join(".envrc").exists() {
        allow_direnv(&worktree_path)?;
    }

    println!("{}", worktree_path.display());
    Ok(())
}

pub fn sync_worktrees(copy_flag: bool) -> Result<(), String> {
    let git_root = get_git_root()?;
    env::set_current_dir(&git_root)
        .map_err(|e| format!("Failed to change to git root directory: {e}"))?;
    let worktrees = get_worktrees()?;
    let config_path = git_root.join(".gwtconfig");
    let files_to_sync = get_files_from_config(&config_path)?;

    if files_to_sync.is_empty() {
        println!(
            "{} No .gwtconfig file found or it is empty. No files to sync.",
            "Info:".green()
        );
        return Ok(());
    }

    for item in files_to_sync {
        let mut most_recent_path: Option<PathBuf> = None;
        let mut most_recent_time: Option<SystemTime> = None;

        for worktree in &worktrees {
            let path = worktree.join(&item);
            if path.exists() {
                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Failed to get metadata for {}: {e}", path.display()))?;
                let modified_time = metadata.modified().map_err(|e| {
                    format!("Failed to get modified time for {}: {e}", path.display())
                })?;

                if most_recent_time.is_none() || modified_time > most_recent_time.unwrap() {
                    most_recent_time = Some(modified_time);
                    most_recent_path = Some(path);
                }
            }
        }

        if let Some(src_path) = most_recent_path {
            for worktree in &worktrees {
                let dest_path = worktree.join(&item);
                if src_path.as_path() != dest_path.as_path() {
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            format!("Failed to create directory {}: {e}", parent.display())
                        })?;
                    }

                    if copy_flag {
                        cp_cow(&src_path, &dest_path)?;
                        println!(
                            "{} Synced '{}' to {} (copied)",
                            "Info:".green(),
                            item,
                            worktree.display()
                        );
                    } else {
                        // Attempt to symlink first
                        let symlink_result = symlink(&src_path, &dest_path);
                        if symlink_result.is_ok() {
                            println!(
                                "{} Synced '{}' to {} (linked)",
                                "Info:".green(),
                                item,
                                worktree.display()
                            );
                        } else {
                            // Fallback to copy if symlink fails
                            eprintln!(
                                "{} Failed to symlink '{}' to {} ({:?}). Falling back to copy.",
                                "Warning:".yellow(),
                                item,
                                worktree.display(),
                                symlink_result.unwrap_err()
                            );
                            cp_cow(&src_path, &dest_path)?;
                            println!(
                                "{} Synced '{}' to {} (copied)",
                                "Info:".green(),
                                item,
                                worktree.display()
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn clone_repo(repo: &str) -> Result<(), String> {
    let repo_name = repo.split('/').next_back().unwrap_or(repo);
    println!("Cloning into '{repo_name}'...");

    fs::create_dir(repo_name)
        .map_err(|e| format!("Failed to create directory {repo_name}: {e}"))?;
    env::set_current_dir(repo_name).map_err(|e| format!("Failed to cd into {repo_name}: {e}"))?;

    let clone_status = Command::new("gh")
        .arg("repo")
        .arg("clone")
        .arg(repo)
        .arg(".bare")
        .arg("--")
        .arg("--bare")
        .status()
        .map_err(|e| format!("Failed to execute gh repo clone: {e}"))?;

    if !clone_status.success() {
        return Err("Failed to clone repository".to_string());
    }

    fs::write(".git", "gitdir: ./.bare").map_err(|e| format!("Failed to write .git file: {e}"))?;

    let default_branch = get_default_branch()?;

    println!("Adding worktree '{default_branch}' for branch '{default_branch}'");
    let worktree_status = Command::new("git")
        .arg("worktree")
        .arg("add")
        .arg(&default_branch)
        .arg(&default_branch)
        .status()
        .map_err(|e| format!("Failed to create worktree: {e}"))?;

    if !worktree_status.success() {
        return Err(format!("Failed to create '{default_branch}' worktree"));
    }

    println!(
        "{} Successfully cloned {} and set up worktree in '{}/{}'",
        "Success:".green(),
        repo,
        repo_name,
        default_branch
    );
    Ok(())
}

pub fn init_gwtconfig() -> Result<(), String> {
    let git_root = get_git_root()?;
    let gitignore_path = git_root.join(".gitignore");
    let gwtconfig_path = git_root.join(".gwtconfig");

    if !gitignore_path.exists() {
        return Err("No .gitignore file found in the repository root.".to_string());
    }

    println!("Reading .gitignore and suggesting patterns for .gwtconfig...");

    let gitignore_content = fs::read_to_string(&gitignore_path)
        .map_err(|e| format!("Failed to read .gitignore: {e}"))?;

    let mut selected_items: Vec<String> = Vec::new();

    for line in gitignore_content.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() || trimmed_line.starts_with("#") {
            continue;
        }

        print!("Should '{trimmed_line}' be added to .gwtconfig? (y/N): ");
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stdout: {e}"))?;

        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|e| format!("Failed to read line: {e}"))?;

        if answer.trim().eq_ignore_ascii_case("y") {
            selected_items.push(trimmed_line.to_string());
        }
    }

    if selected_items.is_empty() {
        println!("{} No items selected for .gwtconfig.", "Info:".green());
        return Ok(());
    }

    let content_to_write = selected_items.join("\n") + "\n";
    fs::write(&gwtconfig_path, content_to_write)
        .map_err(|e| format!("Failed to write .gwtconfig: {e}"))?;

    println!(
        "{} .gwtconfig created/updated at {}.",
        "Success:".green(),
        gwtconfig_path.display()
    );

    Ok(())
}
