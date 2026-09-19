use anyhow::Result;
use nix::sys::resource::{getrusage, UsageWho};
use std::process::Stdio;
use tokio::io::AsyncBufReadExt; // Required for async read_line
use tokio::process::Command;

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
    let _ = crate::terminal::TerminalGuard::suspend();

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
    let _ = crate::terminal::TerminalGuard::resume();

    Ok(())
}