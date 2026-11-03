// src/git.rs
use anyhow::{Result, anyhow};
use std::process::Command;

pub struct GitManager;

impl GitManager {
    /// Checks if current working directory is a git repository
    #[allow(dead_code)]
    pub fn is_git_repo() -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// `git status` summary
    pub fn status() -> Result<String> {
        let output = Command::new("git").args(["status", "-s"]).output()?;
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        if text.trim().is_empty() {
            Ok("Working tree clean (No changes)".to_string())
        } else {
            Ok(text)
        }
    }

    /// `git new repo` (git init)
    pub fn init_new_repo() -> Result<String> {
        let output = Command::new("git").arg("init").output()?;
        if output.status.success() {
            Ok("Successfully initialized a new Git repository!".to_string())
        } else {
            Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()))
        }
    }

    /// `git commit -tags "message" branch "branch"` (Smart Quick Save)
    pub fn quick_save(message: &str, branch: &str) -> Result<String> {
        // 1. Stage all files
        let add_output = Command::new("git").args(["add", "."]).output()?;
        if !add_output.status.success() {
            return Err(anyhow!("Failed to stage files: {}", String::from_utf8_lossy(&add_output.stderr)));
        }

        // 2. Commit
        let commit_msg = if message.is_empty() { "Update via ELIDE" } else { message };
        let commit_output = Command::new("git").args(["commit", "-m", commit_msg]).output()?;
        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            if !stderr.contains("nothing to commit") {
                return Err(anyhow!("Commit failed: {}", stderr));
            }
        }

        // 3. Push to branch
        let mut push_args = vec!["push"];
        let target_branch = branch.trim();
        if !target_branch.is_empty() {
            push_args.push("origin");
            push_args.push(target_branch);
        }

        let push_output = Command::new("git").args(&push_args).output()?;
        if !push_output.status.success() {
            return Err(anyhow!("Push failed: {}", String::from_utf8_lossy(&push_output.stderr)));
        }

        let branch_name = if target_branch.is_empty() { "default branch" } else { target_branch };
        Ok(format!("Successfully committed & pushed to '{}': \"{}\"", branch_name, commit_msg))
    }

    /// `git login` (GitHub CLI Auth check helper)
    pub fn login_check() -> Result<String> {
        let output = Command::new("gh").args(["auth", "status"]).output();
        match output {
            Ok(o) if o.status.success() => Ok("GitHub CLI session active and authenticated!".to_string()),
            _ => Ok("To login, run 'gh auth login' in your system terminal or set up your git credential helper.".to_string()),
        }
    }

    /// `git choose repo` (Scans subdirectories for local repositories)
    pub fn choose_repo() -> Result<String> {
        let mut repos = Vec::new();
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if entry.path().is_dir() && entry.path().join(".git").exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        repos.push(name.to_string());
                    }
                }
            }
        }
        if repos.is_empty() {
            Ok("No sub-repositories found in current workspace.".to_string())
        } else {
            Ok(format!("Detected local repositories:\n{}", repos.join("\n")))
        }
    }
}