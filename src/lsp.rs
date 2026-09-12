use anyhow::Result;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use crate::diagnostics::Diagnostic;

pub struct LspClient {
    pub server_name: String,
    _child: Option<Child>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            server_name: "Disconnected".to_string(),
            _child: None,
        }
    }

    pub async fn start(
        &mut self,
        filename: &str,
        _diag_tx: mpsc::UnboundedSender<Vec<Diagnostic>>,
    ) -> Result<()> {
        let ext = filename.split('.').last().unwrap_or("");
        let cmd = match ext {
            "rs" => "rust-analyzer",
            "cpp" | "c" | "h" => "clangd",
            "adb" | "ads" | "ada" => "ada_language_server",
            _ => return Ok(()),
        };

        self.server_name = cmd.to_string();

        let child = Command::new(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        match child {
            Ok(c) => {
                self._child = Some(c);
                Ok(())
            }
            Err(_) => {
                self.server_name = "Failed to launch".to_string();
                Ok(())
            }
        }
    }
}
