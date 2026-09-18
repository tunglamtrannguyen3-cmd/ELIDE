use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::{io::stdout, time::Duration};

pub enum InputEvent {
    Key(crossterm::event::KeyEvent),
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
        // Enable mouse events so we can detect scroll wheel input
        stdout().execute(EnableMouseCapture)?;
        Ok(Self)
    }

    /// Call this right BEFORE spawning a foreground process (e.g., vim)
    pub fn suspend() -> Result<()> {
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
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Safe to ignore errors during cleanup drop
        let _ = TerminalGuard::suspend();
    }
}

pub fn poll_event(timeout: Duration) -> Result<InputEvent> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(InputEvent::Key(key)),
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

pub fn is_alt_up(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Up
}

pub fn is_alt_down(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Down
}

pub fn is_esc(key: &KeyEvent) -> bool {
    key.code == KeyCode::Esc
}