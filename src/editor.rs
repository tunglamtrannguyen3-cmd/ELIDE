use anyhow::{Context, Result};
use std::fs;
use unicode_width::UnicodeWidthStr;



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
    pub custom_build_cmd: Option<String>,
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
            custom_build_cmd: None,
            row_offset: 0,
            col_offset: 0,
        }
    }

    /// Helper to convert a character column index into a UTF-8 byte offset
    fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
        s.char_indices()
            .nth(char_idx)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| s.len())
    }

    /// Calculate visual display column on terminal (accounts for wide 2-cell CJK/Emoji characters)
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

    /// Keep the cursor within viewable window bounds using visual widths
    pub fn scroll_into_view(&mut self, visible_width: usize, visible_height: usize) {
        if visible_height == 0 || visible_width == 0 {
            return;
        }

        // Vertical Scrolling
        if self.cursor.row < self.row_offset {
            self.row_offset = self.cursor.row;
        } else if self.cursor.row >= self.row_offset + visible_height {
            self.row_offset = self.cursor.row - visible_height + 1;
        }

        // Horizontal Scrolling using visual terminal width
        let visual_col = self.visual_cursor_col();
        if visual_col < self.col_offset {
            self.col_offset = visual_col;
        } else if visual_col >= self.col_offset + visible_width {
            self.col_offset = visual_col - visible_width + 1;
        }
    }

    /// Load a file from disk into the editor buffer
    pub fn open(&mut self, path: &str) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        self.lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };

        self.filename = Some(path.to_string());
        self.cursor = Cursor { row: 0, col: 0 };
        self.row_offset = 0;
        self.col_offset = 0;
        self.is_dirty = false;
        self.custom_build_cmd = None;
        Ok(())
    }

    /// Save current editor buffer back to disk
    pub fn save(&mut self) -> Result<String> {
        if let Some(ref path) = self.filename {
            let mut content = self.lines.join("\n");
            content.push('\n'); // POSIX-compliant trailing newline

            fs::write(path, content)
                .with_context(|| format!("Failed to save file: {}", path))?;
            self.is_dirty = false;
            Ok(format!("Saved to {}", path))
        } else {
            Ok("No file name provided. Use 'new <filename>' or 'save <filename>'".to_string())
        }
    }

    /// Insert a character at current cursor position safely
    pub fn insert_char(&mut self, c: char) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        let line = &mut self.lines[self.cursor.row];
        let byte_idx = Self::char_to_byte_idx(line, self.cursor.col);
        line.insert(byte_idx, c);
        self.cursor.col += 1;
        self.is_dirty = true;
    }

    /// Handle newline keypress (Enter) safely
    pub fn insert_newline(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
            self.cursor.row = 0;
            self.cursor.col = 0;
            return;
        }
        let current_line = &mut self.lines[self.cursor.row];
        let byte_idx = Self::char_to_byte_idx(current_line, self.cursor.col);
        let remainder = current_line.split_off(byte_idx);
        self.lines.insert(self.cursor.row + 1, remainder);
        self.cursor.row += 1;
        self.cursor.col = 0;
        self.is_dirty = true;
    }

    /// Delete character behind cursor (Backspace) safely
    pub fn backspace(&mut self) {
        if self.cursor.col > 0 {
            let line = &mut self.lines[self.cursor.row];
            let target_char_idx = self.cursor.col - 1;

            if let Some((byte_idx, _)) = line.char_indices().nth(target_char_idx) {
                line.remove(byte_idx);
                self.cursor.col -= 1;
                self.is_dirty = true;
            }
        } else if self.cursor.row > 0 {
            // Join current line to previous line
            let current_line = self.lines.remove(self.cursor.row);
            self.cursor.row -= 1;
            self.cursor.col = self.lines[self.cursor.row].chars().count();
            self.lines[self.cursor.row].push_str(&current_line);
            self.is_dirty = true;
        }
    }

    /// Delete character under cursor (Delete key) safely
    pub fn delete_char(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        let line_char_count = self.lines[self.cursor.row].chars().count();

        if self.cursor.col < line_char_count {
            let line = &mut self.lines[self.cursor.row];
            if let Some((byte_idx, _)) = line.char_indices().nth(self.cursor.col) {
                line.remove(byte_idx);
                self.is_dirty = true;
            }
        } else if self.cursor.row + 1 < self.lines.len() {
            // Pull next line up into current line
            let next_line = self.lines.remove(self.cursor.row + 1);
            self.lines[self.cursor.row].push_str(&next_line);
            self.is_dirty = true;
        }
    }

    /// Safely move cursor left, right, up, down with line wrapping
    pub fn move_cursor(&mut self, row_delta: isize, col_delta: isize) {
        if self.lines.is_empty() {
            return;
        }

        // Handle Row Movement
        if row_delta != 0 {
            let new_row = self.cursor.row as isize + row_delta;
            if new_row >= 0 && (new_row as usize) < self.lines.len() {
                self.cursor.row = new_row as usize;
                let max_col = self.lines[self.cursor.row].chars().count();
                self.cursor.col = self.cursor.col.min(max_col);
            }
        }

        // Handle Col Movement
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

