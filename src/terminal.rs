use anyhow::Result;
use crossterm::{                                      event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::{io::stdout, time::Duration};
                                                  pub enum InputEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

pub struct TerminalGuard;
                                                  impl TerminalGuard {
    /// Initialize raw terminal mode and switch to alternate screen buffer                              pub fn init() -> Result<Self> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    /// Safety cleanup: Guarantees terminal restores cleanly even if app crashes/panics                 fn drop(&mut self) {
        let _ = disable_raw_mode();                       let _ = stdout().execute(LeaveAlternateScreen);                                                 }
}

/// Polls crossterm events asynchronously with a non-blocking timeout
pub fn poll_event(timeout: Duration) -> Result<InputEvent> {
    if event::poll(timeout)? {                            match event::read()? {                                Event::Key(key) => Ok(InputEvent::Key(key)),                                                        Event::Resize(w, h) => Ok(InputEvent::Resize(w, h)),                                                _ => Ok(InputEvent::Tick),
        }                                             } else {
        Ok(InputEvent::Tick)
    }
}

/// Helper checks for common hotkeys
pub fn is_alt_t(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('t')
}

pub fn is_esc(key: &KeyEvent) -> bool {
    key.code == KeyCode::Esc
}