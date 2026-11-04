// src/editor.rs
use anyhow::{Context, Result};
use arboard::Clipboard;
use base64::{engine::general_purpose, Engine as _};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use std::fs;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

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

    pub fn set_text(&mut self, text: &str) -> io::Result<()> {
        self.fallback_buffer = text.to_string();

        if let Some(ref mut clipboard) = self.ctx {
            if let Err(e) = clipboard.set_text(text) {
                let _ = writeln!(io::stderr(), "[Warning] arboard clipboard write failed: {}", e);
            }
        }

        let b64 = general_purpose::STANDARD.encode(text.as_bytes());
        let sequence = format!("\x1b]52;c;{}\x07", b64);
        let mut stdout = io::stdout();
        stdout.write_all(sequence.as_bytes())?;
        stdout.flush()?;

        Ok(())
    }

    pub fn get_text(&mut self) -> String {
        if let Some(ref mut clipboard) = self.ctx {
            match clipboard.get_text() {
                Ok(text) if !text.is_empty() => return text,
                _ => {}
            }
        }
        self.fallback_buffer.clone()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize, // Character index in current line
}

pub struct Editor {
    pub filename: Option<String>,
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub is_dirty: bool,
    pub row_offset: usize,
    pub col_offset: usize, // Measured in visual terminal columns
}

impl Editor {
    pub fn new() -> Self {
        Self {
            filename: None,
            lines: vec![String::new()],
            cursor: Cursor { row: 0, col: 0 },
            is_dirty: false,
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
        s.char_indices()
            .nth(char_idx)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| s.len())
    }

    pub fn visual_cursor_col(&self) -> usize {
        if let Some(line) = self.lines.get(self.cursor.row) {
            line.chars()
                .take(self.cursor.col)
                .map(|c| {
                    let mut buf = [0; 4];
                    let str_slice = c.encode_utf8(&mut buf);
                    UnicodeWidthStr::width(str_slice)
                })
                .sum()
        } else {
            0
        }
    }

    pub fn scroll_into_view(&mut self, visible_width: usize, visible_height: usize) {
        if visible_height == 0 || visible_width == 0 {
            return;
        }

        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor.row = self.cursor.row.min(self.lines.len() - 1);

        if self.cursor.row < self.row_offset {
            self.row_offset = self.cursor.row;
        } else if self.cursor.row >= self.row_offset + visible_height {
            self.row_offset = self.cursor.row.saturating_sub(visible_height) + 1;
        }

        let visual_col = self.visual_cursor_col();
        if visual_col < self.col_offset {
            self.col_offset = visual_col;
        } else if visual_col >= self.col_offset + visible_width {
            self.col_offset = visual_col.saturating_sub(visible_width) + 1;
        }
    }

    pub fn render_visible_lines(
        &mut self,
        width: usize,
        height: usize,
        text_color: Color,
        comment_color: Color,
    ) -> Vec<Line<'static>> {
        self.scroll_into_view(width, height);

        self.lines
            .iter()
            .enumerate()
            .skip(self.row_offset)
            .take(height)
            .map(|(i, l)| {
                let mut current_width = 0;
                let mut display_str = String::new();

                for c in l.chars() {
                    let mut buf = [0; 4];
                    let char_width = UnicodeWidthStr::width(c.encode_utf8(&mut buf));

                    if current_width >= self.col_offset + width {
                        break;
                    }
                    if current_width >= self.col_offset {
                        display_str.push(c);
                    }
                    current_width += char_width;
                }

                Line::from(vec![
                    Span::styled(format!("{:3} │ ", i + 1), Style::default().fg(comment_color)),
                    Span::styled(display_str, Style::default().fg(text_color)),
                ])
            })
            .collect()
    }

    pub fn open(&mut self, path: &str) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        self.lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.replace("\r\n", "\n").lines().map(|s| s.to_string()).collect()
        };

        self.filename = Some(path.to_string());
        self.cursor = Cursor { row: 0, col: 0 };
        self.row_offset = 0;
        self.col_offset = 0;
        self.is_dirty = false;
        Ok(())
    }

    pub fn save(&mut self) -> Result<String> {
        if let Some(ref path) = self.filename {
            let mut content = self.lines.join("\n");
            content.push('\n');

            fs::write(path, content)
                .with_context(|| format!("Failed to save file: {}", path))?;
            self.is_dirty = false;
            Ok(format!("Saved to {}", path))
        } else {
            Ok("No file name provided. Use 'new <filename>' or 'save <filename>'".to_string())
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor.row = self.cursor.row.min(self.lines.len() - 1);
        let line = &mut self.lines[self.cursor.row];
        let max_col = line.chars().count();
        self.cursor.col = self.cursor.col.min(max_col);

        let byte_idx = Self::char_to_byte_idx(line, self.cursor.col);
        line.insert(byte_idx, c);
        self.cursor.col += 1;
        self.is_dirty = true;
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..4 {
            self.insert_char(' ');
        }
    }

    pub fn insert_newline(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
            self.cursor.row = 0;
            self.cursor.col = 0;
            return;
        }
        self.cursor.row = self.cursor.row.min(self.lines.len() - 1);
        let current_line = &mut self.lines[self.cursor.row];
        let max_col = current_line.chars().count();
        self.cursor.col = self.cursor.col.min(max_col);

        let byte_idx = Self::char_to_byte_idx(current_line, self.cursor.col);
        let remainder = current_line.split_off(byte_idx);
        self.lines.insert(self.cursor.row + 1, remainder);
        self.cursor.row += 1;
        self.cursor.col = 0;
        self.is_dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        self.cursor.row = self.cursor.row.min(self.lines.len() - 1);

        if self.cursor.col > 0 {
            let line = &mut self.lines[self.cursor.row];
            let target_char_idx = self.cursor.col - 1;

            if let Some((byte_idx, _)) = line.char_indices().nth(target_char_idx) {
                line.remove(byte_idx);
                self.cursor.col -= 1;
                self.is_dirty = true;
            }
        } else if self.cursor.row > 0 {
            let current_line = self.lines.remove(self.cursor.row);
            self.cursor.row -= 1;
            self.cursor.col = self.lines[self.cursor.row].chars().count();
            self.lines[self.cursor.row].push_str(&current_line);
            self.is_dirty = true;
        }
    }

    pub fn delete_char(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        self.cursor.row = self.cursor.row.min(self.lines.len() - 1);
        let line_char_count = self.lines[self.cursor.row].chars().count();

        if self.cursor.col < line_char_count {
            let line = &mut self.lines[self.cursor.row];
            if let Some((byte_idx, _)) = line.char_indices().nth(self.cursor.col) {
                line.remove(byte_idx);
                self.is_dirty = true;
            }
        } else if self.cursor.row + 1 < self.lines.len() {
            let next_line = self.lines.remove(self.cursor.row + 1);
            self.lines[self.cursor.row].push_str(&next_line);
            self.is_dirty = true;
        }
    }

    pub fn move_cursor(&mut self, row_delta: isize, col_delta: isize) {
        if self.lines.is_empty() {
            return;
        }

        if row_delta != 0 {
            let new_row = (self.cursor.row as isize + row_delta).clamp(0, self.lines.len() as isize - 1);
            self.cursor.row = new_row as usize;
            let max_col = self.lines[self.cursor.row].chars().count();
            self.cursor.col = self.cursor.col.min(max_col);
        }

        if col_delta < 0 {
            if self.cursor.col > 0 {
                self.cursor.col -= 1;
            } else if self.cursor.row > 0 {
                self.cursor.row -= 1;
                self.cursor.col = self.lines[self.cursor.row].chars().count();
            }
        } else if col_delta > 0 {
            let max_col = self.lines[self.cursor.row].chars().count();
            if self.cursor.col < max_col {
                self.cursor.col += 1;
            } else if self.cursor.row + 1 < self.lines.len() {
                self.cursor.row += 1;
                self.cursor.col = 0;
            }
        }
    }
}