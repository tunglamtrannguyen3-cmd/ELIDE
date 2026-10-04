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
    Funcs,
    Joke(String),
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
            "-d" | "--debug" | "debug" | ":verify" => return PaletteAction::Debug,
            "-i" | "--info" | "info" => return PaletteAction::Info,
            "-h" | "-?" | "--help" | "help" | "?" => return PaletteAction::Help,
            "-bro!" | "bro" | "bro!" => return PaletteAction::Bro,
            ":funcs" => return PaletteAction::Funcs,
            _ => {}
        }

        if trimmed.starts_with("--im-") {
            PaletteAction::Joke(trimmed.to_string())
        } else if trimmed.starts_with("new ") {
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
                let fname = editor.filename.as_deref().unwrap_or("");
                
                // Route to language-specific formal verification or syntax checkers
                let cmd = if fname.ends_with(".rs") || fname.is_empty() {
                    "cargo check".to_string() 
                } else if fname.ends_with(".adb") || fname.ends_with(".ads") {
                    "gnatprove --mode=prove".to_string() // SPARK verification for Ada
                } else if fname.ends_with(".c") || fname.ends_with(".cpp") || fname.ends_with(".h") {
                    "cppcheck --enable=all .".to_string() // Static analysis for C/C++
                } else if fname.ends_with(".go") {
                    "go vet ./...".to_string() // Standard Go static analysis
                } else if fname.ends_with(".py") {
                    format!("mypy {}", fname) // Type checking for Python
                } else if fname.ends_with(".ts") {
                    format!("tsc --noEmit {}", fname) // TypeScript verification
                } else if fname.ends_with(".js") {
                    format!("eslint {}", fname) // JavaScript linting
                } else if fname.ends_with(".sh") || fname.ends_with(".bash") {
                    format!("shellcheck {}", fname) // Shell script analysis
                } else if fname.ends_with(".java") {
                    format!("javac -Xlint:all {}", fname) // Java linting
                } else if fname.ends_with(".zig") {
                    format!("zig ast-check {}", fname) // Zig syntax check
                } else if fname.ends_with(".rb") {
                    format!("rubocop {}", fname) // Ruby static code analyzer
                } else {
                    "echo 'we don't support this file type'".to_string()
                };
                
                match process::run_bash_cmd(&cmd).await {
                    Ok(res) => {
                        let out = if res.output.trim().is_empty() { 
                            format!("Verification passed cleanly via `{}`.", cmd) 
                        } else { 
                            res.output 
                        };
                        (out, res.success)
                    },
                    Err(e) => (format!("Verify/Debug Error: {}", e), false),
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
                     • Version       : v2.1.0\n\
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
                 • Flags: -s (Save), -d/:verify (Check/Debug), -i (Info), -bro! (Vent)\n\
                 • File Cmds: new <file>, code <file>, switch <file>\n\
                 • Foreground Process: acel <cmd> \n\
                 • Advanced: :funcs (List functions)\n\
                 • LSP and Bash: lsp <cmd>, sh <cmd>"
                    .to_string(),
                true,
            ),
            PaletteAction::Funcs => {
                let funcs: Vec<String> = editor.lines.iter()
                    .enumerate()
                    .filter(|(_, l)| l.trim().starts_with("fn ") || l.trim().starts_with("pub fn ") || l.trim().starts_with("async fn "))
                    .map(|(i, l)| format!("L{}: {}", i + 1, l.trim().trim_end_matches('{').trim()))
                    .collect();
                    
                if funcs.is_empty() {
                    ("No functions found in current buffer.".to_string(), true)
                } else {
                    (format!("📌 [FUNCTIONS IN FILE]\n{}", funcs.join("\n")), true)
                }
            }
            PaletteAction::Joke(joke) => {
                let msg = match joke.as_str() {
                    "--im-stuck" => "Have you tried turning it off and on again? Or maybe read the docs... jk, just ask ChatGPT.",
                    "--im-bored" => "Type -bro! for a game recommendation, or rewrite this editor in Brainfuck.",
                    "--im-hungry" => "Go eat! Your code won't compile better on an empty stomach.",
                    "--im-tired" => "Git commit, git push, go to bed. The bug will be there tomorrow.",
                    _ => "I am an editor, not a therapist. But you got this!",
                };
                (format!("🤖 {}", msg), true)
            }
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