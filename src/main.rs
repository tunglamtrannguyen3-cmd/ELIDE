// src/main.rs
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
    event::KeyCode,
    style::{Attribute, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use editor::Editor;
use lsp::LspClient;
use palette::{Palette, PaletteAction};
use std::{
    collections::HashMap,
    io::{stdout, Stdout, Write},
    path::Path,
    time::Duration,
};
use tokio::sync::mpsc;
use terminal::InputEvent;

fn flush_token(stdout: &mut Stdout, word: &mut String, next_char: Option<char>) -> std::io::Result<()> {
    if word.is_empty() { return Ok(()); }
    
    let color = match word.as_str() {
        "fn" | "let" | "mut" | "return" | "if" | "else" | "match" | "struct" | 
        "enum" | "pub" | "use" | "impl" | "for" | "while" | "loop" | "const" | 
        "static" | "async" | "await" | "trait" | "type" | "as" | "ref" | "mod" |
        "where" | "dyn" | "move" | "unsafe" | "extern" | "break" | "continue"
            => colors::Palette::KEYWORD_MAUVE,
        "String" | "usize" | "isize" | "i8" | "i16" | "i32" | "i64" | "i128" |
        "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "char" |
        "Vec" | "Option" | "Result" | "Self" | "Box" | "Rc" | "Arc" | "HashMap" | "BTreeMap"
            => colors::Palette::TYPE_YELLOW,
        "true" | "false" | "Some" | "None" | "Ok" | "Err"
            => colors::Palette::NUMBER_PEACH,
        _ => {
            if word.starts_with('\'') {
                colors::Palette::STRING_GREEN
            } else if word.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_') || word.starts_with("0x") {
                colors::Palette::NUMBER_PEACH
            } else if next_char == Some('(') || next_char == Some('!') {
                colors::Palette::FUNCTION_BLUE
            } else {
                colors::Palette::TEXT_DEFAULT
            }
        }
    };

    stdout.execute(SetForegroundColor(color))?;
    write!(stdout, "{}", word)?;
    word.clear();
    Ok(())
}

struct AppState {
    editor: Editor,
    palette: Palette,
    lsp_client: LspClient,
    current_diagnostics: HashMap<String, Vec<diagnostics::Diagnostic>>,
    command_history: Vec<(String, bool)>,
    clipboard: String,
    doc_version: u32,
    
    show_lsp_pane: bool,
    lsp_scroll_offset: usize,
    term_scroll_y: usize,
    term_scroll_x: usize,
}

impl AppState {
    fn new() -> Self {
        Self {
            editor: Editor::new(),
            palette: Palette::new(),
            lsp_client: LspClient::new(),
            current_diagnostics: HashMap::new(),
            command_history: Vec::new(),
            clipboard: String::new(),
            doc_version: 1,
            show_lsp_pane: false,
            lsp_scroll_offset: 0,
            term_scroll_y: 0,
            term_scroll_x: 0,
        }
    }

    async fn handle_input(
        &mut self, 
        key: crossterm::event::KeyEvent, 
        diag_tx: &mpsc::UnboundedSender<(String, Vec<diagnostics::Diagnostic>)>
    ) -> bool {
        if terminal::is_esc(&key) {
            if self.palette.is_active {
                self.palette.is_active = false;
            } else {
                return false;
            }
        } else if terminal::is_alt_t(&key) {
            self.palette.is_active = !self.palette.is_active;
        } else if terminal::is_alt_l(&key) {
            self.show_lsp_pane = !self.show_lsp_pane;
        } else if terminal::is_alt_up(&key) {
            self.lsp_scroll_offset = self.lsp_scroll_offset.saturating_sub(1);
        } else if terminal::is_alt_down(&key) {
            self.lsp_scroll_offset = self.lsp_scroll_offset.saturating_add(1);
        } else if self.palette.is_active {
            self.handle_terminal_input(key, diag_tx).await;
        } else {
            self.handle_editor_input(key);
        }
        true
    }

    async fn handle_terminal_input(
        &mut self, 
        key: crossterm::event::KeyEvent,
        diag_tx: &mpsc::UnboundedSender<(String, Vec<diagnostics::Diagnostic>)>
    ) {
        match key.code {
            KeyCode::Up => self.term_scroll_y = self.term_scroll_y.saturating_add(1),
            KeyCode::Down => self.term_scroll_y = self.term_scroll_y.saturating_sub(1),
            KeyCode::Left => self.term_scroll_x = self.term_scroll_x.saturating_sub(1),
            KeyCode::Right => self.term_scroll_x = self.term_scroll_x.saturating_add(1),
            KeyCode::Enter => {
                self.term_scroll_y = 0;
                self.term_scroll_x = 0;

                let action = self.palette.parse_command(self.editor.filename.as_deref());
                let lsp_start_cmd = match &action {
                    PaletteAction::SetLsp(cmd) if !cmd.is_empty() => Some(cmd.clone()),
                    _ => None,
                };

                let (msg, success) = self.palette.execute_action(action, &mut self.editor, &mut self.clipboard).await;
                
                if let Some(cmd) = lsp_start_cmd {
                    let curr_dir = std::env::current_dir().unwrap_or_default().to_string_lossy().to_string();
                    
                    let target_lang = if let Some(ref fname) = self.editor.filename {
                        lsp::get_language_id(fname)
                    } else {
                        lsp::detect_workspace_language(&curr_dir).unwrap_or("plaintext")
                    };

                    if let Err(e) = self.lsp_client.init_workspace(&cmd, target_lang, &curr_dir, diag_tx.clone()).await {
                        self.command_history.push((format!("LSP Boot Error: {}", e), false));
                    } else {
                        self.show_lsp_pane = true;
                        if let Some(ref filename) = self.editor.filename {
                            let text = self.editor.lines.join("\n");
                            let _ = self.lsp_client.open_file(filename, &text);
                        }
                    }
                }
                
                self.command_history.push((msg, success));
                self.palette.input_buffer.clear();
            }
            KeyCode::Char(c) => self.palette.input_buffer.push(c),
            KeyCode::Backspace => { self.palette.input_buffer.pop(); },
            _ => {}
        }
    }

    fn handle_editor_input(&mut self, key: crossterm::event::KeyEvent) {
        let mut text_changed = false;
        
        match key.code {
            KeyCode::Char(c) => {
                if self.editor.filename.is_some() {
                    self.editor.insert_char(c);
                    text_changed = true;
                }
            },
            KeyCode::Enter => {
                if self.editor.filename.is_none() {
                    self.open_selected_tree_item();
                } else {
                    self.editor.insert_newline();
                    text_changed = true;
                }
            },
            KeyCode::Backspace => {
                if self.editor.filename.is_some() {
                    self.editor.backspace();
                    text_changed = true;
                }
            },
            KeyCode::Delete => {
                if self.editor.filename.is_some() {
                    self.editor.delete_char();
                    text_changed = true;
                }
            },
            KeyCode::Left => self.editor.move_cursor(0, -1),
            KeyCode::Right => self.editor.move_cursor(0, 1),
            KeyCode::Up => self.editor.move_cursor(-1, 0),
            KeyCode::Down => self.editor.move_cursor(1, 0),
            _ => {}
        }

        if text_changed {
            if let Some(ref filename) = self.editor.filename {
                self.doc_version += 1;
                let full_text = self.editor.lines.join("\n");
                let _ = self.lsp_client.notify_change(filename, self.doc_version, &full_text);
            }
        }
    }

    fn open_selected_tree_item(&mut self) {
        let row = self.editor.cursor.row;
        if row >= self.editor.lines.len() {
            return;
        }
        let raw_line = &self.editor.lines[row];
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with("Workspace:") || trimmed.starts_with('─') {
            return;
        }

        let target_str = trimmed.trim_end_matches('/');
        let path = Path::new(target_str);

        if path.is_dir() {
            if let Ok(canonical_dir) = std::fs::canonicalize(path) {
                let _ = std::env::set_current_dir(&canonical_dir);
                if let Ok(entries) = std::fs::read_dir(&canonical_dir) {
                    let mut lines = vec![
                        format!(" Workspace: {}", canonical_dir.display()),
                        " ──────────────────────────────────────────".to_string(),
                        " ../".to_string(), // Ensure parent directory navigation is available
                    ];

                    let mut sorted_entries: Vec<String> = entries
                        .flatten()
                        .filter_map(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            if name.starts_with('.') || name == "target" || name == "node_modules" {
                                return None;
                            }
                            if e.path().is_dir() {
                                Some(format!(" {}/", name))
                            } else {
                                Some(format!(" {}", name))
                            }
                        })
                        .collect();

                    sorted_entries.sort();
                    lines.extend(sorted_entries);

                    self.editor.lines = lines;
                    self.editor.filename = None;
                    self.editor.cursor.row = 0;
                    self.editor.cursor.col = 0;
                }
            }
        } else if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let filename_str = path.to_string_lossy().to_string();
                self.editor.lines = content.lines().map(String::from).collect();
                if self.editor.lines.is_empty() {
                    self.editor.lines.push(String::new());
                }
                self.editor.filename = Some(filename_str.clone());
                self.editor.cursor.row = 0;
                self.editor.cursor.col = 0;

                let full_text = self.editor.lines.join("\n");
                let _ = self.lsp_client.open_file(&filename_str, &full_text);
            }
        }
    }
}

struct Layout {
    term_width: u16,
    term_height: u16,
    banner_height: usize,
    term_pane_height: usize,
    editor_height: usize,
    debug_width: usize,
    codespace_width: usize,
    edit_pane_w: usize,
}

impl Layout {
    fn compute(state: &AppState) -> Result<Self> {
        let (term_width, term_height) = crossterm::terminal::size()?;
        
        let banner_height = 1;
        let status_height = 1;
        let palette_height = 1;
        
        let term_pane_height = if state.palette.is_active {
            ((term_height as usize) * 30) / 100
        } else {
            0
        };
        
        let editor_height = (term_height as usize)
            .saturating_sub(term_pane_height)
            .saturating_sub(banner_height + status_height + palette_height);
            
        let debug_width = if state.show_lsp_pane {
            ((term_width as usize) * 25) / 100
        } else {
            0
        };
        
        let codespace_width = (term_width as usize).saturating_sub(debug_width);
        let edit_pane_w = codespace_width.saturating_sub(9);

        Ok(Self {
            term_width, term_height, banner_height, term_pane_height,
            editor_height, debug_width, codespace_width, edit_pane_w
        })
    }
}

fn set_border_color(stdout: &mut Stdout) -> Result<()> {
    stdout.execute(SetForegroundColor(colors::Palette::UI_BORDER))?;
    Ok(())
}

fn draw_frame(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    set_border_color(stdout)?;
    
    stdout.execute(cursor::MoveTo(0, 0))?;
    write!(stdout, "╭{}╮", "─".repeat(layout.term_width.saturating_sub(2) as usize))?;

    let divider_x = layout.codespace_width.saturating_sub(1) as u16;
    for i in 0..layout.editor_height {
        let y = (layout.banner_height + i) as u16;
        stdout.execute(cursor::MoveTo(0, y))?;
        write!(stdout, "│")?;
        
        if state.show_lsp_pane && layout.debug_width > 0 {
            stdout.execute(cursor::MoveTo(divider_x, y))?;
            write!(stdout, "│")?;
        }
        
        stdout.execute(cursor::MoveTo(layout.term_width - 1, y))?;
        write!(stdout, "│")?;
    }

    let status_y = (layout.banner_height + layout.editor_height) as u16;
    stdout.execute(cursor::MoveTo(0, status_y))?;
    write!(stdout, "├{}┤", "─".repeat(layout.term_width.saturating_sub(2) as usize))?;
    
    for i in 0..layout.term_pane_height {
        let y = status_y + 1 + i as u16;
        stdout.execute(cursor::MoveTo(0, y))?;
        write!(stdout, "│")?;
        stdout.execute(cursor::MoveTo(layout.term_width - 1, y))?;
        write!(stdout, "│")?;
    }

    let bottom_y = layout.term_height.saturating_sub(1);
    stdout.execute(cursor::MoveTo(0, bottom_y))?;
    write!(stdout, "╰{}╯", "─".repeat(layout.term_width.saturating_sub(2) as usize))?;

    if state.show_lsp_pane && layout.debug_width > 0 {
        stdout.execute(cursor::MoveTo(divider_x, 0))?;
        write!(stdout, "┬")?;
        stdout.execute(cursor::MoveTo(divider_x, status_y))?;
        write!(stdout, "┴")?;
    }

    stdout.execute(ResetColor)?;
    Ok(())
}

fn render_ui(stdout: &mut Stdout, state: &mut AppState, layout: &Layout) -> Result<()> {
    stdout.execute(cursor::Hide)?;
    stdout.execute(Clear(ClearType::All))?;

    draw_frame(stdout, state, layout)?;
    draw_banner(stdout, layout)?;
    draw_editor(stdout, state, layout)?;
    draw_lsp_pane(stdout, state, layout)?;
    draw_status_bar(stdout, state, layout)?;
    
    if state.palette.is_active && layout.term_pane_height > 0 {
        draw_terminal_pane(stdout, state, layout)?;
    }
    
    draw_command_palette(stdout, state, layout)?;
    sync_cursor(stdout, state, layout)?;

    stdout.execute(cursor::Show)?;
    stdout.flush()?;
    Ok(())
}

fn draw_banner(stdout: &mut Stdout, _layout: &Layout) -> Result<()> {
    stdout.execute(cursor::MoveTo(2, 0))?;
    stdout.execute(SetForegroundColor(colors::Palette::FUNCTION_BLUE))?; 
    write!(stdout, " Your shortcut to an Easier Life")?;
    stdout.execute(ResetColor)?;
    Ok(())
}

fn draw_editor(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    for i in 0..layout.editor_height {
        let row = state.editor.row_offset + i;
        let screen_y = (layout.banner_height + i) as u16;
        
        stdout.execute(cursor::MoveTo(2, screen_y))?;
        
        if row < state.editor.lines.len() {
            let line = &state.editor.lines[row];
            let display_line: String = line
                .chars()
                .skip(state.editor.col_offset)
                .take(layout.edit_pane_w)
                .collect();
            
            stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
            write!(stdout, "{:3} ", row + 1)?;
            set_border_color(stdout)?;
            write!(stdout, "│ ")?;

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
                } else if c == '/' && next_c == Some('/') {
                    let _ = flush_token(stdout, &mut word, Some('/'));
                    in_comment = true;
                    stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
                    write!(stdout, "{}", c)?;
                } else if c == '"' {
                    let _ = flush_token(stdout, &mut word, Some('"'));
                    in_string = true;
                    stdout.execute(SetForegroundColor(colors::Palette::STRING_GREEN))?;
                    write!(stdout, "{}", c)?;
                } else if c.is_alphanumeric() || c == '_' {
                    word.push(c);
                } else {
                    let _ = flush_token(stdout, &mut word, Some(c));
                    stdout.execute(SetForegroundColor(colors::Palette::TEXT_DEFAULT))?;
                    write!(stdout, "{}", c)?;
                }
            }
            let _ = flush_token(stdout, &mut word, None);
            stdout.execute(ResetColor)?;
        } else {
            stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
            write!(stdout, "    ")?;
            set_border_color(stdout)?;
            write!(stdout, "│ ")?;
            stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
            write!(stdout, "~")?;
            stdout.execute(ResetColor)?;
        }
    }
    Ok(())
}

fn draw_lsp_pane(stdout: &mut Stdout, state: &mut AppState, layout: &Layout) -> Result<()> {
    if !state.show_lsp_pane || layout.debug_width == 0 { return Ok(()); }

    let divider_x = layout.codespace_width.saturating_sub(1) as u16;
    let debug_x = divider_x + 1;
    
    stdout.execute(cursor::MoveTo(debug_x + 1, 1))?;
    stdout.execute(SetForegroundColor(colors::Palette::HINT_TEAL))?;
    write!(stdout, "LSP Workspace")?;
    stdout.execute(ResetColor)?;

    if state.current_diagnostics.is_empty() {
        stdout.execute(cursor::MoveTo(debug_x + 1, 3))?;
        stdout.execute(SetForegroundColor(colors::Palette::SUCCESS_GREEN))?;
        write!(stdout, "Nominal")?;
        stdout.execute(ResetColor)?;
    } else {
        let mut wrapped_lines = Vec::new();
        let text_width = layout.debug_width.saturating_sub(4);
        
        if text_width > 0 {
            let mut sorted_uris: Vec<_> = state.current_diagnostics.keys().collect();
            sorted_uris.sort();

            for uri in sorted_uris {
                let diags = &state.current_diagnostics[uri];
                let filename = uri.rsplit('/').next().unwrap_or("?");

                for diag in diags {
                    let status = match diag.severity {
                        Some(1) => colors::Status::Error,
                        Some(2) => colors::Status::Warning,
                        Some(3) | Some(4) => colors::Status::Hint,
                        _ => {
                            let msg_lower = diag.message.to_lowercase();
                            if msg_lower.contains("error") {
                                colors::Status::Error
                            } else if msg_lower.contains("warn") {
                                colors::Status::Warning
                            } else if msg_lower.contains("hint") || msg_lower.contains("info") {
                                colors::Status::Hint
                            } else {
                                colors::Status::Error
                            }
                        }
                    };

                    let full_msg = format!("{}:L{}: {}", filename, diag.line, diag.message);
                    for chunk in full_msg.chars().collect::<Vec<_>>().chunks(text_width) {
                        wrapped_lines.push((chunk.iter().collect::<String>(), status));
                    }
                    wrapped_lines.push((String::new(), status));
                }
            }
        }

        let max_display_lines = layout.editor_height.saturating_sub(3);
        let max_scroll = wrapped_lines.len().saturating_sub(max_display_lines);
        state.lsp_scroll_offset = state.lsp_scroll_offset.min(max_scroll);

        for (idx, (line, status)) in wrapped_lines.iter().skip(state.lsp_scroll_offset).take(max_display_lines).enumerate() {
            stdout.execute(cursor::MoveTo(debug_x + 1, 3 + idx as u16))?;
            if !line.is_empty() {
                stdout.execute(SetAttribute(colors::attribute_for_status(*status)))?;
                stdout.execute(SetForegroundColor(colors::color_for_status(*status)))?;
                write!(stdout, "{}", line)?;
                stdout.execute(SetAttribute(Attribute::Reset))?;
                stdout.execute(ResetColor)?;
            }
        }
    }
    Ok(())
}

fn draw_status_bar(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    let status_row = (layout.banner_height + layout.editor_height) as u16;
    stdout.execute(cursor::MoveTo(2, status_row))?;
    
    let status = format!(
        " {} | R:{} C:{} | {} ({}) ",
        state.editor.filename.as_deref().unwrap_or("[Workspace]"),
        state.editor.cursor.row + 1,
        state.editor.cursor.col + 1,
        state.lsp_client.server_name.as_str(),
        state.lsp_client.language_id.as_str(),
    );
    
    stdout.execute(SetForegroundColor(colors::Palette::TEXT_DEFAULT))?;
    write!(stdout, "{}", status)?;
    stdout.execute(ResetColor)?;
    Ok(())
}

fn draw_terminal_pane(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    let status_row = (layout.banner_height + layout.editor_height) as u16;
    let term_start_row = status_row + 1;
    
    let mut display_history = Vec::new();
    for (msg, success) in &state.command_history {
        for line in msg.lines() {
            display_history.push((line.to_string(), *success));
        }
    }
    
    let term_output_height = layout.term_pane_height;
    let max_scroll_y = display_history.len().saturating_sub(term_output_height);
    let current_scroll_y = state.term_scroll_y.min(max_scroll_y);
    
    let start_idx = display_history.len()
        .saturating_sub(term_output_height)
        .saturating_sub(current_scroll_y);
    
    let max_width = layout.term_width.saturating_sub(4) as usize;

    for (idx, (line, success)) in display_history.iter().skip(start_idx).take(term_output_height).enumerate() {
        stdout.execute(cursor::MoveTo(2, term_start_row + idx as u16))?;
        
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

        let chars: Vec<char> = line.chars().collect();
        let display_str: String = if state.term_scroll_x < chars.len() {
            chars.into_iter()
                .skip(state.term_scroll_x)
                .take(max_width)
                .collect()
        } else {
            String::new()
        };

        stdout.execute(SetAttribute(colors::attribute_for_status(status_type)))?;
        stdout.execute(SetForegroundColor(colors::color_for_status(status_type)))?;
        write!(stdout, "{}", display_str)?;
        
        stdout.execute(SetAttribute(Attribute::Reset))?;
        stdout.execute(ResetColor)?;
    }
    Ok(())
}

fn draw_command_palette(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    let bottom_y = layout.term_height.saturating_sub(1);
    stdout.execute(cursor::MoveTo(2, bottom_y))?;
    if state.palette.is_active {
        stdout.execute(SetForegroundColor(colors::Palette::TYPE_YELLOW))?;
        write!(stdout, " : {} ", state.palette.input_buffer)?;
        stdout.execute(ResetColor)?;
    } else {
        stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
        write!(stdout, " : (Alt+T term, Esc close) ")?;
        stdout.execute(ResetColor)?;
    }
    Ok(())
}

fn sync_cursor(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    if state.palette.is_active {
        let prompt_len = 3; 
        stdout.execute(cursor::MoveTo((2 + prompt_len + state.palette.input_buffer.len()) as u16, layout.term_height.saturating_sub(1)))?;
    } else {
        let screen_row = (state.editor.cursor.row.saturating_sub(state.editor.row_offset) + layout.banner_height) as u16;
        let visual_col = state.editor.visual_cursor_col().saturating_sub(state.editor.col_offset);
        let screen_col = (visual_col + 8).min(layout.codespace_width.saturating_sub(2)) as u16;
        
        if (screen_row as usize) < (layout.banner_height + layout.editor_height) {
            stdout.execute(cursor::MoveTo(screen_col, screen_row))?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = terminal::TerminalGuard::init()?;
    let tracer = tracer::KernelTracer::init();
    tracer.log_event("ELIDE Editor Initialized!");

    let mut stdout = stdout();
    let mut state = AppState::new();
    let (diag_tx, mut diag_rx) = mpsc::unbounded_channel::<(String, Vec<diagnostics::Diagnostic>)>();

    if let Some(arg) = std::env::args().nth(1) {
        let path = Path::new(&arg);
        
        if path.is_dir() {
            let canonical_dir = std::fs::canonicalize(path)
                .unwrap_or_else(|_| path.to_path_buf());
            let _ = std::env::set_current_dir(&canonical_dir);

            if let Ok(entries) = std::fs::read_dir(&canonical_dir) {
                let mut lines = vec![
                    format!(" Workspace: {}", canonical_dir.display()),
                    " ──────────────────────────────────────────".to_string(),
                    " ../".to_string(), // Ensure parent directory navigation is available
                ];

                let mut sorted_entries: Vec<String> = entries
                    .flatten()
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') || name == "target" || name == "node_modules" {
                            return None;
                        }
                        if e.path().is_dir() {
                            Some(format!(" {}/", name))
                        } else {
                            Some(format!(" {}", name))
                        }
                    })
                    .collect();

                sorted_entries.sort();
                lines.extend(sorted_entries);

                state.editor.lines = lines;
                state.editor.filename = None;
            }
        } else if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(path) {
                state.editor.lines = content.lines().map(String::from).collect();
                if state.editor.lines.is_empty() {
                    state.editor.lines.push(String::new());
                }
                state.editor.filename = Some(arg);
            }
        }
    }

    // [NOTE]: Automatic LSP startup block has been removed completely. 
    // You can now activate your LSP manually via the command palette using `lsp <cmd>` (e.g., `lsp rust-analyzer`).

    let mut needs_redraw = true;

    loop {
        while let Ok((uri, diags)) = diag_rx.try_recv() {
            if diags.is_empty() {
                state.current_diagnostics.remove(&uri);
            } else {
                state.current_diagnostics.insert(uri, diags);
            }
            needs_redraw = true;
        }

        if needs_redraw {
            let layout = Layout::compute(&state)?;
            state.editor.scroll_into_view(layout.edit_pane_w, layout.editor_height);
            render_ui(&mut stdout, &mut state, &layout)?;
            needs_redraw = false;
        }

        match terminal::poll_event(Duration::from_millis(16))? {
            InputEvent::Key(key) => {
                needs_redraw = true;
                if !state.handle_input(key, &diag_tx).await {
                    break;
                }
            }
            InputEvent::ScrollUp => {
                needs_redraw = true;
                if state.palette.is_active {
                    state.term_scroll_y = state.term_scroll_y.saturating_add(3);
                } else if state.show_lsp_pane {
                    state.lsp_scroll_offset = state.lsp_scroll_offset.saturating_sub(3);
                } else {
                    state.editor.move_cursor(-3, 0);
                }
            }
            InputEvent::ScrollDown => {
                needs_redraw = true;
                if state.palette.is_active {
                    state.term_scroll_y = state.term_scroll_y.saturating_sub(3);
                } else if state.show_lsp_pane {
                    state.lsp_scroll_offset = state.lsp_scroll_offset.saturating_add(3);
                } else {
                 state.editor.move_cursor(3, 0);
                }
            }
            InputEvent::Resize => {
                needs_redraw = true;
            }
            InputEvent::Tick => {}
        }
    }
    
    Ok(())
}