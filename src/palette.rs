// src/palette.rs
use crate::editor::{Cursor, Editor};
use crate::process;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PaletteAction {
    Save,
    Debug,
    Info,
    Help,
    Bro,
    RunBash(String),
    UnknownCommand(String),
    Empty,

    New(String),
    Code(String),
    Switch(String),
    SetLsp(String),
    Acel(String),
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

    fn extract_arg(cmd: &str, prefix: &str) -> String {
        cmd[prefix.len()..]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string()
    }

    pub fn parse_command(&self, _current_file: Option<&str>) -> PaletteAction {
        let trimmed = self.input_buffer.trim();
        if trimmed.is_empty() {
            return PaletteAction::Empty;
        }

        match trimmed {
            "-s" | "--save" | "save" | ":w" => return PaletteAction::Save,
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
        } else if trimmed.starts_with("lsp ") {
            PaletteAction::SetLsp(Self::extract_arg(trimmed, "lsp "))
        } else if trimmed.starts_with("sh ") {
            let raw_bash = trimmed[3..].trim().to_string();
            PaletteAction::RunBash(raw_bash)
        } else if trimmed.starts_with("acel ") {
            PaletteAction::Acel(Self::extract_arg(trimmed, "acel "))
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

        match action {
            PaletteAction::Save => match editor.save() {
                Ok(msg) => (msg, true),
                Err(e) => (format!("Save Error: {}", e), false),
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
                   "███████╗██╗     ██╗██████╗ ███████╗\n\
                    ██╔════╝██║     ██║██╔══██╗██╔════╝\n\
                    █████╗  ██║     ██║██║  ██║█████╗  \n\
                    ██╔══╝  ██║     ██║██║  ██║██╔══╝  \n\
                    ███████╗███████╗██║██████╔╝███████╗\n\
                    ╚══════╝╚══════╝╚═╝╚═════╝ ╚══════╝\n\
                    ℹ️ [ABOUT AUTHOR]\n\
                     • Author        : Eggchese\n\
                     • Email         : (tunglamtrannguyen3@gmail.com)\n\
                     • Favorite Idol : Yatsuzume, All Minecraft Manhunt Speedruners\n\
                     • Skills        : Systems Programming, Rust, C, Ada, Micro-skills, Cooking, Chess, Touhou on Lunatic\n\
                     -----------------------------------\n\
                     ℹ️ [EDITOR INFO]\n\
                     • Version       : v2.0.2\n\
                     • Target File   : {}\n\
                     • Total Lines   : {}\n\
                     • Cursor Pos    : Row {}, Col {}\n\
                     • Target OS/Arch: {} / {}",
                    editor.filename.as_deref().unwrap_or("[Untitled]"),
                    editor.lines.len(),
                    editor.cursor.row + 1,
                    editor.cursor.col + 1,
                    std::env::consts::OS,
                    std::env::consts::ARCH
                );
                (info, true)
            }
            PaletteAction::Help => (
                "📖 [ELIDE COMMAND PALETTE MANUAL]\n\
                 • Flags: -s (Save), -d (Debug), -i (Info), -bro! (Vent)\n\
                 • File Cmds: new <file>, code <file>, switch <file>\n\
                 • Runner: acel <cmd> (Replaces legacy compile/set-build)\n\
                 • Overrides: lsp <cmd>, sh <cmd>"
                    .to_string(),
                true,
            ),
            PaletteAction::Bro => {
                let games = [
                    "Minecraft (Sandbox / Build & Relax)",
                    "Touhou Project (Bullet Hell / STG)",
                    "Elden Ring (Action RPG / Exploration)",
                    "CS2 (Tactical Shooter / Competitive)",
                    "Chess (Strategy / Mental Workout)",
                    "Sleep (The Ultimate Recovery Meta)",
                    "Terraria (2D Sandbox Adventure)",
                    "Hollow Knight (Metroidvania Exploration)",
                    "Celeste (Precision Platformer)",
                    "Stardew Valley (Farming & Chill)"
                ];

                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as usize;

                let chosen = games[seed % games.len()];

                (
                    format!(
                        "🎲 [RANDOM BREAK GENERATOR]\n\n\
                         Time to step away from the keyboard. Your assigned activity:\n\n\
                         • {}\n\n\
                         Close the terminal, go clear your head, and come back fresh!",
                        chosen
                    ),
                    true,
                )
            }
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
            PaletteAction::SetLsp(cmd) => {
                if cmd.is_empty() {
                    ("Error: Specify an LSP command (e.g., lsp rust-analyzer).".to_string(), false)
                } else {
                    (format!("Starting LSP: {}", cmd), true)
                }
            }
            PaletteAction::Acel(cmd) => {
                if cmd.is_empty() {
                    ("Error: Specify a command to run (e.g., acel build).".to_string(), false)
                } else {
                    match process::run_interactive_cmd(&cmd).await {
                        Ok(_) => (format!("Returned from acel: {}", cmd), true),
                        Err(e) => (format!("Failed to launch acel app: {}", e), false),
                    }
                }
            }
            PaletteAction::UnknownCommand(cmd) => {
                (format!("Unknown command: {}", cmd), false)
            }
            PaletteAction::Empty => ("No command entered.".to_string(), true),
        }
    }
}