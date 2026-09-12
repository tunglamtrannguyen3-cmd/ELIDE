mod colors;
mod diagnostics;
mod editor;
mod lsp;
mod palette;
mod process;
mod terminal;
mod tracer;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use diagnostics::DiagnosticStore;
use editor::Editor;
use lsp::{LspClient, LspStatus};
use palette::{Palette, PaletteAction};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::{env, time::Duration};
use terminal::{InputEvent, TerminalGuard};
use tokio::sync::mpsc;

pub struct App {
    pub editor: Editor,
    pub palette: Palette,
    pub diagnostics: DiagnosticStore,
    pub lsp: LspClient,
    pub status_message: String,
    pub last_success: bool,
    pub is_running: bool,
    pub status_scroll: u16,
    pub diag_rx: mpsc::UnboundedReceiver<Vec<diagnostics::Diagnostic>>,
    pub diag_tx: mpsc::UnboundedSender<Vec<diagnostics::Diagnostic>>,
}

impl App {
    pub fn new() -> Self {
        let (diag_tx, diag_rx) = mpsc::unbounded_channel();
        Self {
            editor: Editor::new(),
            palette: Palette::new(),
            diagnostics: DiagnosticStore::new(),
            lsp: LspClient::new(),
            status_message: "Ready. Alt+T: Palette | Alt+L: Diagnostics".to_string(),
            last_success: true,
            is_running: true,
            status_scroll: 0,
            diag_rx,
            diag_tx,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _tracer = tracer::KernelTracer::init().ok();

    let _guard = TerminalGuard::init()?;
    let mut terminal = ratatui::init();
    let mut app = App::new();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "-bro!" {
            app.editor.lines = vec![
                "☯️ [ELIDE - Always ready] ☯️".to_string(),
                "System: All spiritual buffers loaded.".to_string(),
                "Ready for spellcard compilation...".to_string(),
            ];
            app.status_message = "Entered Bro Mode! Press Alt+T to open command palette.".to_string();
        } else {
            let filename = &args[1];
            if let Err(e) = app.editor.open(filename) {
                app.status_message = format!("Failed to open {}: {}", filename, e);
                app.last_success = false;
            } else {
                app.status_message = format!("Loaded file: {}", filename);
                // Start LSP in the background and send the file contents
                let _ = app.lsp.start(filename, app.diag_tx.clone()).await;
                let content = app.editor.lines.join("\n");
                let _ = app.lsp.notify_did_open(filename, &content).await;
            }
        }
    }

    // Main Event & Render Loop
    while app.is_running {
        // Non-blocking poll for incoming background diagnostics
        while let Ok(diags) = app.diag_rx.try_recv() {
            app.diagnostics.diagnostics = diags;
        }

        terminal.draw(|frame| render_ui(frame, &mut app))?;

        match terminal::poll_event(Duration::from_millis(16))? {
            InputEvent::Tick => {}
            InputEvent::Resize(_width, _height) => {}
            InputEvent::Key(key) => {
                if terminal::is_alt_t(&key) {
                    app.palette.toggle();
                    app.status_scroll = 0;
                    continue;
                }

                if terminal::is_alt_l(&key) {
                    app.diagnostics.toggle();
                    continue;
                }

                if app.palette.is_active {
                    if terminal::is_esc(&key) {
                        app.palette.toggle();
                        continue;
                    }

                    match key.code {
                        KeyCode::Up => app.status_scroll = app.status_scroll.saturating_sub(1),
                        KeyCode::Down => app.status_scroll = app.status_scroll.saturating_add(1),
                        KeyCode::PageUp => app.status_scroll = app.status_scroll.saturating_sub(5),
                        KeyCode::PageDown => app.status_scroll = app.status_scroll.saturating_add(5),
                        KeyCode::Enter => {
                            let action = app.palette.parse_command(app.editor.filename.as_deref());
                            app.palette.input_buffer.clear();
                            app.status_scroll = 0;

                            match action {
                                PaletteAction::Save => {
                                    match app.editor.save() {
                                        Ok(_) => {
                                            app.status_message = "File saved successfully.".to_string();
                                            app.last_success = true;
                                            
                                            // Sync current buffer to LSP on save
                                            if let Some(ref fname) = app.editor.filename {
                                                let content = app.editor.lines.join("\n");
                                                let _ = app.lsp.notify_did_open(fname, &content).await;
                                            }
                                        }
                                        Err(e) => {
                                            app.status_message = format!("Save failed: {}", e);
                                            app.last_success = false;
                                        }
                                    }
                                    app.palette.is_active = false;
                                }
                                PaletteAction::Compile => {
                                    app.palette.is_active = true;
                                    app.status_message = "Compiling / Running code...".to_string();
                                    match process::compile_file(&app.editor).await {
                                        Ok(res) => {
                                            if res.max_rss_kb > 0 {
                                                app.status_message = format!("{}\n[Peak RSS: {} KB]", res.output, res.max_rss_kb);
                                            } else {
                                                app.status_message = res.output;
                                            }
                                            app.last_success = res.success;
                                        }
                                        Err(e) => {
                                            app.status_message = format!("Execution error: {}", e);
                                            app.last_success = false;
                                        }
                                    }
                                }
                                PaletteAction::RunBash(cmd) => {
                                    app.palette.is_active = true;
                                    match process::run_bash_cmd(&cmd).await {
                                        Ok(res) => {
                                            app.status_message = res.output;
                                            app.last_success = res.success;
                                        }
                                        Err(e) => {
                                            app.status_message = format!("Bash execution error: {}", e);
                                            app.last_success = false;
                                        }
                                    }
                                }
                                _ => {
                                    app.palette.is_active = match action {
                                        PaletteAction::Help
                                        | PaletteAction::Info
                                        | PaletteAction::Bro
                                        | PaletteAction::Debug
                                        | PaletteAction::UnknownCommand(_)
                                        | PaletteAction::UnknownCode(_) => true,
                                        _ => false,
                                    };

                                    let (msg, success) =
                                        app.palette.execute_action(action, &mut app.editor).await;
                                    app.status_message = msg;
                                    app.last_success = success;
                                }
                            }
                        }
                        KeyCode::Backspace => { app.palette.input_buffer.pop(); }
                        KeyCode::Char(c) => { app.palette.input_buffer.push(c); }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => { app.is_running = false; }
                        KeyCode::Char(c) => app.editor.insert_char(c),
                        KeyCode::Enter => app.editor.insert_newline(),
                        KeyCode::Backspace => app.editor.backspace(),
                        KeyCode::Delete => app.editor.delete_char(),
                        KeyCode::Up => app.editor.move_cursor(-1, 0),
                        KeyCode::Down => app.editor.move_cursor(1, 0),
                        KeyCode::Left => app.editor.move_cursor(0, -1),
                        KeyCode::Right => app.editor.move_cursor(0, 1),
                        _ => {}
                    }
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn render_ui(frame: &mut ratatui::Frame, app: &mut App) {
    let diag_height = if app.diagnostics.is_visible { 8 } else { 0 };
    let palette_height = if app.palette.is_active { 10 } else { 2 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(diag_height),
            Constraint::Length(palette_height),
        ])
        .split(frame.area());

    let editor_area = chunks[0];
    let diag_area = chunks[1];
    let status_area = chunks[2];

    let visible_height = editor_area.height.saturating_sub(2) as usize;
    let visible_width = editor_area.width.saturating_sub(2) as usize;

    app.editor.scroll_into_view(visible_width, visible_height);

    let visible_lines: Vec<Line> = app
        .editor
        .lines
        .iter()
        .skip(app.editor.row_offset)
        .take(visible_height)
        .map(|line| {
            let scrolled_line: String = line.chars().skip(app.editor.col_offset).collect();
            Line::from(scrolled_line)
        })
        .collect();

    let lsp_badge = match &app.lsp.status {
        LspStatus::Connected => format!("[LSP: {} Ready]", app.lsp.server_name),
        LspStatus::MissingBinary(_) => "[LSP: Missing Server]".to_string(),
        LspStatus::Error(_) => "[LSP: Failed]".to_string(),
        LspStatus::Connecting => "[LSP: Connecting...]".to_string(),
        LspStatus::Disconnected => "".to_string(),
    };

    let title = format!(
        " ☯️ ELIDE v1.2.0 - {} {} {} ",
        app.editor.filename.as_deref().unwrap_or("[Untitled Buffer]"),
        if app.editor.is_dirty { "*" } else { "" },
        lsp_badge
    );

    let editor_widget = Paragraph::new(visible_lines)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(editor_widget, editor_area);

    if app.diagnostics.is_visible {
        app.diagnostics.render(frame, diag_area);
    }

    if !app.palette.is_active {
        let visual_col = app.editor.visual_cursor_col();
        let screen_cursor_row = (app.editor.cursor.row.saturating_sub(app.editor.row_offset)) as u16 + 1;
        let screen_cursor_col = (visual_col.saturating_sub(app.editor.col_offset)) as u16 + 1;

        frame.set_cursor_position((editor_area.x + screen_cursor_col, editor_area.y + screen_cursor_row));
    }

    let (status_text, status_style) = if app.palette.is_active {
        let content = if app.status_message.is_empty() {
            format!("Alt+T Palette > {}_", app.palette.input_buffer)
        } else {
            format!("Alt+T Palette > {}_\n--- Execution / Output Log ---\n{}", app.palette.input_buffer, app.status_message)
        };
        (content, colors::warning_style())
    } else {
        let status = if app.status_message.starts_with("Unknown command:") {
            colors::Status::UnknownCommand
        } else if app.status_message.starts_with("Unknown code:") {
            colors::Status::UnknownCode
        } else if app.status_message.starts_with("Warning:") {
            colors::Status::Warning
        } else if app.last_success {
            colors::Status::Success
        } else {
            colors::Status::Error
        };

        (format!(" {}", app.status_message), colors::style_for_status(status))
    };

    let status_widget = if app.palette.is_active {
        Paragraph::new(status_text)
            .style(status_style)
            .wrap(Wrap { trim: false })
            .scroll((app.status_scroll, 0))
            .block(Block::default().borders(Borders::ALL).title(format!(" Terminal / Palette (Scroll: ↑/↓ | Esc: Close) [{}] ", app.status_scroll)))
    } else {
        Paragraph::new(status_text)
            .style(status_style)
            .wrap(Wrap { trim: false })
    };

    frame.render_widget(status_widget, status_area);
}
