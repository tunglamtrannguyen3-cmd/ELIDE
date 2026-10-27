// src/palette.rs
use crate::editor::Editor;
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
    Tree,
    New(String),
    Code(String),
    Switch(String),
    SetLsp(String),
    Acel(String),
    Copy(Option<String>),
    CopyTerm,
    CopyLsp,
    Paste,
    AiCheck(String),
    Search(String),
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
            "-bro!" | "bro" | "bro!" | "-bro" => return PaletteAction::Bro,
            ":funcs" => return PaletteAction::Funcs,
            "copy" | "\"+y" => return PaletteAction::Copy(None),
            "copy term" | "copy log" => return PaletteAction::CopyTerm,
            "copy lsp" => return PaletteAction::CopyLsp,
            "paste" | "\"+p" => return PaletteAction::Paste,
            "tree" | "workspace" | ":tree" | ":files" => return PaletteAction::Tree,
            _ => {}
        }

        // Prefix match commands (supporting classic + hacker aliases)
        if trimmed.starts_with(":w ") {
            PaletteAction::Save(Some(Self::extract_arg(trimmed, ":w ")))
        } else if trimmed.starts_with("-s ") {
            PaletteAction::Save(Some(Self::extract_arg(trimmed, "-s ")))
        } else if trimmed.starts_with("save ") {
            PaletteAction::Save(Some(Self::extract_arg(trimmed, "save ")))
        } else if trimmed.starts_with("search ") {
            PaletteAction::Search(Self::extract_arg(trimmed, "search "))
        } else if trimmed.starts_with("/ ") {
            PaletteAction::Search(Self::extract_arg(trimmed, "/ "))
        } else if let Some(stripped) = trimmed.strip_prefix("sh ").or_else(|| trimmed.strip_prefix("!# ")) {
            PaletteAction::RunBash(stripped.trim().to_string())
        } else if let Some(stripped) = trimmed.strip_prefix("acel ").or_else(|| trimmed.strip_prefix("/! ")) {
            PaletteAction::Acel(stripped.trim().trim_matches('"').trim_matches('\'').to_string())
        } else if trimmed.starts_with("copy term") || trimmed.starts_with("copy log") {
            PaletteAction::CopyTerm
        } else if trimmed.starts_with("copy lsp") {
            PaletteAction::CopyLsp
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
        } else if let Some(stripped) = trimmed.strip_prefix("check ") {
            PaletteAction::AiCheck(stripped.trim().to_string())
        } else {
            PaletteAction::UnknownCommand(trimmed.to_string())
        }
    }

    /// Helper to parse line ranges safely for copy operations
    fn parse_line_range(range_str: &str, max_lines: usize) -> Result<(usize, usize), String> {
        let range_str = range_str.trim();
        if range_str.is_empty() || max_lines == 0 {
            return Err("Editor buffer is empty.".to_string());
        }

        let parts: Vec<&str> = range_str.split('-').map(|s| s.trim()).collect();
        let (start, end) = match parts.len() {
            1 => {
                let n = parts[0].parse::<usize>().map_err(|_| "Error: Invalid line number.")?;
                (n, n)
            }
            2 => {
                let s = parts[0].parse::<usize>().map_err(|_| "Error: Invalid line numbers.")?;
                let e = parts[1].parse::<usize>().map_err(|_| "Error: Invalid line numbers.")?;
                (s, e)
            }
            _ => return Err("Error: Invalid range format. Use format a-b or single line (e.g. copy 5)".to_string()),
        };

        if start == 0 || end == 0 {
            return Err("Error: Line numbers start at 1 (no line 0).".to_string());
        }
        if start > max_lines || end > max_lines {
            return Err(format!("Error: Line range out of bounds (file has only {} lines).", max_lines));
        }
        if start > end {
            return Err("Error: Start line cannot be greater than end line.".to_string());
        }

        Ok((start - 1, end - 1))
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
            PaletteAction::Tree => {
                ("Tree view opened.".to_string(), true)
            }
            PaletteAction::Search(query) => {
                if query.is_empty() {
                    ("Error: Specify a search term (e.g. search keyword or / keyword).".to_string(), false)
                } else {
                    let mut found_row = None;
                    let start_row = editor.cursor.row;
                    let total_lines = editor.lines.len();
                    
                    for i in 0..total_lines {
                        let check_row = (start_row + i) % total_lines;
                        if editor.lines[check_row].contains(&query) {
                            found_row = Some(check_row);
                            break;
                        }
                    }

                    match found_row {
                        Some(row) => {
                            editor.cursor.row = row;
                            editor.cursor.col = 0;
                            (format!("Found '{}' at line {}", query, row + 1), true)
                        }
                        None => (format!("String '{}' not found in buffer.", query), false),
                    }
                }
            }
            PaletteAction::SaveAll 
            | PaletteAction::Quit 
            | PaletteAction::SaveAndQuit 
            | PaletteAction::CopyTerm 
            | PaletteAction::CopyLsp => {
                ("Handled".to_string(), true)
            }
            PaletteAction::Copy(range_opt) => {
                let text_to_copy = match range_opt {
                    None => editor.lines.join("\n"),
                    Some(range_str) => {
                        match Self::parse_line_range(&range_str, editor.lines.len()) {
                            Ok((start_idx, end_idx)) => editor.lines[start_idx..=end_idx].join("\n"),
                            Err(err_msg) => return (err_msg, false),
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
                     - Version       : v2.3.1\n\
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
                "📖 [ELIDE v2.3.1 COMMAND PALETTE MANUAL]\n\n\
                 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                 ⌨️ QUICK HOTKEYS:\n\
                 - Alt+T                        - Toggle terminal / command palette\n\
                 - Alt+A                        - Toggle AI Chatbot Assistant\n\
                 - Alt+L                        - Toggle LSP diagnostics pane\n\
                 - Alt+D                        - Toggle interactive function/symbol modal\n\n\
                 📂 FILE & WORKSPACE NAVIGATION:\n\
                 - tree / :tree / workspace     - Open workspace file tree browser\n\
                 - new <file>                   - Create and open a new buffer file\n\
                 - code <file> / switch <file>  - Switch to or open an existing file\n\n\
                 💾 FILE SAVING & EXITING:\n\
                 - :w / save / -s [file]        - Save current buffer to disk\n\
                 - wa / :wa / -sa               - Save all open buffers in cache\n\
                 - :wq / wq / -sl               - Save all buffers and quit editor\n\
                 - :q! / q! / -l                - Quit editor without saving changes\n\n\
                 🔍 SEARCH & CODE EXPLORATION:\n\
                 - search <term> / / <term>     - Nano-style word search in buffer\n\
                 - :funcs                       - List all function signatures in buffer\n\n\
                 📋 CLIPBOARD MANAGEMENT:\n\
                 - copy [a-b]                   - Copy entire buffer or line range a-b\n\
                 - copy term / copy log         - Copy full terminal/command history\n\
                 - copy lsp                     - Copy active LSP diagnostics\n\
                 - paste / \"+p                 - Paste from system clipboard\n\n\
                 🤖 AI AUDIT & PROCESSES:\n\
                 - check <token> <url> [model]  - Connect AI backend (Groq/OpenAI/Ollama)\n\
                 - check <query>                - Run workspace AI audit or query check\n\
                 - lsp <cmd>                    - Initialize Language Server Protocol process\n\
                 - sh <cmd> / !# <cmd>          - Execute background Bash shell command\n\
                 - acel <cmd> / /! <cmd>        - Run interactive foreground process\n\n\
                 ☕ WELLNESS & RECOVERY (FULL --im CMDS):\n\
                 - -bro! / bro                  - Open developer recovery & side quest station\n\
                 - --im-stuck                   - Get unstuck debugging advice\n\
                 - --im-bored                   - Get fun coding challenges & game ideas\n\
                 - --im-hungry                  - Food break reminders\n\
                 - --im-tired                   - Sleep & rest optimization tips\n\
                 - --im-stressed                - Breathing & anxiety relief steps\n\
                 - --im-burntout                - Mental health & recharge guide\n\
                 - --im-confused                - Rubber duck refactoring tips\n\
                 - --im-angry                   - Cool down advice for rage bugs\n\
                 - --im-lonely                  - Friendly developer motivation"
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
                        "Delete the whole function and rewrite it. Sometimes starting clean is faster than fixing spaghetti.",
                        "If it compiles, it's a feature. If it crashes, it's a learning opportunity.",
                        "Check your git diff. It worked 10 minutes ago, what did you ruin?",
                        "Comment out the entire file until it compiles, then slowly uncomment your sins."
                    ],
                    "--im-bored" => vec![
                        "Type -bro! for a game recommendation, or rewrite this editor in Brainfuck.",
                        "Try optimizing your loops until your CPU starts screaming.",
                        "Go refactor that one messy file you've been avoiding for three weeks.",
                        "Challenge yourself to write a game in 10 lines of Bash.",
                        "Go to GitHub and find a typosquatting issue to fix for easy green squares.",
                        "Run cargo bench and stare at the numbers like a day trader.",
                        "Open a markdown file and start drafting your sci-fi novel about AI taking over your job."
                    ],
                    "--im-hungry" => vec![
                        "Go eat! Your code won't compile better on an empty stomach.",
                        "Instant ramen is calling your name. Go fuel up.",
                        "Step away from the screen and grab some water and a snack.",
                        "A well-fed programmer finds bugs twice as fast.",
                        "The human brain consumes 20% of your metabolic energy. Go replenish the cache.",
                        "DoorDash something nutritious. Caffeine and pure spite can only carry you so far.",
                        "Your blood sugar level is lower than your code coverage. Go eat."
                    ],
                    "--im-tired" => vec![
                        "Git commit, git push, go to bed. The bug will be there tomorrow.",
                        "Sleep is the ultimate compiler optimization for your brain.",
                        "Close your laptop. Nothing good gets coded past 2 AM.",
                        "Rest now, conquer the segmentation fault tomorrow.",
                        "You've been staring at a typo for 45 minutes. Go to sleep, you'll see it instantly in the morning.",
                        "Your brain is currently experiencing high latency. Power down safely.",
                        "Close your eyes for 20 minutes. It's called a micro-nap, not a crash."
                    ],
                    "--im-stressed" => vec![
                        "Take 3 deep breaths. Drop your shoulders away from your ears.",
                        "Remember: it's just code. No one's life is on the line right now.",
                        "Step away from the keyboard, stretch your arms, and drink a glass of water.",
                        "Break the problem down into the tiniest possible piece. Solve just that one micro-step.",
                        "Your worth as a human being is completely decoupled from your production uptime.",
                        "The architecture isn't broken, it's just 'evolving unexpectedly'. Take a breath.",
                        "In the grand scheme of the universe, this merge conflict does not exist."
                    ],
                    "--im-burntout" => vec![
                        "You've been grinding hard. Close ELIDE, touch some grass, and play a game.",
                        "Burnout is your brain's firewall kicking in. Listen to it and log off.",
                        "Take a guilt-free rest day. Productivity will wait for you.",
                        "The terminal will still be black and text will still be colorful tomorrow. Go disconnect completely.",
                        "Cancel your meetings, silence your Slack, and go do absolutely nothing for an afternoon.",
                        "Your mental heap is fragmented. You need a total cold reboot, not a garbage collection cycle."
                    ],
                    "--im-confused" => vec![
                        "Confusion is just understanding on the horizon. Take a step back and map out the data flow on paper.",
                        "Trace your variables one by one. You've got this!",
                        "Draw it out on a physical whiteboard or piece of paper. Abstract code becomes real when it's messy scribbles.",
                        "If the abstractions are too deep, you've over-engineered it. Strip it back down to a basic loop.",
                        "The library authors were human too. Read their source code; they probably didn't know what they were doing either."
                    ],
                    "--im-angry" => vec![
                        "Step away from the keyboard before punch-testing your monitor. Go drink ice water.",
                        "Breathe out slowly. Rage-coding introduces 5 new bugs for every 1 fixed.",
                        "Do not run git push --force right now. Walk away.",
                        "The computer is an inanimate object operating on flawless logic. It's not trying to hurt you, it's just dumb.",
                        "Go scream into a pillow or smash a boxing bag. Then come back and fix the missing semicolon."
                    ],
                    "--im-lonely" => vec![
                        "Open up AI chat (Alt+A) or go grab a coffee with a fellow dev. You're part of an awesome community!",
                        "Even solo devs have ELIDE and your coffee mug cheering for you.",
                        "Pop into a Discord server or IRC channel and help a beginner. It feels good to be the expert.",
                        "You are connected to a global network of millions of sleep-deprived individuals doing the exact same thing."
                    ],
                    _ => vec![
                        "I am an editor, not a therapist. But you got this!",
                        "Keep pushing forward, you're closer than you think.",
                        "Every expert was once a beginner staring at a blank screen.",
                        "Command unrecognized, but I believe in you anyway.",
                        "Error 404: Mood not found. Here is a generic token of emotional validation instead."
                    ],
                };

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as usize;
                
                let chosen_response = responses[now % responses.len()].to_string();

                (chosen_response, true)
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
                    "🎮 Game Rec: Factorio (The factory must grow, but your codebase can wait)",
                    "🎮 Game Rec: Hades (Rogue-like / Escape your current debugging hell)",
                    "🎮 Game Rec: Cyberpunk 2077 (To remind yourself that your bugs aren't actually that bad)",
                    "🎮 Game Rec: Doom Eternal (Rip and tear through your frustration)",
                    "🎮 Game Rec: Animal Crossing (Maximum chill, zero compiler errors)",
                    "🏎️ Game Rec: Trackmania (Speedrunning short tracks to reset your brain waves)",
                    "🧱 Game Rec: Tetris Effect (Visual audio bliss to clean your mental cache)",
                    "🎮 Game Rec: RimWorld (Manage a colony instead of managing memory leaks)",
                    "🧠 Mind Rec: Read a physical book (Zero screen-glare, infinite immersion)",
                ];

                let quests = [
                    "⚔️ Side Quest: Drink a full glass of water without spilling it on your keyboard.",
                    "⚔️ Side Quest: Go outside and look at the sky for at least 3 minutes.",
                    "⚔️ Side Quest: Stretch your back, neck, and wrists right now.",
                    "⚔️ Side Quest: Text a friend a random meme and close your IDE for 15 minutes.",
                    "⚔️ Side Quest: Do 10 push-ups or 15 air squats to pump blood back into your brain.",
                    "⚔️ Side Quest: Fix your posture. Uncross your legs and drop your shoulders.",
                    "⚔️ Side Quest: Go look at a plant or something green that isn't a successful build terminal.",
                    "⚔️ Side Quest: Clean three random items off your desk right now.",
                    "⚔️ Side Quest: Close all 47 browser tabs that you know deep down you aren't going to read.",
                    "⚔️ Side Quest: Wash your hands and face with cold water to force a quick physical reset.",
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
            PaletteAction::New(_filename) => ("Handled".to_string(), true),
            PaletteAction::Code(_filename) | PaletteAction::Switch(_filename) => ("Handled".to_string(), true),
            PaletteAction::SetLsp(cmd) => {
                if cmd.is_empty() {
                    ("What do you tryna check it with, air?.".to_string(), false)
                } else {
                    (format!("Starting LSP: {}", cmd), true)
                }
            }
            PaletteAction::Acel(cmd) => {
                if cmd.is_empty() {
                    ("Error: Specify a command to run (e.g., acel build or /! build).".to_string(), false)
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