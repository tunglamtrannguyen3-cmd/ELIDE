mod editor;
mod palette;
mod terminal;
mod lsp;
mod diagnostics;
mod process;
mod colors;
mod tracer;
mod lsp_writer;

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

// Helper to colorize and print words as they are assembled
fn flush_token(stdout: &mut std::io::Stdout, word: &mut String, next_char: Option<char>) -> std::io::Result<()> {
    if word.is_empty() { return Ok(()); }
    
    let color = match word.as_str() {
        // Core Rust Keywords
        "fn" | "let" | "mut" | "return" | "if" | "else" | "match" | "struct" | "enum" | "pub" | "use" | "impl" | "for" | "while" | "loop" | "const" | "static" | "async" | "await" | "trait" | "type" | "as" | "ref" 
            => colors::Palette::KEYWORD_PURPLE,
        // Common Types
        "String" | "usize" | "i32" | "u32" | "f64" | "bool" | "Vec" | "Option" | "Result" | "Self" 
            => colors::Palette::TYPE_YELLOW,
        // Booleans
        "true" | "false" 
            => colors::Palette::NUMBER_ORANGE,
        _ => {
            if word.chars().all(|c| c.is_ascii_digit()) {
                colors::Palette::NUMBER_ORANGE // Numbers
            } else if next_char == Some('(') || next_char == Some('!') {
                colors::Palette::FUNCTION_BLUE // Functions & Macros
            } else {
                colors::Palette::TEXT_DEFAULT // Standard text
            }
        }
    };

    stdout.execute(SetForegroundColor(color))?;
    write!(stdout, "{}", word)?;
    word.clear();
    Ok(())
}

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
    let mut doc_version: u32 = 1; // <-- NEW: Track document version for LSP
    
    // Default Idle State: 100% Codespace
    let mut show_lsp_pane = false; 
    let mut lsp_scroll_offset: usize = 0;
    loop {
        let (term_width, term_height) = crossterm::terminal::size()?;
        
        // --- LAYOUT MATH ---
        let banner = [
            "┌──────────────────────────────────────────────┐",
            "│  E L I D E  ::  Easier Life @ IDE  :: v1.2.0 │",
            "└──────────────────────────────────────────────┘",
        ];
        let banner_height = banner.len();

        // 1. Vertical Split: Terminal (60%) vs Codespace
        let term_pane_height = if palette.is_active {
            ((term_height as usize) * 60) / 100
        } else {
            0
        };
        // The top area height (Codespace + LSP) leaves room for Banner + Terminal + Status Bar (1 line) + Palette Input (1 line)
        let editor_height = (term_height as usize)
            .saturating_sub(term_pane_height)
            .saturating_sub(2)
            .saturating_sub(banner_height);
        
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
                            
                            // Check if this is an LSP command before execution consumes it
                            let lsp_start_cmd = match &action {
                                palette::PaletteAction::SetLsp(cmd) if !cmd.is_empty() => Some(cmd.clone()),
                                _ => None,
                            };

                            let (msg, success) = palette.execute_action(action, &mut editor).await;
                            
                            // If the command was to start the LSP, execute it on the LspClient
                           // If the command was to start the LSP, execute it on the LspClient
                            if let Some(cmd) = lsp_start_cmd {
                                if let Some(ref filename) = editor.filename {
                                    if let Err(e) = lsp_client.start(&cmd, filename, diag_tx.clone()).await {
                                        command_history.push((format!("LSP Boot Error: {}", e), false));
                                    } else {
                                        show_lsp_pane = true; // <-- NEW: Pop open the pane!
                                    }
                                } else {
                                    command_history.push(("Error: You must open a file before starting an LSP.".to_string(), false));
                                }
                            }
                            
                            command_history.push((msg, success));
                            palette.input_buffer.clear(); // Clear input, but keep terminal open!
                        }
                        crossterm::event::KeyCode::Char(c) => palette.input_buffer.push(c),
                        crossterm::event::KeyCode::Backspace => { palette.input_buffer.pop(); },
                        _ => {}
                        // Cleaned up the unused variable warning here!
            
                    }
                } else {
                    // Codespace Input Handling
                    let mut text_changed = false;
                    
                    match key.code {
                        // Mark text_changed = true only for keys that alter the buffer
                        crossterm::event::KeyCode::Char(c) => { editor.insert_char(c); text_changed = true; },
                        crossterm::event::KeyCode::Enter => { editor.insert_newline(); text_changed = true; },
                        crossterm::event::KeyCode::Backspace => { editor.backspace(); text_changed = true; },
                        crossterm::event::KeyCode::Delete => { editor.delete_char(); text_changed = true; },
                        
                        // Movement doesn't change the text, so we leave it alone
                        crossterm::event::KeyCode::Left => editor.move_cursor(0, -1),
                        crossterm::event::KeyCode::Right => editor.move_cursor(0, 1),
                        crossterm::event::KeyCode::Up => editor.move_cursor(-1, 0),
                        crossterm::event::KeyCode::Down => editor.move_cursor(1, 0),
                        _ => {}
                    }

                    // Sync changes to the LSP server if a mutation happened
                    if text_changed {
                        if let Some(ref filename) = editor.filename {
                            doc_version += 1;
                            let full_text = editor.lines.join("\n");
                            let _ = lsp_client.notify_change(filename, doc_version, &full_text);
                        }
                    }
                }
            }
            // Cleaned up the unused variable warning here!
            InputEvent::Resize(w, h) => {
                let _ = (w, h); // This tells Rust we read the variables
            }
            InputEvent::Tick => {}
        }
        
        let edit_pane_w = codespace_width.saturating_sub(6);
        editor.scroll_into_view(edit_pane_w, editor_height);

        // --- RENDER PASS ---
        stdout.execute(cursor::Hide)?;
        stdout.execute(Clear(ClearType::All))?;

        // 0. Draw Banner (At the very top)
        for (i, line) in banner.iter().enumerate() {
            stdout.execute(cursor::MoveTo(0, i as u16))?;
            stdout.execute(SetForegroundColor(Color::Cyan))?; 
            write!(stdout, "{}", line)?;
            stdout.execute(ResetColor)?;
        }

        // 1. Draw Codespace Box with Relaxing Syntax Highlighting
        for i in 0..editor_height {
            let row = editor.row_offset + i;
            let screen_y = (banner_height + i) as u16;
            stdout.execute(cursor::MoveTo(0, screen_y))?;
            
            if row < editor.lines.len() {
                let line = &editor.lines[row];
                let display_line: String = line
                    .chars()
                    .skip(editor.col_offset)
                    .take(edit_pane_w)
                    .collect();
                
                // Draw Line Number in Dark Grey
                stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                write!(stdout, "{:3} | ", row + 1)?;

                // --- Syntax Highlighting State Machine ---
                let mut in_string = false;
                let mut in_comment = false;
                let mut word = String::new();
                let chars: Vec<char> = display_line.chars().collect();

                for char_idx in 0..chars.len() {
                    let c = chars[char_idx];
                    let next_c = chars.get(char_idx + 1).copied();
                    
                    if in_comment {
                        stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
                        write!(stdout, "{}", c)?;
                    } else if in_string {
                        stdout.execute(SetForegroundColor(colors::Palette::STRING_GREEN))?;
                        write!(stdout, "{}", c)?;
                        if c == '"' { in_string = false; }
                    } else {
                        // Check for start of comments //
                        if c == '/' && next_c == Some('/') {
                            let _ = flush_token(&mut stdout, &mut word, Some('/'));
                            in_comment = true;
                            stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
                            write!(stdout, "{}", c)?;
                        // Check for start of strings "
                        } else if c == '"' {
                            let _ = flush_token(&mut stdout, &mut word, Some('"'));
                            in_string = true;
                            stdout.execute(SetForegroundColor(colors::Palette::STRING_GREEN))?;
                            write!(stdout, "{}", c)?;
                        // Build up words
                        } else if c.is_alphanumeric() || c == '_' {
                            word.push(c);
                        // Print symbols
                        } else {
                            let _ = flush_token(&mut stdout, &mut word, Some(c));
                            stdout.execute(SetForegroundColor(colors::Palette::TEXT_DEFAULT))?;
                            write!(stdout, "{}", c)?;
                        }
                    }
                }
                // Flush any remaining word at the end of the line
                let _ = flush_token(&mut stdout, &mut word, None);
                
                // Pad the remaining space so layout doesn't break
                let visual_len = chars.len();
                if visual_len < edit_pane_w {
                    stdout.execute(ResetColor)?;
                    write!(stdout, "{}", " ".repeat(edit_pane_w - visual_len))?;
                }
                
                stdout.execute(ResetColor)?;
            } else {
                stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                write!(stdout, "{}", "~".repeat(codespace_width.min(4)))?;
                stdout.execute(ResetColor)?;
            }
        }

        // 2. Draw Diagnosis LSP Box
        if show_lsp_pane && debug_width > 0 {
            let debug_x = codespace_width as u16;
            let total_top_height = banner_height + editor_height;
            
            // Draw the vertical divider wall all the way from the top
            for i in 0..total_top_height {
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

                let max_display_lines = total_top_height.saturating_sub(3);
                let max_scroll = wrapped_lines.len().saturating_sub(max_display_lines);
                lsp_scroll_offset = lsp_scroll_offset.min(max_scroll);

                for (idx, line) in wrapped_lines.iter().skip(lsp_scroll_offset).take(max_display_lines).enumerate() {
                    stdout.execute(cursor::MoveTo(debug_x + 2, 2 + idx as u16))?;
                    write!(stdout, "{}", line)?;
                }
            }
        }

        // 3. Draw Status Bar (Below the Codespace and Banner)
        let status_row = (banner_height + editor_height) as u16;
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

        // 4. Draw Persistent Terminal Pane
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

        // 5. Draw Command Palette Input Line
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
            let screen_row = (editor.cursor.row.saturating_sub(editor.row_offset) + banner_height) as u16;
            let visual_col = editor.visual_cursor_col().saturating_sub(editor.col_offset);
            let screen_col = (visual_col + 6).min(codespace_width.saturating_sub(1)) as u16;
            
            if (screen_row as usize) < (banner_height + editor_height) {
                stdout.execute(cursor::MoveTo(screen_col, screen_row))?;
            }
        }

        stdout.execute(cursor::Show)?;
        stdout.flush()?;
    }

    Ok(())
}