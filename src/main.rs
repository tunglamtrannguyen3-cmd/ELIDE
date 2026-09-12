mod editor;
mod palette;
mod terminal;
mod lsp;
mod diagnostics;
mod process;
mod colors;
mod tracer;

use anyhow::Result;
use crossterm::{
    cursor,
    style::{Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use editor::Editor;
use lsp::LspClient;
use palette::Palette;
use std::{
    io::{stdout, Write},
    time::Duration,
};
use tokio::sync::mpsc;
use terminal::InputEvent;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = terminal::TerminalGuard::init()?;
    let mut stdout = stdout();

    let mut editor = Editor::new();
    let mut palette = Palette::new();
    let mut lsp_client = LspClient::new();

    let (diag_tx, mut diag_rx) = mpsc::unbounded_channel::<Vec<diagnostics::Diagnostic>>();

    // UI State
    let mut show_diagnostics = false;
    let mut current_diagnostics: Vec<diagnostics::Diagnostic> = Vec::new();
    let mut command_msg: Option<(String, bool)> = None;

    if let Some(ref filename) = editor.filename {
        let _ = lsp_client.start(filename, diag_tx.clone()).await;
    }

    loop {
        let (term_width, term_height) = crossterm::terminal::size()?;
        let editor_width = term_width as usize;
        let editor_height = term_height.saturating_sub(2) as usize; // Reserve bottom 2 lines

        // Pull incoming async diagnostics
        while let Ok(diags) = diag_rx.try_recv() {
            current_diagnostics = diags;
        }

        let event = terminal::poll_event(Duration::from_millis(50))?;

        match event {
            InputEvent::Key(key) => {
                // Clear command messages on keystroke if palette isn't active
                if !palette.is_active && !terminal::is_alt_t(&key) && !terminal::is_alt_l(&key) {
                    command_msg = None;
                }

                if terminal::is_esc(&key) {
                    if palette.is_active {
                        palette.toggle();
                    } else {
                        break; // Exit editor
                    }
                } else if terminal::is_alt_t(&key) {
                    palette.toggle();
                } else if terminal::is_alt_l(&key) {
                    show_diagnostics = !show_diagnostics; // Toggle your custom overlay!
                } else if palette.is_active {
                    match key.code {
                        crossterm::event::KeyCode::Enter => {
                            let action = palette.parse_command(editor.filename.as_deref());
                            let (msg, success) = palette.execute_action(action, &mut editor).await;
                            command_msg = Some((msg, success));
                            palette.toggle(); // Auto-close palette on execute
                        }
                        crossterm::event::KeyCode::Char(c) => palette.input_buffer.push(c),
                        crossterm::event::KeyCode::Backspace => { palette.input_buffer.pop(); },
                        _ => {}
                    }
                } else {
                    match key.code {
                        crossterm::event::KeyCode::Char(c) => editor.insert_char(c),
                        crossterm::event::KeyCode::Enter => editor.insert_newline(),
                        crossterm::event::KeyCode::Backspace => editor.backspace(),
                        crossterm::event::KeyCode::Delete => editor.delete_char(),
                        crossterm::event::KeyCode::Left => editor.move_cursor(0, -1),
                        crossterm::event::KeyCode::Right => editor.move_cursor(0, 1),
                        crossterm::event::KeyCode::Up => editor.move_cursor(-1, 0),
                        crossterm::event::KeyCode::Down => editor.move_cursor(1, 0),
                        _ => {}
                    }
                }
            }
            InputEvent::Resize(w, h) => {
                let _ = (w, h);
            }
            InputEvent::Tick => {}
        }

        // Keep cursor in view before rendering
        editor.scroll_into_view(editor_width.saturating_sub(6), editor_height);

        // --- RENDER PASS ---
        stdout.execute(cursor::Hide)?;
        stdout.execute(Clear(ClearType::All))?;
        stdout.execute(cursor::MoveTo(0, 0))?;

        // 1. Draw Editor Buffer
        for i in 0..editor_height {
            let row = editor.row_offset + i;
            if row < editor.lines.len() {
                let line = &editor.lines[row];
                let display_line: String = line.chars().skip(editor.col_offset).take(editor_width.saturating_sub(6)).collect();
                writeln!(stdout, "{:3} | {}", row + 1, display_line)?;
            } else {
                stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                writeln!(stdout, "~")?;
                stdout.execute(ResetColor)?;
            }
        }

        // 2. Draw Alt+L Diagnostic Overlay
        if show_diagnostics {
            stdout.execute(cursor::MoveTo(term_width.saturating_sub(40), 1))?;
            stdout.execute(SetForegroundColor(Color::Red))?;
            stdout.execute(SetBackgroundColor(Color::DarkGrey))?;
            write!(stdout, " [LSP DIAGNOSTICS] ")?;                                                 
            if current_diagnostics.is_empty() {
                stdout.execute(cursor::MoveTo(term_width.saturating_sub(40), 2))?;
                write!(stdout, " No issues found. System nominal. ")?;
            } else {
                for (idx, diag) in current_diagnostics.iter().take(5).enumerate() {
                    stdout.execute(cursor::MoveTo(term_width.saturating_sub(40), 2 + idx as u16))?;
                    // Truncate message so it doesn't wrap wildly
                    let msg: String = diag.message.chars().take(28).collect();
                    write!(stdout, " L{}: {}... ", diag.line, msg)?;
                }
            }
            stdout.execute(ResetColor)?;
        }

        // 3. Draw Status Bar (Inverted Colors)
        stdout.execute(cursor::MoveTo(0, term_height - 2))?;
        stdout.execute(SetForegroundColor(Color::Black))?;
        stdout.execute(SetBackgroundColor(Color::White))?;
        let status = format!(
            " {} | Row: {} Col: {} | LSP: {:?} ",
            editor.filename.as_deref().unwrap_or("[Untitled]"),
            editor.cursor.row + 1,
            editor.cursor.col + 1,
            lsp_client.server_name.as_str()
        );
        write!(stdout, "{:width$}", status, width = term_width as usize)?;
        stdout.execute(ResetColor)?;

        // 4. Draw Command Palette OR Execution Messages
        stdout.execute(cursor::MoveTo(0, term_height - 1))?;
        if palette.is_active {
            stdout.execute(SetForegroundColor(Color::Yellow))?;
            write!(stdout, ": {}", palette.input_buffer)?;
            stdout.execute(ResetColor)?;
        } else if let Some((msg, success)) = &command_msg {
            let status = if *success { 
                colors::Status::Success 
            } else if msg.starts_with("Unknown command") {
                colors::Status::UnknownCommand
            } else if msg.starts_with("Unknown code") {
                colors::Status::UnknownCode
            } else if msg.contains("Warning") {
                colors::Status::Warning
            } else { 
                colors::Status::Error 
            };

            stdout.execute(SetAttribute(colors::attribute_for_status(status)))?;
            stdout.execute(SetForegroundColor(colors::color_for_status(status)))?;
            
            let flat_msg = msg.replace('\n', " | ");
            write!(stdout, "{}", flat_msg)?;
            
            stdout.execute(SetAttribute(Attribute::Reset))?;
            stdout.execute(ResetColor)?;
        }

        // 5. Sync Hardware Cursor
        if palette.is_active {
            stdout.execute(cursor::MoveTo((2 + palette.input_buffer.len()) as u16, term_height - 1))?;
        } else {
            let screen_row = (editor.cursor.row.saturating_sub(editor.row_offset)) as u16;
            let screen_col = (editor.visual_cursor_col().saturating_sub(editor.col_offset) + 6) as u16;
            stdout.execute(cursor::MoveTo(screen_col, screen_row))?;
        }

        stdout.execute(cursor::Show)?;
        stdout.flush()?;
    }

    Ok(())
}
