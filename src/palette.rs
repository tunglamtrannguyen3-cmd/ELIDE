use crate::editor::Editor;
use crate::process;                               
/// Represents distinct actions parsed from Alt+T overlay inputs
#[derive(Debug, PartialEq, Eq, Clone)]            pub enum PaletteAction {
    Save,                                             Compile,
    Debug,
    Info,                                             Help,
    Bro, // Emergency vent & game recommendation option
    RunBash(String),
    Unknown(String),                                  Empty,

    // Quản lý file nâng cao                          New(String),
    Code(String),                                     Switch(String),
                                                      // Cấu hình Build Lệnh Tùy Biến cho Ngôn ngữ lạ                                                     SetBuild(String),
}                                                 
pub struct Palette {                                  pub input_buffer: String,
    pub is_active: bool,                          }

impl Palette {                                        pub fn new() -> Self {
        Self {                                                input_buffer: String::new(),
            is_active: false,                             }
    }                                             
    /// Toggle command palette overlay state          pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
        if !self.is_active {                                  self.input_buffer.clear();
        }                                             }
                                                      /// Helper nội bộ để trích xuất và làm sạch tham số từ chuỗi lệnh                                   fn extract_arg(cmd: &str, prefix: &str) -> String {                                                     cmd[prefix.len()..]
            .trim()                                           .trim_matches('"')  // Xóa dấu ngoặc kép đôi nếu có                                                 .trim_matches('\'') // Xóa dấu ngoặc đơn nếu có                                                     .to_string()
    }                                             
    /// Parse raw text string entered into Alt+T prompt
    pub fn parse_command(&self) -> PaletteAction {
        let trimmed = self.input_buffer.trim();
        if trimmed.is_empty() {
            return PaletteAction::Empty;
        }                                         
        // 1. Kiểm tra các lệnh tĩnh & Vim-style shortcuts
        match trimmed {
            "-s" | "--save" | "save" | ":w" => return PaletteAction::Save,
            "-c" | "--compile" | "compile" | "build" | ":b" => return PaletteAction::Compile,
            "-d" | "--debug" | "debug" => return PaletteAction::Debug,                                          "-i" | "--info" | "info" => return PaletteAction::Info,                                             "-h" | "-?" | "--help" | "help" | "?" => return PaletteAction::Help,
            "-bro!" | "bro" | "bro!" => return PaletteAction::Bro,                                              _ => {}
        }                                         
        // 2. Kiểm tra các lệnh động có tiền tố và tham số đi kèm
        if trimmed.starts_with("new ") {
            PaletteAction::New(Self::extract_arg(trimmed, "new "))                                          } else if trimmed.starts_with("code ") {
            PaletteAction::Code(Self::extract_arg(trimmed, "code "))                                        } else if trimmed.starts_with("switch ") {
            PaletteAction::Switch(Self::extract_arg(trimmed, "switch "))                                    } else if trimmed.starts_with("set-build ") {
            PaletteAction::SetBuild(Self::extract_arg(trimmed, "set-build "))
        } else if trimmed.starts_with("sh ") {
            let raw_bash = trimmed[3..].trim().to_string();
            PaletteAction::RunBash(raw_bash)
        } else {
            // Tự động chuyển thẳng lệnh vào Shell nếu không khớp từ khóa đặc biệt nào
            PaletteAction::RunBash(trimmed.to_string())
        }                                             }

    /// Executes the parsed action and returns a status string + success boolean
    pub async fn execute_action(
        &mut self,
        action: PaletteAction,
        editor: &mut Editor,                          ) -> (String, bool) {                                 self.input_buffer.clear();                        self.is_active = false;                                                                             match action {                                        PaletteAction::Save => match editor.save() {                                                            Ok(msg) => (msg, true),                           Err(e) => (format!("Save Error: {}", e), false),
            },
            PaletteAction::Compile => {
                match process::compile_file(editor).await {
                    Ok(res) => (res.output, res.success),
                    Err(e) => (format!("Build Failed: {}", e), false),
                }
            }
            PaletteAction::Debug => {
                let target = editor.filename.as_deref().unwrap_or("main.rs");
                let debug_cmd = format!("gdb --batch -ex r -ex bt --args ./{}", target);
                match process::run_bash_cmd(&debug_cmd).await {
                    Ok(res) => (res.output, res.success),
                    Err(e) => (format!("Debug Exec Error: {}", e), false),
                }
            }
            PaletteAction::Info => {
                let info = format!(
                    "ℹ️ [ELIDE INFO]\n\
                     • Version       : v0.3.1\n\
                     • Author        : Eggchese\n\
                     • Email         : (tunglamtrannguyen3@gmail.com)\n\
                     • Favorite Idol : Dream\n\
                     • Skills        : Systems Programming, Rust, Linux Kernel, TUI Arch\n\
                     • Birthdate     : January 6, 2013\n\
                     -----------------------------------\n\
                     • Target File   : {}\n\                           • Total Lines   : {}\n\
                     • Cursor Pos    : Row {}, Col {}\n\
                     • Custom Build  : {}\n\
                     • Target OS/Arch: {} / {}",
                    editor.filename.as_deref().unwrap_or("[Untitled]"),
                    editor.lines.len(),
                    editor.cursor.row + 1,
                    editor.cursor.col + 1,
                    editor.custom_build_cmd.as_deref().unwrap_or("None (Auto-detect)"),
                    std::env::consts::OS,
                    std::env::consts::ARCH
                );
                (info, true)

            }
            PaletteAction::Help => (
                "📖 [ELIDE COMMAND PALETTE MANUAL]\n\
                 • Flags: -c (Compile), -d (Debug/GDB), -s (Save), -i (Info), -bro! (Vent) | :w, :b\n\
                 • File Cmds: new <file>, code <file>, switch <file>\n\
                 • Shell & Overrides: set-build <cmd>, sh <cmd> (or type direct shell command)\n\
                 • Terminal Navigation: Scroll logs using ↑ / ↓ / PageUp / PageDown"
                    .to_string(),
                true,
            ),
            PaletteAction::Bro => (
                "It's seems like you have a hard time, here's some game we recommend you:\n\
                 • Touhou Project: a 'nice' Japanese game build by one person I once playing it and absolute perfection, also all cast are female except one person\n\
                 • Elden Ring: a medival style game for a nice adventure\n\
                 • Minecraft: a nice open world game, perfect if Verity is there".to_string(),                      true,
            ),
            PaletteAction::RunBash(cmd) => match process::run_bash_cmd(&cmd).await {
                Ok(res) => (res.output, res.success),
                Err(e) => (format!("Bash Error: {}", e), false),
            },

            PaletteAction::New(filename) => {
                if filename.is_empty() {
                    ("Error: Filename cannot be empty.".to_string(), false)
                } else {
                    editor.filename = Some(filename.clone());
                    editor.lines = vec![String::new()];
                    editor.cursor = crate::editor::Cursor { row: 0, col: 0 };
                    editor.is_dirty = true;
                    editor.custom_build_cmd = None;
                    (format!("Created new buffer: {}", filename), true)
                }
            }

            PaletteAction::Code(filename) => {
                if filename.is_empty() {
                    ("Error: Specify a file to open.".to_string(), false)
                } else {
                    match editor.open(&filename) {
                        Ok(_) => (format!("Opened file: {}", filename), true),                                              Err(e) => (format!("Failed to open {}: {}", filename, e), false),
                    }
                }
            }

            PaletteAction::Switch(filename) => {
                if filename.is_empty() {
                    ("Error: Specify a file to switch to.".to_string(), false)
                } else {
                    match editor.open(&filename) {
                        Ok(_) => (format!("Switched to file: {}", filename), true),
                        Err(e) => (format!("Switch failed: {}", e), false),
                    }
                }
            }
                                                              PaletteAction::SetBuild(cmd) => {
                if cmd.is_empty() {                                   editor.custom_build_cmd = None;
                    ("Cleared custom build command. Fallback to default compiler.".to_string(), true)
                } else {
                    editor.custom_build_cmd = Some(cmd.clone());
                    (format!("Custom build command set to: '{}'", cmd), true)
                }
            }                                     
            PaletteAction::Unknown(cmd) => (format!("Unknown flag/command: '{}'", cmd), false),
            PaletteAction::Empty => ("No command entered.".to_string(), true),                              }
    }
}