// src/palette.rs
use crate::editor::{Cursor, Editor};
use crate::process;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PaletteAction {
    Save,
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
    Copy(Option<String>),
    Paste,
    AiCheck(String), // Handles both 'check <token> <url>' and 'check <query>'
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
            "-i" | "--info" | "info" => return PaletteAction::Info,
            "-h" | "-?" | "--help" | "help" | "?" => return PaletteAction::Help,
            "-bro!" | "bro" | "bro!" => return PaletteAction::Bro,
            ":funcs" => return PaletteAction::Funcs,
            "copy" | "\"+y" => return PaletteAction::Copy(None),
            "paste" | "\"+p" => return PaletteAction::Paste,
            _ => {}
        }

        if trimmed.starts_with("copy ") {
            PaletteAction::Copy(Some(Self::extract_arg(trimmed, "copy ")))
        } else if trimmed.starts_with("\"+y ") {
            PaletteAction::Copy(Some(Self::extract_arg(trimmed, "\"+y ")))
        } else if trimmed.starts_with("--im-") {
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
        } else if trimmed.starts_with("check ") {
            let args = trimmed[6..].trim().to_string();
            PaletteAction::AiCheck(args)
        } else {
            PaletteAction::UnknownCommand(trimmed.to_string())
        }
    }

    pub async fn execute_action(
        &mut self,
        action: PaletteAction,
        editor: &mut Editor,
        clipboard: &mut String,
    ) -> (String, bool) {
        self.input_buffer.clear();

        match action {
            PaletteAction::Save => match editor.save() {
                Ok(msg) => (msg, true),
                Err(e) => (format!("Save Error: {}", e), false),
            },
            PaletteAction::Copy(range_opt) => {
                match range_opt {
                    None => {
                        *clipboard = editor.lines.join("\n");
                        ("File copied to internal clipboard (all lines)".to_string(), true)
                    }
                    Some(range_str) => {
                        let range_str = range_str.trim();
                        if range_str.is_empty() {
                            *clipboard = editor.lines.join("\n");
                            return ("File copied to internal clipboard (all lines)".to_string(), true);
                        }

                        let parts: Vec<&str> = range_str.split('-').map(|s| s.trim()).collect();
                        let (start, end) = if parts.len() == 1 {
                            if let Ok(n) = parts[0].parse::<usize>() {
                                (n, n)
                            } else {
                                return ("Error: Invalid line number.".to_string(), false);
                            }
                        } else if parts.len() == 2 {
                            if let (Ok(s), Ok(e)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                                (s, e)
                            } else {
                                return ("Error: Invalid line numbers. Use format a-b (e.g. copy 1-5)".to_string(), false);
                            }
                        } else {
                            return ("Error: Invalid range format. Use format a-b or single line (e.g. copy 5)".to_string(), false);
                        };

                        if editor.lines.is_empty() {
                            ("Editor buffer is empty.".to_string(), false)
                        } else if start == 0 || end == 0 {
                            ("Error: Line numbers start at 1 (no line 0).".to_string(), false)
                        } else if start > editor.lines.len() || end > editor.lines.len() {
                            (format!("Error: Line range out of bounds (file has only {} lines).", editor.lines.len()), false)
                        } else {
                            let start_idx = start - 1;
                            let end_idx = end - 1;
                            if start_idx <= end_idx {
                                let sliced: Vec<String> = editor.lines[start_idx..=end_idx].to_vec();
                                *clipboard = sliced.join("\n");
                                (format!("Copied lines {}-{} to internal clipboard", start, end), true)
                            } else {
                                ("Error: Start line cannot be greater than end line.".to_string(), false)
                            }
                        }
                    }
                }
            }
           PaletteAction::Paste => {
    let clip_lines: Vec<String> = clipboard.lines().map(|s| s.to_string()).collect();
    if clip_lines.is_empty() {
        ("Clipboard is empty.".to_string(), false)
    } else {
        let row = editor.cursor.row.min(editor.lines.len());
        if row < editor.lines.len() {
            let current_line = &mut editor.lines[row];
            let byte_idx = Editor::char_to_byte_idx(current_line, editor.cursor.col);
            let remainder = current_line.split_off(byte_idx);
            
            if clip_lines.len() == 1 {
                current_line.push_str(&clip_lines[0]);
                current_line.push_str(&remainder);
                editor.cursor.col += clip_lines[0].chars().count();
            } else {
                let first_pasted = clip_lines[0].clone();
                let last_pasted = clip_lines.last().unwrap().clone();
                
                current_line.push_str(&first_pasted);
                
                let mut new_lines = vec![current_line.clone()];
                for middle_line in &clip_lines[1..clip_lines.len() - 1] {
                    new_lines.push(middle_line.clone());
                }
                
                let mut final_line = last_pasted.clone(); // ✅ Added .clone() here
                final_line.push_str(&remainder);
                new_lines.push(final_line);
                
                editor.lines.splice(row..=row, new_lines);
                editor.cursor.row += clip_lines.len() - 1;
                editor.cursor.col = last_pasted.chars().count(); // ✅ Works perfectly now
            }
        } else {
            editor.lines.extend(clip_lines);
        }
        editor.is_dirty = true;
        ("Pasted from internal clipboard at cursor".to_string(), true)
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
- Author        : Eggchese\n\
- Email         : (tunglamtrannguyen3@gmail.com)\n\
- Favorite Idol : Yatsuzume, All Minecraft Manhunt Speedruners\n\
- Skills        : Systems Programming, Rust, C, Ada, Micro-skills, Cooking, Chess, Touhou on Lunatic\n\
                     -----------------------------------\n\
                     ℹ️ [EDITOR INFO]\n\
- Version       : v2.1.1\n\
- Target File   : {}\n\
- Total Lines   : {}\n\
- Cursor Pos    : Row {}, Col {}\n\
- Target OS/Arch: {} / {}",
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
- Flags: -s (Save), -i (Info), copy [a-b] / paste, -bro! (Vent)\n\
- File Cmds: new <file>, code <file>, switch <file>\n\
- AI Audit & Config: check <token> <url> OR check <query>\n\
- Foreground Process: acel <cmd>\n\
- Advanced: :funcs (Line functions)\n\
- LSP and Bash: lsp <cmd>, sh <cmd>"
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
                let responses = match joke.as_str() {
                    "--im-stuck" => vec![
                        "Have you tried turning it off and on again? Or maybe read the docs... jk, just ask ChatGPT.",
                        "Rubber duck debugging time: explain your entire code line-by-line to an inanimate object.",
                        "Take a 5-minute walk. Your subconscious will solve the bug while you breathe fresh air.",
                        "Delete the whole function and rewrite it. Sometimes starting clean is faster than fixing spaghetti."
                    ],
                    "--im-bored" => vec![
                        "Type -bro! for a game recommendation, or rewrite this editor in Brainfuck.",
                        "Try optimizing your loops until your CPU starts screaming.",
                        "Go refactor that one messy file you've been avoiding for three weeks.",
                        "Challenge yourself to write a game in 10 lines of Bash."
                    ],
                    "--im-hungry" => vec![
                        "Go eat! Your code won't compile better on an empty stomach.",
                        "Instant ramen is calling your name. Go fuel up.",
                        "Step away from the screen and grab some water and a snack.",
                        "A well-fed programmer finds bugs twice as fast."
                    ],
                    "--im-tired" => vec![
                        "Git commit, git push, go to bed. The bug will be there tomorrow.",
                        "Sleep is the ultimate compiler optimization for your brain.",
                        "Close your laptop. Nothing good gets coded past 2 AM.",
                        "Rest now, conquer the segmentation fault tomorrow."
                    ],
                    _ => vec![
                        "I am an editor, not a therapist. But you got this!",
                        "Keep pushing forward, you're closer than you think.",
                        "Every expert was once a beginner staring at a blank screen."
                    ],
                };

                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as usize;

                let chosen = responses[seed % responses.len()];
                (format!("🤖 {}", chosen), true)
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
                         - {}\n\n\
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
            PaletteAction::AiCheck(query) => {
                if query.is_empty() {
                    ("Error: Specify a query or use 'check <token> <url>'".to_string(), false)
                } else {
                    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    match crate::ai::run_check(&root, &query).await {
                        Ok(res) => (format!("🤖 [ELIDE AI AUDIT]\n\n{}", res), true),
                        Err(e) => (format!("❌ AI Error: {}", e), false),
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