use anyhow::{Context, Result};                    use std::fs;
                                                  #[derive(Default, Debug, Clone)]
pub struct Cursor {
    pub row: usize,                                   pub col: usize,
}
                                                  pub struct Editor {
    pub filename: Option<String>,                     pub lines: Vec<String>,
    pub cursor: Cursor,                               pub is_dirty: bool,
                                                      // Thêm trường lưu trữ lệnh biên dịch tùy biến cho riêng file hiện tại                              pub custom_build_cmd: Option<String>,
                                                      // Viewport scroll offsets to fix the BlindCursor bug                                               pub row_offset: usize,
    pub col_offset: usize,                        }
                                                  impl Editor {
    pub fn new() -> Self {                                Self {
            filename: None,                                   lines: vec![String::new()],
            cursor: Cursor { row: 0, col: 0 },                is_dirty: false,
            custom_build_cmd: None, // Mặc định không có lệnh build custom
            row_offset: 0,                                    col_offset: 0,
        }                                             }
                                                      /// Keep the cursor within viewable window bounds                                                   pub fn scroll_into_view(&mut self, visible_width: usize, visible_height: usize) {                       if visible_height == 0 || visible_width == 0 {                                                          return;
        }                                         
        // Vertical Scrolling                             if self.cursor.row < self.row_offset {
            self.row_offset = self.cursor.row;            } else if self.cursor.row >= self.row_offset + visible_height {                                         self.row_offset = self.cursor.row - visible_height + 1;                                         }

        // Horizontal Scrolling                           if self.cursor.col < self.col_offset {
            self.col_offset = self.cursor.col;            } else if self.cursor.col >= self.col_offset + visible_width {                                          self.col_offset = self.cursor.col - visible_width + 1;                                          }
    }                                             
    /// Load a file from disk into the editor buffer
    pub fn open(&mut self, path: &str) -> Result<()> {                                                      let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        self.lines = if content.is_empty() {                  vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };                                        
        self.filename = Some(path.to_string());           self.cursor = Cursor { row: 0, col: 0 };
        self.row_offset = 0;                              self.col_offset = 0;
        self.is_dirty = false;                    
        // Khi mở một file mới từ đĩa, reset lệnh build custom về mặc định (None)
        // để tránh việc file mới kế thừa nhầm lệnh build của file cũ trước đó.
        self.custom_build_cmd = None;
        Ok(())                                        }
                                                      /// Save current editor buffer back to disk
    pub fn save(&mut self) -> Result<String> {            if let Some(ref path) = self.filename {
            let mut content = self.lines.join("\n");
            content.push('\n'); // POSIX-compliant trailing newline

            fs::write(path, content)                              .with_context(|| format!("Failed to save file: {}", path))?;                        
            self.is_dirty = false;                            Ok(format!("Saved to {}", path))
        } else {                                              Ok("No file name provided. Use 'new <filename>' or 'save <filename>'".to_string())              }
    }                                             
    /// Insert a character at current cursor position
    pub fn insert_char(&mut self, c: char) {              if self.lines.is_empty() {
            self.lines.push(String::new());               }
        let line = &mut self.lines[self.cursor.row];
        line.insert(self.cursor.col, c);
        self.cursor.col += 1;
        self.is_dirty = true;
    }
                                                      /// Handle newline keypress (Enter)
    pub fn insert_newline(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
            self.cursor.row = 0;
            self.cursor.col = 0;
            return;
        }
        let current_line = &mut self.lines[self.cursor.row];
        let remainder = current_line.split_off(self.cursor.col);                                    
        self.lines.insert(self.cursor.row + 1, remainder);
        self.cursor.row += 1;                             self.cursor.col = 0;
        self.is_dirty = true;
    }                                             
    /// Delete character behind cursor (Backspace)    pub fn backspace(&mut self) {
        if self.cursor.col > 0 {                              let line = &mut self.lines[self.cursor.row];                                                        if self.cursor.col <= line.len() {
                line.remove(self.cursor.col - 1);                 self.cursor.col -= 1;
                self.is_dirty = true;                         }
        } else if self.cursor.row > 0 {                       // Join current line to previous line
            let current_line = self.lines.remove(self.cursor.row);
            self.cursor.row -= 1;                             self.cursor.col = self.lines[self.cursor.row].len();                                                self.lines[self.cursor.row].push_str(&current_line);                                                self.is_dirty = true;
        }                                             }
                                                      /// Safely move cursor left, right, up, down without overflow panic                                 pub fn move_cursor(&mut self, row_delta: isize, col_delta: isize) {
        if self.lines.is_empty() {                            return;
        }
        // Handle Row Movement                            let new_row = self.cursor.row as isize + row_delta;                                                 if new_row >= 0 && (new_row as usize) < self.lines.len() {                                              self.cursor.row = new_row as usize;
        }                                         
        // Clamp Col position to length of target line
        let max_col = self.lines[self.cursor.row].len();
        let new_col = self.cursor.col as isize + col_delta;
                                                          if new_col >= 0 {
            self.cursor.col = (new_col as usize).min(max_col);
        } else if self.cursor.row > 0 {                       // Wrap to end of previous line
            self.cursor.row -= 1;                             self.cursor.col = self.lines[self.cursor.row].len();                                            }
    }                                             
    pub fn delete_char(&mut self) {                       if self.cursor.col > 0 {
            let line = &mut self.lines[self.cursor.row];
            if self.cursor.col <= line.len() {                    line.remove(self.cursor.col - 1);
                self.cursor.col -= 1;                             self.is_dirty = true;
            }                                             } else if self.cursor.row > 0 {
            // Join current line to previous line             let current_line = self.lines.remove(self.cursor.row);                                              self.cursor.row -= 1;
            self.cursor.col = self.lines[self.cursor.row].len();
            self.lines[self.cursor.row].push_str(&current_line);
            self.is_dirty = true;                         }
    }                                             
}