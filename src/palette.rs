// src/palette.rs
use crate::editor::{Cursor, Editor};
use crate::process;
use crate::clipboard::SystemClipboard;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PaletteAction {
    Save(Option<String>), // Supports optional filename argument (e.g., :w file.rs)
    SaveAll,              // Save all open buffers (-sa, wa, :wa)
    Quit,                 // Quit without saving (-l, :q!)
    SaveAndQuit,          // Save and quit (-sl, :wq)
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
    AiCheck(String),
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

        // Exact match commands
        match trimmed {
            "-s" | "--save" | "save" | ":w" => return PaletteAction::Save(None),
            "-sa" | "--save-all" | "wa" | ":wa" => return PaletteAction::SaveAll,
            "-l" | ":q!" | "q!" | "quit!" => return PaletteAction::Quit,
            "-sl" | ":wq" | "wq" => return PaletteAction::SaveAndQuit,
            "-i" | "--info" | "info" => return PaletteAction::Info,
            "-h" | "-?" | "--help" | "help" | "?" => return PaletteAction::Help,
            "-bro!" | "bro" | "bro!" => return PaletteAction::Bro,
            ":funcs" => return PaletteAction::Funcs,
            "copy" | "\"+y" => return PaletteAction::Copy(None),
            "paste" | "\"+p" => return PaletteAction::Paste,
            _ => {}
        }

        // Prefix match commands
        if trimmed.starts_with(":w ") {
            PaletteAction::Save(Some(Self::extract_arg(trimmed, ":w ")))
        } else if trimmed.starts_with("-s ") {
            PaletteAction::Save(Some(Self::extract_arg(trimmed, "-s ")))
        } else if trimmed.starts_with("save ") {
            PaletteAction::Save(Some(Self::extract_arg(trimmed, "save ")))
        } else if trimmed.starts_with("copy ") {
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
        } else if let Some(stripped) = trimmed.strip_prefix("sh ") {
            PaletteAction::RunBash(stripped.trim().to_string())
        } else if trimmed.starts_with("acel ") {
            PaletteAction::Acel(Self::extract_arg(trimmed, "acel "))
        } else if let Some(stripped) = trimmed.strip_prefix("check ") {
            PaletteAction::AiCheck(stripped.trim().to_string())
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
            PaletteAction::Save(path_opt) => {
                if let Some(path) = path_opt {
                    if !path.is_empty() {
                        editor.filename = Some(path);
                    }
                }
                match editor.save() {
                    Ok(msg) => (msg, true),
                    Err(e) => (format!("Save Error: {}", e), false),
                }
            }
            PaletteAction::SaveAll | PaletteAction::Quit | PaletteAction::SaveAndQuit => {
                // Handled at the AppState level
                ("Handled".to_string(), true)
            }
            PaletteAction::Copy(range_opt) => {
                let text_to_copy = match range_opt {
                    None => editor.lines.join("\n"),
                    Some(range_str) => {
                        let range_str = range_str.trim();
                        if range_str.is_empty() {
                            editor.lines.join("\n")
                        } else {
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
                                return ("Editor buffer is empty.".to_string(), false);
                            } else if start == 0 || end == 0 {
                                return ("Error: Line numbers start at 1 (no line 0).".to_string(), false);
                            } else if start > editor.lines.len() || end > editor.lines.len() {
                                return (format!("Error: Line range out of bounds (file has only {} lines).", editor.lines.len()), false);
                            } else {
                                let start_idx = start - 1;
                                let end_idx = end - 1;
                                if start_idx <= end_idx {
                                    editor.lines[start_idx..=end_idx].join("\n")
                                } else {
                                    return ("Error: Start line cannot be greater than end line.".to_string(), false);
                                }
                            }
                        }
                    }
                };

                *clipboard = text_to_copy.clone();

                let mut clip_mgr = SystemClipboard::new();
                match clip_mgr.set_text(&text_to_copy) {
                    Ok(_) => ("Copied! (Synced to System Clipboard, OSC 52, & Internal Buffer)".to_string(), true),
                    Err(e) => (format!("Copied internally, but clipboard sync warned: {}", e), false),
                }
            }
            PaletteAction::Paste => {
                let mut clip_mgr = SystemClipboard::new();
                let clip_text = clip_mgr.get_text();

                let clip_lines: Vec<String> = clip_text.lines().map(|s| s.to_string()).collect();
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
                            
                            let mut final_line = last_pasted.clone();
                            let last_pasted_len = last_pasted.chars().count();
                            final_line.push_str(&remainder);
                            new_lines.push(final_line);
                            
                            editor.lines.splice(row..=row, new_lines);
                            editor.cursor.row += clip_lines.len() - 1;
                            editor.cursor.col = last_pasted_len;
                        }
                    } else {
                        editor.lines.extend(clip_lines);
                    }
                    editor.is_dirty = true;
                    ("Pasted from system clipboard at cursor".to_string(), true)
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
- Version       : v2.1.3\n\
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
- Flags: -s [file] (Save), -sa / wa / :wa (Save All), -l / :q! (Quit), -sl / :wq (Save & Quit)\n\
- File Cmds: new <file>, code <file>, switch <file>\n\
- AI Audit & Config: check <token> <url> OR check <query>\n\
- Foreground Process: acel <cmd>\n\
- Mood Commands: --im-stuck, --im-bored, --im-hungry, --im-tired, --im-stressed, --im-burntout, -bro!\n\
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
                        "Have you tried turning it off and on again? Or maybe read the docs... jk, just ask ELIDE AI.",
                        "Rubber duck debugging time: explain your entire code line-by-line to your coffee mug.",
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
                    "--im-stressed" => vec![
                        "Take 3 deep breaths. Drop your shoulders away from your ears.",
                        "Remember: it's just code. No one's life is on the line right now.",
                        "Step away from the keyboard, stretch your arms, and drink a glass of water.",
                        "Break the problem down into the tiniest possible piece. Solve just that one micro-step."
                    ],
                    "--im-burntout" => vec![
                        "You've been grinding hard. Close ELIDE, touch some grass, and play a game.",
                        "Burnout is your brain's firewall kicking in. Listen to it and log off.",
                        "Take a guilt-free rest day. Productivity will wait for you."
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
                let recommendations = [
                    "🎮 Game Rec: Minecraft (Sandbox / Build & Relax)",
                    "🎮 Game Rec: Touhou Project (Bullet Hell / STG - time to dodge bullet hells instead of bugs)",
                    "🎮 Game Rec: Elden Ring (Action RPG / Exploration)",
                    "🎮 Game Rec: CS2 (Tactical Shooter / Competitive)",
                    "♟️ Mind Rec: Chess (Strategy / Mental Workout)",
                    "🛌 Meta Rec: Sleep (The Ultimate Recovery Meta)",
                    "🌿 Game Rec: Stardew Valley (Farming & Chill)",
                    "🗡️ Game Rec: Hollow Knight (Metroidvania Exploration)",
                    "🧗 Game Rec: Celeste (Precision Platformer)",
                ];

                let quests = [
                    "⚔️ Side Quest: Drink a full glass of water without spilling it on your keyboard.",
                    "⚔️ Side Quest: Go outside and look at the sky for at least 3 minutes.",
                    "⚔️ Side Quest: Stretch your back, neck, and wrists right now.",
                    "⚔️ Side Quest: Text a friend a random meme and close your IDE for 15 minutes.",
                ];

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as usize;

                let chosen_game = recommendations[now % recommendations.len()];
                let chosen_quest = quests[(now / 7) % quests.len()];

                (
                    format!(
                        "🎲 [ELIDE BRO! RECOVERY STATION]\n\n\
                         Listen up, fellow developer. Here is your prescription:\n\n\
                         - {}\n\
                         - {}\n\n\
                         Close the terminal, clear your head, and come back ready to beat Doom!",
                        chosen_game, chosen_quest
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
                    ("What do you tryna check it with, air?.".to_string(), false)
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
                (format!("tf is {}", cmd), false)
            }
            PaletteAction::Empty => (".Fym there's nothing".to_string(), true),
        }
    }
}