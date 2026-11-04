// src/git.rs
use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::Path;

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
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Git status failed: {}", stderr.trim()));
        }
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

    /// Smart Quick Save: Stages all files, commits with message, and pushes to remote branch
    pub fn quick_save(message: &str, branch: &str) -> Result<String> {
        // 1. Stage all files
        let add_output = Command::new("git").args(["add", "."]).output()?;
        if !add_output.status.success() {
            return Err(anyhow!("Failed to stage files: {}", String::from_utf8_lossy(&add_output.stderr)));
        }

        // 2. Commit changes
        let commit_msg = if message.is_empty() { "Update via ELIDE" } else { message };
        let commit_output = Command::new("git").args(["commit", "-m", commit_msg]).output()?;
        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            if !stderr.contains("nothing to commit") {
                return Err(anyhow!("Commit failed: {}", stderr));
            }
        }

        // 3. Determine target branch (fallback to current active branch if not specified)
        let target_branch = if branch.trim().is_empty() {
            let branch_output = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output();
            match branch_output {
                Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
                _ => "main".to_string(),
            }
        } else {
            branch.trim().to_string()
        };

        // 4. Push to remote with upstream tracking
        let push_output = Command::new("git")
            .args(["push", "-u", "origin", &target_branch])
            .output()?;

        if !push_output.status.success() {
            return Err(anyhow!("Push failed: {}", String::from_utf8_lossy(&push_output.stderr)));
        }

        Ok(format!("Successfully committed & pushed to '{}': \"{}\"", target_branch, commit_msg))
    }

    /// `git login` (GitHub CLI Auth check helper)
    pub fn login_check() -> Result<String> {
        let output = Command::new("gh").args(["auth", "status"]).output();
        match output {
            Ok(o) if o.status.success() => Ok("GitHub CLI session active and authenticated!".to_string()),
            _ => Ok("To login, run 'gh auth login' in your system terminal or set up your git credential helper.".to_string()),
        }
    }

    /// `git choose repo` (Scans subdirectories for local repositories or switches to one)
    pub fn choose_repo(target_repo: Option<&str>) -> Result<String> {
        let mut repos = Vec::new();
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join(".git").exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        repos.push(name.to_string());
                    }
                }
            }
        }

        if let Some(repo_name) = target_repo {
            let repo_name = repo_name.trim();
            if repo_name.is_empty() {
                return Ok(format!("Detected local repositories:\n{}", repos.join("\n")));
            }
            if repos.contains(&repo_name.to_string()) {
                std::env::set_current_dir(Path::new(repo_name))?;
                Ok(format!("Successfully switched workspace to repository: '{}'", repo_name))
            } else {
                Err(anyhow!("Repository '{}' not found in current directory.", repo_name))
            }
        } else {
            if repos.is_empty() {
                Ok("No sub-repositories found in current workspace.".to_string())
            } else {
                Ok(format!("Detected local repositories:\n{}\n\nUse 'git choose repo <repo_name>' to switch.", repos.join("\n")))
            }
        }
    }
}