// src/clipboard.rs
use arboard::Clipboard;
use base64::{engine::general_purpose, Engine as _};
use std::io::{self, Write};

pub struct SystemClipboard {
    ctx: Option<Clipboard>,
    fallback_buffer: String,
}

impl SystemClipboard {
    pub fn new() -> Self {
        let ctx = Clipboard::new().ok();
        Self {
            ctx,
            fallback_buffer: String::new(),
        }
    }

    /// Sets text to local system clipboard, blasts OSC 52 to stdout, and updates internal fallback
    pub fn set_text(&mut self, text: &str) -> io::Result<()> {
        self.fallback_buffer = text.to_string();

        // 1. Try local system clipboard via arboard
        if let Some(ref mut clipboard) = self.ctx {
            let _ = clipboard.set_text(text);
        }

        // 2. Blast OSC 52 escape sequence for remote/SSH/terminal support
        let b64 = general_purpose::STANDARD.encode(text.as_bytes());
        let sequence = format!("\x1b]52;c;{}\x07", b64);
        let mut stdout = io::stdout();
        stdout.write_all(sequence.as_bytes())?;
        stdout.flush()?;

        Ok(())
    }

    /// Retrieves text from system clipboard, falling back to internal memory buffer
    pub fn get_text(&mut self) -> String {
        if let Some(ref mut clipboard) = self.ctx {
            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() {
                    return text;
                }
            }
        }
        self.fallback_buffer.clone()
    }
}