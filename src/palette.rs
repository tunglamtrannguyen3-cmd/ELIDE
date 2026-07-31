use crate::editor::{Cursor, Editor};
use crate::process;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PaletteAction {
    Save,
    Compile,
    Debug,
    Info,
    Help,
    Bro,
    RunBash(String),
    UnknownCommand(String),
    UnknownCode(String),
    Empty,

    New(String),
    Code(String),
    Switch(String),
    SetBuild(String),
}

pub struct Palette {
    pub input_buffer: String,
    pub is_active: bool,
}

impl Palette {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            is_active: false,
        }
    }

    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
        if !self.is_active {
            self.input_buffer.clear();
        }
    }

    fn extract_arg(cmd: &str, prefix: &str) -> String {
        cmd[prefix.len()..]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string()
    }

    fn detect_language_mismatch(&self, input: &str, current_file: Option<&str>) -> bool {
        let ext = current_file
            .and_then(|f| f.split('.').last())
            .unwrap_or("");

        match ext {
            "cpp" | "c" | "h" => {
                input.contains("fn ") || input.contains("let mut") || input.contains("println!") || input.contains("use crate::")
            }
            "rs" => {
                input.contains("#include") || input.contains("std::cout") || input.contains("using namespace")
            }
            "py" => {
                input.contains("fn ") || input.contains("#include") || input.contains("public static void")
            }
            _ => false,
        }
    }

    pub fn parse_command(&self, current_file: Option<&str>) -> PaletteAction {
        let trimmed = self.input_buffer.trim();
        if trimmed.is_empty() {
            return PaletteAction::Empty;
        }

        match trimmed {
            "-s" | "--save" | "save" | ":w" => return PaletteAction::Save,
            "-c" | "--compile" | "compile" | "build" | ":b" => return PaletteAction::Compile,
            "-d" | "--debug" | "debug" => return PaletteAction::Debug,
            "-i" | "--info" | "info" => return PaletteAction::Info,
            "-h" | "-?" | "--help" | "help" | "?" => return PaletteAction::Help,
            "-bro!" | "bro" | "bro!" => return PaletteAction::Bro,
            _ => {}
        }

        if trimmed.starts_with("new ") {
            PaletteAction::New(Self::extract_arg(trimmed, "new "))
        } else if trimmed.starts_with("code ") {
            PaletteAction::Code(Self::extract_arg(trimmed, "code "))
        } else if trimmed.starts_with("switch ") {
            PaletteAction::Switch(Self::extract_arg(trimmed, "switch "))
        } else if trimmed.starts_with("set-build ") {
            PaletteAction::SetBuild(Self::extract_arg(trimmed, "set-build "))
        } else if trimmed.starts_with("sh ") {
            let raw_bash = trimmed[3..].trim().to_string();
            PaletteAction::RunBash(raw_bash)
        } else if self.detect_language_mismatch(trimmed, current_file) {
            PaletteAction::UnknownCode(trimmed.to_string())
        } else {
            PaletteAction::UnknownCommand(trimmed.to_string())
        }
    }

    pub async fn execute_action(
        &mut self,
        action: PaletteAction,
        editor: &mut Editor,
    ) -> (String, bool) {
        self.input_buffer.clear();
        self.is_active = false;

        match action {
            PaletteAction::Save => match editor.save() {
                Ok(msg) => (msg, true),
                Err(e) => (format!("Save Error: {}", e), false),
            },
            PaletteAction::Compile => match process::compile_file(editor).await {
                Ok(res) => (res.output, res.success),
                Err(e) => (format!("Build Failed: {}", e), false),
            },
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
                     • Target File   : {}\n\
                     • Total Lines   : {}\n\
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
                 • Flags: -c (Compile), -d (Debug), -s (Save), -i (Info), -bro! (Vent)\n\
                 • File Cmds: new <file>, code <file>, switch <file>\n\
                 • Overrides: set-build <cmd>, sh <cmd>"
                    .to_string(),
                true,
            ),
            PaletteAction::Bro => (
                "Take a breather! Here are some recommended games to unwind:\n\n\
                 • Touhou Project (Bullet Hell / STG)\n\
                   Solo-developed iconic Japanese series with intense spell-card patterns and incredible soundtracks.\n\n\
                 • Elden Ring (Action RPG / Open World)\n\
                   Vast open-world dark fantasy filled with deep exploration and rewarding combat.\n\n\
                 • Minecraft (Sandbox / Survival)\n\
                   Infinite creative sandbox to build, relax, and chill after long coding sessions."
                    .to_string(),
                true,
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
                    editor.cursor = Cursor { row: 0, col: 0 };
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
                        Ok(_) => (format!("Opened file: {}", filename), true),
                        Err(e) => (format!("Failed to open {}: {}", filename, e), false),
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
                if cmd.is_empty() {
                    editor.custom_build_cmd = None;
                    ("Cleared custom build command.".to_string(), true)
                } else {
                    editor.custom_build_cmd = Some(cmd.clone());
                    (format!("Custom build command set to: '{}'", cmd), true)
                }
            }
            PaletteAction::UnknownCommand(cmd) => {
                (format!("Unknown command: {}", cmd), false)
            }
            PaletteAction::UnknownCode(code) => {
                (format!("Unknown code: {}", code), false)
            }
            PaletteAction::Empty => ("No command entered.".to_string(), true),
        }
    }
}
