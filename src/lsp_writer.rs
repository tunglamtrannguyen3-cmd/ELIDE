use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
use serde_json::Value;
use anyhow::Result;

#[derive(Clone)]
pub struct LspWriter {
    tx: mpsc::UnboundedSender<Value>,
}

impl LspWriter {
    /// Spawns a dedicated background task owning `stdin` so the pipe is never dropped.
    pub fn spawn(mut stdin: tokio::process::ChildStdin) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Value>();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let msg_str = msg.to_string();
                let payload = format!("Content-Length: {}\r\n\r\n{}", msg_str.len(), msg_str);

                if stdin.write_all(payload.as_bytes()).await.is_err() || stdin.flush().await.is_err() {
                    break; // Process terminated or pipe closed
                }
            }
        });

        Self { tx }
    }

    /// Non-blocking method to queue JSON-RPC payloads to the LSP server.
    pub fn send(&self, msg: Value) -> Result<()> {
        self.tx.send(msg)?;
        Ok(())
    }
}