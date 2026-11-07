// src/git.rs
use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

pub struct GitManager;

impl GitManager {
    /// Helper to execute a git command and return stdout as a trimmed string
    fn run_git(args: &[&str]) -> Result<String> {
        let output = Command::new("git").args(args).output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(anyhow!(stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Checks if current working directory is a git repository
    pub fn is_git_repo() -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// `git status` summary with repository check guard
    pub fn status() -> Result<String> {
        if !Self::is_git_repo() {
            return Err(anyhow!("Not a git repository (or any of the parent directories)."));
        }

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

    /// List all local branches
    pub fn list_branches() -> Result<String> {
        let output = Self::run_git(&["branch", "-a"])?;
        Ok(format!("🌿 [WORKSPACE BRANCHES]\n{}", output))
    }

    /// Switch or checkout to a specific branch (creates it with -b if requested or needed)
    pub fn switch_branch(branch_name: &str, create_new: bool) -> Result<String> {
        let branch_name = branch_name.trim();
        if branch_name.is_empty() {
            return Err(anyhow!("Error: Specify a branch name (e.g. git switch main)."));
        }

        let res = if create_new {
            Self::run_git(&["checkout", "-b", branch_name])
        } else {
            Self::run_git(&["switch", branch_name]).or_else(|_| Self::run_git(&["checkout", branch_name]))
        };

        match res {
            Ok(_) => Ok(format!("🌿 Successfully switched to branch: '{}'", branch_name)),
            Err(e) => Err(anyhow!("Failed to switch branch '{}': {}", branch_name, e)),
        }
    }

    /// Smart Quick Save: Stages specific files or all changes, commits locally, and pushes with fallback
    pub fn quick_save(message: &str, branch: &str, specific_file: Option<&str>) -> Result<String> {
        // 1. Stage files (specific file or all)
        if let Some(file) = specific_file {
            let file = file.trim();
            if !file.is_empty() {
                Self::run_git(&["add", file]).map_err(|e| anyhow!("Failed to stage file '{}': {}", file, e))?;
            } else {
                Self::run_git(&["add", "."]).map_err(|e| anyhow!("Failed to stage files: {}", e))?;
            }
        } else {
            Self::run_git(&["add", "."]).map_err(|e| anyhow!("Failed to stage files: {}", e))?;
        }

        // 2. Commit changes (check both stdout and stderr for "nothing to commit")
        let commit_msg = if message.is_empty() { "Update via ELIDE" } else { message };
        let commit_output = Command::new("git").args(["commit", "-m", commit_msg]).output()?;
        if !commit_output.status.success() {
            let stdout = String::from_utf8_lossy(&commit_output.stdout);
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            let combined_output = format!("{}\n{}", stdout, stderr);
            
            if !combined_output.contains("nothing to commit") {
                return Err(anyhow!("Commit failed: {}", stderr.trim()));
            }
        }

        // 3. Determine target branch
        let target_branch = if branch.trim().is_empty() {
            let branch_output = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output();
            match branch_output {
                Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
                _ => "main".to_string(),
            }
        } else {
            branch.trim().to_string()
        };

        // 4. Attempt push with fallback handling
        let push_output = Command::new("git")
            .args(["push", "-u", "origin", &target_branch])
            .output();

        match push_output {
            Ok(out) if out.status.success() => {
                Ok(format!("Successfully committed & pushed to '{}': \"{}\"", target_branch, commit_msg))
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let reason = if stderr.is_empty() { "No remote repository or network connection." } else { &stderr };
                Ok(format!(
                    "Committed locally as \"{}\", but remote push fell back to local-only mode.\nReason: {}",
                    commit_msg, reason
                ))
            }
            Err(e) => {
                Ok(format!("Committed locally as \"{}\", but push command failed to execute: {}", commit_msg, e))
            }
        }
    }

    /// `git fetch` updates remote tracking branches
    pub fn fetch() -> Result<String> {
        let output = Self::run_git(&["fetch", "--all", "--prune"])?;
        if output.is_empty() {
            Ok("Successfully fetched all remotes (up to date).".to_string())
        } else {
            Ok(format!("Successfully fetched remotes:\n{}", output))
        }
    }

    /// `git pull` with support for optional branch targeting and rebase flag
    pub fn pull(branch: Option<&str>, rebase: bool) -> Result<String> {
        let mut args = vec!["pull"];
        if rebase {
            args.push("--rebase");
        }
        
        let target_branch = branch.map(|s| s.trim()).filter(|s| !s.is_empty());
        if let Some(b) = target_branch {
            args.push("origin");
            args.push(b);
        }

        match Self::run_git(&args) {
            Ok(out) => {
                let msg = if out.is_empty() { "Already up to date." } else { &out };
                Ok(format!("Successfully pulled latest changes: {}", msg))
            }
            Err(e) => {
                Err(anyhow!(
                    "Pull failed: {}\n(Hint: check for local uncommitted changes or merge conflicts)", 
                    e
                ))
            }
        }
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

// ==========================================
// UNIT TESTS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_repo() {
        // Just verify it doesn't panic and returns a boolean
        let _res = GitManager::is_git_repo();
    }

    #[test]
    fn test_switch_branch_empty() {
        let res = GitManager::switch_branch("", false);
        assert!(res.is_err());
    }

    #[test]
    fn test_choose_repo_nonexistent() {
        let res = GitManager::choose_repo(Some("nonexistent_repo_abc_123"));
        assert!(res.is_err());
    }
}