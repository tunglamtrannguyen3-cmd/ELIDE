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
    let _tracer = tracer::KernelTracer::init();
    let mut stdout = stdout();

    let mut editor = Editor::new();
    let mut palette = Palette::new();
    let mut lsp_client = LspClient::new();

    let (diag_tx, mut diag_rx) = mpsc::unbounded_channel::<Vec<diagnostics::Diagnostic>>();

    // UI State
    let mut current_diagnostics: Vec<diagnostics::Diagnostic> = Vec::new();
    let mut command_history: Vec<(String, bool)> = Vec::new(); 
    
    // Default Idle State: 100% Codespace
    let mut show_lsp_pane = false; 
    let mut lsp_scroll_offset: usize = 0;

    if let Some(ref filename) = editor.filename {
        let _ = lsp_client.start(filename, diag_tx.clone()).await;
    }

    loop {
        let (term_width, term_height) = crossterm::terminal::size()?;
        
        // --- LAYOUT MATH ---
        // 1. Vertical Split: Terminal (60%) vs Codespace
        let term_pane_height = if palette.is_active {
            ((term_height as usize) * 60) / 100
        } else {
            0
        };
        // The top area height (Codespace + LSP) leaves room for the Terminal + Status Bar (1 line) + Palette Input (1 line)
        let editor_height = (term_height as usize).saturating_sub(term_pane_height).saturating_sub(2);
        
        // 2. Horizontal Split: LSP (10%) vs Codespace
        let debug_width = if show_lsp_pane {
            ((term_width as usize) * 10) / 100
        } else {
            0
        };
        let codespace_width = (term_width as usize).saturating_sub(debug_width);

        while let Ok(diags) = diag_rx.try_recv() {
            current_diagnostics = diags;
        }

        let event = terminal::poll_event(Duration::from_millis(50))?;

        match event {
            InputEvent::Key(key) => {
                // EXCLUSIVE CLOSING LOGIC
                if terminal::is_esc(&key) {
                    if palette.is_active {
                        palette.is_active = false; // ONLY Esc closes terminal
                    } else {
                        break; // Exit app if terminal is already closed
                    }
                } else if terminal::is_alt_t(&key) {
                    palette.is_active = true; // Open terminal (doesn't toggle off)
                } else if terminal::is_alt_l(&key) {
                    show_lsp_pane = !show_lsp_pane; // Alt+L toggles LSP
                } else if terminal::is_alt_up(&key) {
                    lsp_scroll_offset = lsp_scroll_offset.saturating_sub(1);
                } else if terminal::is_alt_down(&key) {
                    lsp_scroll_offset = lsp_scroll_offset.saturating_add(1);
                } else if palette.is_active {
                    // Terminal Input Handling
                    match key.code {
                        crossterm::event::KeyCode::Enter => {
                            let action = palette.parse_command(editor.filename.as_deref());
                            let (msg, success) = palette.execute_action(action, &mut editor).await;
                            
                            if success && lsp_client.server_name == "Disconnected" {
                                if let Some(ref filename) = editor.filename {
                                    let _ = lsp_client.start(filename, diag_tx.clone()).await;
                                }
                            }
                            
                            command_history.push((msg, success));
                            palette.input_buffer.clear(); // Clear input, but keep terminal open!
                        }
                        crossterm::event::KeyCode::Char(c) => palette.input_buffer.push(c),
                        crossterm::event::KeyCode::Backspace => { palette.input_buffer.pop(); },
                        _ => {}
                    }
                } else {
                    // Codespace Input Handling
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

        let edit_pane_w = codespace_width.saturating_sub(6);
        editor.scroll_into_view(edit_pane_w, editor_height);

        // --- RENDER PASS ---
        stdout.execute(cursor::Hide)?;
        stdout.execute(Clear(ClearType::All))?;

        // 1. Draw Codespace Box (Top Left, 100% or 90% width)
        for i in 0..editor_height {
            let row = editor.row_offset + i;
            stdout.execute(cursor::MoveTo(0, i as u16))?;
            
            if row < editor.lines.len() {
                let line = &editor.lines[row];
                let display_line: String = line
                    .chars()
                    .skip(editor.col_offset)
                    .take(edit_pane_w)
                    .collect();
                let formatted = format!("{:3} | {:<w$}", row + 1, display_line, w = edit_pane_w);
                write!(stdout, "{}", &formatted[..std::cmp::min(formatted.len(), codespace_width)])?;
            } else {
                stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                write!(stdout, "{}", "~".repeat(codespace_width.min(4)))?;
                stdout.execute(ResetColor)?;
            }
        }

        // 2. Draw Diagnosis LSP Box (Top Right, 10% width)
        if show_lsp_pane && debug_width > 0 {
            let debug_x = codespace_width as u16;
            for i in 0..editor_height {
                stdout.execute(cursor::MoveTo(debug_x, i as u16))?;
                stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                write!(stdout, "│")?;
                stdout.execute(ResetColor)?;
            }

            stdout.execute(cursor::MoveTo(debug_x + 2, 0))?;
            stdout.execute(SetForegroundColor(Color::Cyan))?;
            write!(stdout, "LSP")?;
            stdout.execute(ResetColor)?;

            if current_diagnostics.is_empty() {
                stdout.execute(cursor::MoveTo(debug_x + 2, 2))?;
                write!(stdout, "Nominal")?;
            } else {
                let mut wrapped_lines = Vec::new();
                let text_width = debug_width.saturating_sub(3);
                
                if text_width > 0 {
                    for diag in &current_diagnostics {
                        let full_msg = format!("L{}: {}", diag.line, diag.message);
                        let chars: Vec<char> = full_msg.chars().collect();
                        for chunk in chars.chunks(text_width) {
                            wrapped_lines.push(chunk.iter().collect::<String>());
                        }
                        wrapped_lines.push(String::new());
                    }
                }

                let max_display_lines = editor_height.saturating_sub(3);
                let max_scroll = wrapped_lines.len().saturating_sub(max_display_lines);
                lsp_scroll_offset = lsp_scroll_offset.min(max_scroll);

                for (idx, line) in wrapped_lines.iter().skip(lsp_scroll_offset).take(max_display_lines).enumerate() {
                    stdout.execute(cursor::MoveTo(debug_x + 2, 2 + idx as u16))?;
                    write!(stdout, "{}", line)?;
                }
            }
        }

        // 3. Draw Status Bar (The horizontal boundary)
        let status_row = editor_height as u16;
        stdout.execute(cursor::MoveTo(0, status_row))?;
        stdout.execute(SetForegroundColor(Color::Black))?;
        stdout.execute(SetBackgroundColor(Color::White))?;
        let status = format!(
            " Codespace: {} | Row: {} Col: {} | LSP: {:?} ",
            editor.filename.as_deref().unwrap_or("[Untitled]"),
            editor.cursor.row + 1,
            editor.cursor.col + 1,
            lsp_client.server_name.as_str()
        );
        write!(stdout, "{:width$}", status, width = term_width as usize)?;
        stdout.execute(ResetColor)?;

        // 4. Draw Persistent Terminal Pane (Bottom, 60% height placeholder)
        if palette.is_active && term_pane_height > 0 {
            let term_start_row = status_row + 1;
            
            // Render Command History
            let mut display_history = Vec::new();
            for (msg, success) in &command_history {
                for line in msg.lines() {
                    display_history.push((line.to_string(), *success));
                }
            }
            
            let term_output_height = term_pane_height.saturating_sub(1);
            let start_idx = display_history.len().saturating_sub(term_output_height);
            
            for (idx, (line, success)) in display_history.iter().skip(start_idx).take(term_output_height).enumerate() {
                stdout.execute(cursor::MoveTo(0, term_start_row + idx as u16))?;
                
                let status_type = if *success { 
                    colors::Status::Success 
                } else if line.starts_with("Unknown") {
                    colors::Status::UnknownCommand
                } else if line.starts_with("Code") {
                    colors::Status::UnknownCode
                } else if line.contains("warn") || line.contains("Warning") {
                    colors::Status::Warning
                } else if line.contains("hint") || line.contains("Hint") {
                    colors::Status::Hint
                } else { 
                    colors::Status::Error 
                };

                stdout.execute(SetAttribute(colors::attribute_for_status(status_type)))?;
                stdout.execute(SetForegroundColor(colors::color_for_status(status_type)))?;
                write!(stdout, "{}", &line[..std::cmp::min(line.len(), term_width as usize)])?;
                
                stdout.execute(SetAttribute(Attribute::Reset))?;
                stdout.execute(ResetColor)?;
            }
        }

        // 5. Draw Command Palette Input Line (Absolute Bottom)
        stdout.execute(cursor::MoveTo(0, term_height - 1))?;
        if palette.is_active {
            stdout.execute(SetForegroundColor(Color::Yellow))?;
            write!(stdout, ": {}", palette.input_buffer)?;
            stdout.execute(ResetColor)?;
        } else {
            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
            write!(stdout, ": (press Alt+T for terminal, Esc to close)")?;
            stdout.execute(ResetColor)?;
        }

        // 6. Sync Hardware Cursor
        if palette.is_active {
            stdout.execute(cursor::MoveTo((2 + palette.input_buffer.len()) as u16, term_height - 1))?;
        } else {
            let screen_row = (editor.cursor.row.saturating_sub(editor.row_offset)) as u16;
            let visual_col = editor.visual_cursor_col().saturating_sub(editor.col_offset);
            let screen_col = (visual_col + 6).min(codespace_width.saturating_sub(1)) as u16;
            
            if (screen_row as usize) < editor_height {
                stdout.execute(cursor::MoveTo(screen_col, screen_row))?;
            }
        }

        stdout.execute(cursor::Show)?;
        stdout.flush()?;
    }

    Ok(())
}