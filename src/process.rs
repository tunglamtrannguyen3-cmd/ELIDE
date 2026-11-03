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
use nix::sys::resource::{getrusage, UsageWho};
use std::{
    fs::OpenOptions,
    io::{stdout, Write},
    time::Duration,
};
use tokio::{
    io::AsyncBufReadExt,
    process::Command,
};
use std::process::Stdio;

// ==========================================
// 1. PROCESS EXECUTION & RESOURCE TRACKING
// ==========================================

#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub success: bool,
    pub output: String,
}

pub async fn run_bash_cmd(cmd_str: &str) -> Result<ProcessResult> {
    let child = Command::new("/bin/bash")
        .arg("-c")
        .arg(cmd_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output().await?;
    let success = output.status.success();

    let max_rss_kb = match getrusage(UsageWho::RUSAGE_CHILDREN) {
        Ok(usage) => usage.max_rss(),
        Err(_) => 0,
    };

    let text = if success {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    let formatted_output = if max_rss_kb > 0 {
        format!("{}\n[Kernel Stats: Peak RAM {} KB]", text.trim(), max_rss_kb)
    } else {
        text.trim().to_string()
    };

    Ok(ProcessResult {
        success,
        output: formatted_output,
    })
}

pub async fn run_interactive_cmd(cmd_str: &str) -> Result<()> {
    // 1. Temporarily surrender the terminal to the child app
    let _ = TerminalGuard::suspend();

    // 2. Spawn the process interactively (inheriting stdin/stdout/stderr)
    let mut child = Command::new("/bin/bash")
        .arg("-c")
        .arg(cmd_str)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // 3. Wait for the task or game to complete
    let status = child.wait().await?;

    // 4. Pause execution so developers can read compiler/build output
    println!("\n\x1b[32m[Process completed with exit status: {}]\x1b[0m", status);
    println!("\x1b[33mPress ENTER to return to ELIDE...\x1b[0m");
    
    let mut pause_buf = String::new();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let _ = stdin.read_line(&mut pause_buf).await;

    // 5. Reclaim the terminal UI for ELIDE
    let _ = TerminalGuard::resume();

    Ok(())
}

// ==========================================
// 2. TERMINAL MANAGEMENT & INPUT EVENTS
// ==========================================

#[allow(dead_code)]
pub enum InputEvent {
    Key(crossterm::event::KeyEvent),
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

    /// Call this right BEFORE spawning a foreground process
    pub fn suspend() -> Result<()> {
        stdout().execute(DisableBracketedPaste)?;
        stdout().execute(DisableMouseCapture)?;
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    /// Call this right AFTER the foreground process exits
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
        // Clear the log file on startup
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