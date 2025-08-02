use colored::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::get_files_from_config;
use crate::git_utils::get_git_root;

pub fn link_files_from_config(worktree_path: &Path, git_root: &Path) -> Result<(), String> {
    let config_path = git_root.join(".gwtconfig");
    let files_to_link = get_files_from_config(&config_path)?;

    if files_to_link.is_empty() {
        println!(
            "{} No .gwtconfig file found or it is empty. No files will be linked.",
            "Info:".green()
        );
        return Ok(());
    }

    for item_str in files_to_link {
        let item_trimmed = item_str.trim_end_matches('/');
        let src_path_abs = git_root.join(item_trimmed);

        if src_path_abs.exists() {
            // Check if it's a broken symlink
            if src_path_abs.is_symlink() && fs::metadata(&src_path_abs).is_err() {
                eprintln!(
                    "{} Source path '{}' is a broken symlink, skipping.",
                    "Warning:".yellow(),
                    src_path_abs.display()
                );
                continue;
            }

            let dest_path_abs = worktree_path.join(item_trimmed);

            if let Some(parent) = dest_path_abs.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
            }

            // Remove existing file/directory/symlink at destination if it exists
            if dest_path_abs.symlink_metadata().is_ok() {
                if dest_path_abs.is_dir() {
                    fs::remove_dir_all(&dest_path_abs).map_err(|e| {
                        format!(
                            "Failed to remove directory {}: {e}",
                            dest_path_abs.display()
                        )
                    })?;
                } else {
                    fs::remove_file(&dest_path_abs).map_err(|e| {
                        format!("Failed to remove file {}: {e}", dest_path_abs.display())
                    })?;
                }
            }

            // Use absolute paths for symlink
            symlink(&src_path_abs, &dest_path_abs).map_err(|e| {
                format!(
                    "Failed to create symlink from {} to {}: {e}",
                    src_path_abs.display(),
                    dest_path_abs.display()
                )
            })?;
            println!(
                "{} Linked '{}' to new worktree.",
                "Info:".green(),
                item_trimmed
            );
        } else {
            eprintln!(
                "{} File or directory '{}' not found, skipping.",
                "Warning:".yellow(),
                item_str
            );
        }
    }

    Ok(())
}

pub fn copy_files_from_config(worktree_path: &Path) -> Result<(), String> {
    let git_root = get_git_root()?;
    let config_path = git_root.join(".gwtconfig");
    let files_to_copy = get_files_from_config(&config_path)?;

    if files_to_copy.is_empty() {
        println!(
            "{} No .gwtconfig file found or it is empty. No files will be copied.",
            "Info:".green()
        );
        return Ok(());
    }

    for item in files_to_copy {
        let src_path = PathBuf::from(&item);
        if src_path.exists() {
            let dest_path = worktree_path.join(&item);
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
            }
            cp_cow(&src_path, &dest_path)?;
            println!("{} Copied '{}' to new worktree.", "Info:".green(), item);
        } else {
            eprintln!(
                "{} File or directory '{}' not found, skipping.",
                "Warning:".yellow(),
                item
            );
        }
    }

    Ok(())
}

pub fn cp_cow(src: &Path, dest: &Path) -> Result<(), String> {
    // Try cp with copy-on-write (macOS)
    let mut cmd = Command::new("cp");
    cmd.arg("-Rc").arg(src).arg(dest);
    if cmd.status().is_ok_and(|s| s.success()) {
        return Ok(());
    }

    // Try cp with copy-on-write (GNU)
    let mut cmd = Command::new("cp");
    cmd.arg("-R").arg("--reflink=auto").arg(src).arg(dest);
    if cmd.status().is_ok_and(|s| s.success()) {
        return Ok(());
    }

    // Fallback to standard recursive copy
    let mut cmd = Command::new("cp");
    cmd.arg("-R").arg(src).arg(dest);
    if cmd.status().is_ok_and(|s| s.success()) {
        return Ok(());
    }

    Err(format!(
        "{} Unable to copy {} to {}",
        "Warning:".yellow(),
        src.display(),
        dest.display()
    ))
}
