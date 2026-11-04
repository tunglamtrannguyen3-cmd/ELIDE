// src/process.rs
use anyhow::Result;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
        EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{stdout, Write},
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::AsyncBufReadExt,
    process::Command,
};

#[cfg(unix)]
use nix::sys::resource::{getrusage, UsageWho};

// ==========================================
// 1. PROCESS EXECUTION & RESOURCE TRACKING
// ==========================================

#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub success: bool,
    pub output: String,
}

pub async fn run_bash_cmd(cmd_str: &str) -> Result<ProcessResult> {
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        &std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    };

    let arg_flag = if cfg!(target_os = "windows") { "/C" } else { "-c" };

    let child = Command::new(shell)
        .arg(arg_flag)
        .arg(cmd_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output().await?;
    let success = output.status.success();

    #[cfg(unix)]
    let max_rss_kb = match getrusage(UsageWho::RUSAGE_CHILDREN) {
        Ok(usage) => usage.max_rss(),
        Err(_) => 0,
    };
    #[cfg(not(unix))]
    let max_rss_kb: i64 = 0;

    let stdout_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let text = match (stdout_str.is_empty(), stderr_str.is_empty()) {
        (false, false) => format!("{}\n{}", stdout_str, stderr_str),
        (false, true) => stdout_str,
        (true, false) => stderr_str,
        (true, true) => String::new(),
    };

    let formatted_output = if max_rss_kb > 0 {
        format!("{}\n[Kernel Stats: Peak RAM {} KB]", text, max_rss_kb)
    } else {
        text
    };

    Ok(ProcessResult {
        success,
        output: formatted_output,
    })
}

pub async fn run_interactive_cmd(cmd_str: &str) -> Result<()> {
    TerminalGuard::suspend()?;

    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        &std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    };
    let arg_flag = if cfg!(target_os = "windows") { "/C" } else { "-c" };

    // Ensure terminal is resumed even if spawning or waiting fails
    let execution_result = async {
        let mut child = Command::new(shell)
            .arg(arg_flag)
            .arg(cmd_str)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let status = child.wait().await?;
        Ok::<std::process::ExitStatus, anyhow::Error>(status)
    }
    .await;

    TerminalGuard::resume()?;

    let status = execution_result?;

    println!("\n\x1b[32m[Process completed with exit status: {}]\x1b[0m", status);
    println!("\x1b[33mPress ENTER to return to ELIDE...\x1b[0m");
    
    let mut pause_buf = String::new();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let _ = stdin.read_line(&mut pause_buf).await;

    Ok(())
}

// ==========================================
// 2. TERMINAL MANAGEMENT & INPUT EVENTS
// ==========================================

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum InputEvent {
    Key(KeyEvent),
    Paste(String),
    Resize,
    ScrollUp,
    ScrollDown,
    Tick,
}

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn init() -> Result<Self> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(EnableMouseCapture)?;
        stdout().execute(EnableBracketedPaste)?;
        Ok(Self)
    }

    pub fn suspend() -> Result<()> {
        stdout().execute(DisableBracketedPaste)?;
        stdout().execute(DisableMouseCapture)?;
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn resume() -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(EnableMouseCapture)?;
        stdout().execute(EnableBracketedPaste)?;
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = TerminalGuard::suspend();
    }
}

#[allow(dead_code)]
pub fn poll_event(timeout: Duration) -> Result<InputEvent> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(InputEvent::Key(key)),
            Event::Paste(s) => Ok(InputEvent::Paste(s)),
            Event::Resize(_, _) => Ok(InputEvent::Resize),
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => Ok(InputEvent::ScrollUp),
                MouseEventKind::ScrollDown => Ok(InputEvent::ScrollDown),
                _ => Ok(InputEvent::Tick),
            },
            _ => Ok(InputEvent::Tick),
        }
    } else {
        Ok(InputEvent::Tick)
    }
}

pub fn is_alt_t(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('t')
}

pub fn is_alt_l(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('l')
}

pub fn is_esc(key: &KeyEvent) -> bool {
    key.code == KeyCode::Esc
}

// ==========================================
// 3. KERNEL TRACER & DEBUG LOGGING
// ==========================================

#[derive(Debug, Clone)]
pub struct KernelTracer {
    pub active: bool,
}

impl KernelTracer {
    pub fn init() -> Self {
        let _ = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("elide_debug.log");
            
        Self { active: true }
    }

    pub fn log_event(&self, event: &str) {
        if self.active {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("elide_debug.log") {
                let _ = writeln!(file, "[DEBUG] {}", event);
            }
        }
    }
}

// ==========================================
// 4. INTERACTIVE WORKSPACE FILE TREE SIDEBAR
// ==========================================

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
}

pub struct FileTree {
    pub root_path: PathBuf,
    pub nodes: Vec<TreeNode>,
    pub selected_idx: usize,
    pub expanded_dirs: HashSet<PathBuf>,
    pub is_visible: bool,
}

impl FileTree {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root_path = root.as_ref().to_path_buf();
        let mut tree = Self {
            root_path: root_path.clone(),
            nodes: Vec::new(),
            selected_idx: 0,
            expanded_dirs: HashSet::new(),
            is_visible: false,
        };
        // Expand root by default
        tree.expanded_dirs.insert(root_path.clone());
        tree.refresh();
        tree
    }

    /// Recursively build the flat list of visible nodes based on expansion state
    pub fn refresh(&mut self) {
        self.nodes.clear();
        let expanded = self.expanded_dirs.clone();
        let root_path = self.root_path.clone();
        if root_path.exists() {
            self.build_nodes(&root_path, 0, &expanded);
        }
        if self.nodes.is_empty() {
            self.selected_idx = 0;
        } else {
            self.selected_idx = self.selected_idx.min(self.nodes.len() - 1);
        }
    }

    fn build_nodes(&mut self, dir: &Path, depth: usize, expanded: &HashSet<PathBuf>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return, // Gracefully skip unreadable directories / permission errors
        };

        let mut items: Vec<(PathBuf, String, bool)> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            
            // Skip hidden files and heavy build/dependency artifacts
            if name.starts_with('.') || name == "target" || name == "node_modules" || name == "build" {
                continue;
            }

            let is_dir = path.is_dir();
            items.push((path, name, is_dir));
        }

        // Sort directories first, then files alphabetically (case-insensitive)
        items.sort_by(|a, b| {
            match b.2.cmp(&a.2) {
                std::cmp::Ordering::Equal => a.1.to_lowercase().cmp(&b.1.to_lowercase()),
                other => other,
            }
        });

        for (path, name, is_dir) in items {
            let is_expanded = is_dir && expanded.contains(&path);

            self.nodes.push(TreeNode {
                path: path.clone(),
                name,
                is_dir,
                depth,
            });

            if is_expanded {
                self.build_nodes(&path, depth + 1, expanded);
            }
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.nodes.is_empty() {
            return;
        }
        let new_idx = (self.selected_idx as isize + delta).clamp(0, self.nodes.len() as isize - 1);
        self.selected_idx = new_idx as usize;
    }

    pub fn toggle_selected(&mut self) -> Option<PathBuf> {
        let (path, is_dir) = {
            let node = self.nodes.get(self.selected_idx)?;
            (node.path.clone(), node.is_dir)
        };

        if is_dir {
            if self.expanded_dirs.contains(&path) {
                self.expanded_dirs.remove(&path);
            } else {
                self.expanded_dirs.insert(path);
            }
            self.refresh();
            None
        } else {
            Some(path)
        }
    }
}