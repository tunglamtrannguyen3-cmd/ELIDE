// src/palette.rs
use crate::colors::Theme;
use crate::editor::{Editor, SystemClipboard};
use crate::git::GitManager;
use crate::process;

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
    ViewRepoTree,
    New(String),
    Code(String),
    Switch(String),
    SetLsp(String),
    Acel(String),
    Copy(Option<String>),
    CopyTerm,
    CopyLsp,
    Paste,
    AiCommand(String),
    AiCheck(String),
    Search(String),
    Theme(String),
    // Git Integration Actions
    GitStatus,
    GitInitRepo,
    GitChooseRepo(Option<String>),
    GitLogin,
    GitCommit { message: String, branch: String },
    GitPull { branch: Option<String>, rebase: bool },
    GitFetch,
    GitBranchList,
    GitSwitchBranch(String),
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
            "-bro!" | "bro" | "-bro" => return PaletteAction::Bro,
            ":funcs" => return PaletteAction::Funcs,
            "copy" | "\"+y" => return PaletteAction::Copy(None),
            "copy term" | "copy log" => return PaletteAction::CopyTerm,
            "copy lsp" => return PaletteAction::CopyLsp,
            "paste" | "\"+p" => return PaletteAction::Paste,
            "tree" | "workspace" | ":tree" | ":files" => return PaletteAction::Tree,
            "tree --view-repo" => return PaletteAction::ViewRepoTree,
            _ => {}
        }

        // Beginner-friendly Git command parsing (supports optional colon like :git status)
        let git_payload = trimmed
            .strip_prefix(":git ")
            .or_else(|| trimmed.strip_prefix("git "));

        if let Some(git_cmd) = git_payload {
            let tokens = parse_cli_tokens(git_cmd.trim());
            let subcommand = tokens.first().map(|s| s.as_str()).unwrap_or("");

            return match subcommand {
                "status" => PaletteAction::GitStatus,
                "branch" => PaletteAction::GitBranchList,
                "switch" | "checkout" => {
                    let branch_name = tokens.get(1).cloned().unwrap_or_default();
                    if branch_name.is_empty() {
                        PaletteAction::UnknownCommand(trimmed.to_string())
                    } else {
                        PaletteAction::GitSwitchBranch(branch_name)
                    }
                }
                "new" => {
                    if tokens.get(1).map(|s| s.as_str()) == Some("repo") {
                        PaletteAction::GitInitRepo
                    } else {
                        PaletteAction::UnknownCommand(trimmed.to_string())
                    }
                }
                "choose" => {
                    if tokens.get(1).map(|s| s.as_str()) == Some("repo") {
                        let repo_name = tokens.get(2).cloned();
                        PaletteAction::GitChooseRepo(repo_name)
                    } else {
                        PaletteAction::UnknownCommand(trimmed.to_string())
                    }
                }
                "login" => PaletteAction::GitLogin,
                "fetch" => PaletteAction::GitFetch,
                "pull" => {
                    let mut rebase = false;
                    let mut branch = None;
                    let mut i = 1;
                    while i < tokens.len() {
                        let token = tokens[i].as_str();
                        if token == "--rebase" || token == "-r" {
                            rebase = true;
                        } else if !token.starts_with('-') && branch.is_none() {
                            branch = Some(tokens[i].clone());
                        }
                        i += 1;
                    }
                    PaletteAction::GitPull { branch, rebase }
                }
                "commit" => {
                    let mut msg = String::new();
                    let mut branch = String::new();
                    let mut i = 1;
                    while i < tokens.len() {
                        match tokens[i].as_str() {
                            "-tags" | "-m" | "--message" => {
                                if let Some(val) = tokens.get(i + 1) {
                                    msg = val.clone();
                                    i += 1;
                                }
                            }
                            "branch" | "-b" | "--branch" => {
                                if let Some(val) = tokens.get(i + 1) {
                                    branch = val.clone();
                                    i += 1;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    PaletteAction::GitCommit { message: msg, branch }
                }
                _ => PaletteAction::UnknownCommand(trimmed.to_string()),
            };
        }

        // Prefix match commands (AI settings, themes, files, and utilities)
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
        } else if trimmed.starts_with("connect:") || trimmed.starts_with("connect ")
            || trimmed.starts_with("set-sysprompt") || trimmed.starts_with("set-max-lim")
            || trimmed.starts_with("prompt:") || trimmed.starts_with("limit:") {
            PaletteAction::AiCommand(trimmed.to_string())
        } else if trimmed.starts_with("theme:") || trimmed.starts_with("colorscheme:") {
            let prefix = if trimmed.starts_with("theme:") { "theme:" } else { "colorscheme:" };
            PaletteAction::Theme(Self::extract_arg(trimmed, prefix))
        } else if trimmed.starts_with("theme ") || trimmed.starts_with("colorscheme ") {
            let prefix = if trimmed.starts_with("theme ") { "theme " } else { "colorscheme " };
            PaletteAction::Theme(Self::extract_arg(trimmed, prefix))
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
            PaletteAction::ViewRepoTree => {
                if !GitManager::is_git_repo() {
                    return ("Error: Not a git repository. Cannot display repository tree.".to_string(), false);
                }

                let tracked = std::process::Command::new("git")
                    .args(["ls-files"])
                    .output();
                let untracked = std::process::Command::new("git")
                    .args(["ls-files", "--others", "--exclude-standard"])
                    .output();

                match (tracked, untracked) {
                    (Ok(t), Ok(u)) if t.status.success() && u.status.success() => {
                        let mut all_files: Vec<(String, bool)> = Vec::new();

                        let tracked_str = String::from_utf8_lossy(&t.stdout);
                        for line in tracked_str.lines() {
                            if !line.trim().is_empty() {
                                all_files.push((line.trim().to_string(), false));
                            }
                        }

                        let untracked_str = String::from_utf8_lossy(&u.stdout);
                        for line in untracked_str.lines() {
                            if !line.trim().is_empty() {
                                all_files.push((line.trim().to_string(), true));
                            }
                        }

                        if all_files.is_empty() {
                            ("📁 Repository tree is empty.".to_string(), true)
                        } else {
                            all_files.sort_by(|a, b| a.0.cmp(&b.0));

                            let mut formatted_tree = Vec::new();
                            formatted_tree.push("📁 [REPOSITORY TREE]".to_string());

                            for (path_str, is_untracked) in all_files {
                                let parts: Vec<&str> = path_str.split('/').collect();
                                let depth = parts.len().saturating_sub(1);
                                let indent = "  ".repeat(depth + 1);
                                let file_name = parts.last().unwrap_or(&"");
                                
                                let tag = if is_untracked { " (untracked)" } else { "" };
                                formatted_tree.push(format!("{}📄 {}{}", indent, file_name, tag));
                            }

                            (formatted_tree.join("\n"), true)
                        }
                    }
                    _ => {
                        ("Error: Failed to fetch repository files.".to_string(), false)
                    }
                }
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
            PaletteAction::AiCommand(payload) => {
                match crate::ai::try_handle_command(&payload) {
                    Ok(Some(msg)) => (msg, true),
                    Ok(None) => ("AI command processed successfully.".to_string(), true),
                    Err(err) => (err, false),
                }
            }
            PaletteAction::Theme(theme_name) => {
                if theme_name.is_empty() {
                    ("Error: Specify a theme name (e.g. theme catppuccin or colorscheme dracula).".to_string(), false)
                } else {
                    let new_theme = Theme::from_input(&theme_name);
                    editor.current_theme = new_theme;
                    (format!("🎨 Theme updated successfully to '{}'!", theme_name), true)
                }
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
                    Ok(_) => ("Copied! (Synced to System Clipboard & Internal Buffer)".to_string(), true),
                    Err(e) => (format!("Copied to internal buffer, but system clipboard sync warned: {}", e), false),
                }
            }
            PaletteAction::Paste => {
                let mut clip_mgr = SystemClipboard::new();
                let clip_text = match clip_mgr.get_text() {
                    text if !text.is_empty() => text,
                    _ => clipboard.clone(),
                };

                if clip_text.is_empty() {
                    return ("Clipboard is empty.".to_string(), false);
                }

                let clip_lines: Vec<String> = clip_text.split('\n').map(|s| s.to_string()).collect();
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
                ("Pasted successfully at cursor position.".to_string(), true)
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
- Favorite Idol : Yatsuzume, All Minecraft Manhunt Speedrunners\n\
- Skills        : Systems Programming, Rust, C, Ada, Micro-skills, Cooking, Chess, Touhou on Lunatic\n\
                     -----------------------------------\n\
                     ℹ️ [EDITOR INFO]\n\
- Version       : v2.4.0\n\
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
                "📖 [ELIDE v2.4.0 COMMAND PALETTE MANUAL]\n\n\
                 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                 ⌨️ QUICK HOTKEYS:\n\
- Alt+T                        - Toggle terminal / command palette\n\
- Alt+A                        - Toggle AI Chatbot Assistant\n\
- Alt+L                        - Toggle LSP diagnostics pane\n\
- Alt+D                        - Toggle interactive function/symbol modal\n\n\
                 📂 FILE & WORKSPACE NAVIGATION:\n\
- tree / :tree / workspace     - Open workspace file tree browser\n\
- tree --view-repo             - View and inspect repository structure\n\
- new <file>                   - Create and open a new buffer file\n\
- code <file> / switch <file>  - Switch to or open an existing file\n\n\
                 💾 FILE SAVING & EXITING:\n\
- :w / save / -s [file]        - Save current buffer to disk\n\
- wa / :wa / -sa               - Save all open buffers in cache\n\
- :wq / wq / -sl               - Save all buffers and quit editor\n\
- :q! / q! / -l                - Quit editor without saving changes\n\n\
                 🐙 BEGINNER-FRIENDLY GIT INTEGRATION:\n\
- git status / :git status     - Check git status of workspace\n\
- git new repo                 - Initialize a new git repository\n\
- git choose repo [name]       - Scan subdirectories or choose specific repo\n\
- git login                    - Check GitHub CLI authentication status\n\
- git fetch                    - Fetch remote tracking branches\n\
- git pull [-r/--rebase] [br]  - Pull updates with rebase option\n\
- git branch                   - List all local & remote branches\n\
- git switch <branch>          - Switch active git branch\n\
- git commit -m \"msg\"          - Stage, commit, and push with fallback\n\n\
                 🎨 THEME & APPEARANCE:\n\
- theme <name> / colorscheme <name> - Switch Neovim-style theme/colorscheme\n\n\
                 🔍 SEARCH & CODE EXPLORATION:\n\
- search <term> / / <term>     - Nano-style word search in buffer\n\
- :funcs                       - List all function signatures in buffer\n\n\
                 📋 CLIPBOARD MANAGEMENT:\n\
- copy [a-b]                   - Copy entire buffer or line range a-b\n\
- copy term / copy log         - Copy full terminal/command history\n\
- copy lsp                     - Copy active LSP diagnostics\n\
- paste / \"+p                 - Paste from system/internal clipboard\n\n\
                 🤖 AI AUDIT & PROCESSES:\n\
- connect: <token> <url> [model] - Connect AI backend\n\
- set-sysprompt <prompt>         - Change AI system prompt\n\
- set-max-lim <chars>            - Change context character limit\n\
- lsp <cmd>                      - Initialize Language Server Protocol process\n\
- sh <cmd> / !# <cmd>            - Execute background Bash shell command\n\
- acel <cmd> / /! <cmd>          - Run interactive foreground process\n\n\
                 ☕ WELLNESS & RECOVERY:\n\
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
            "S-Stuck again? Explain the problem to a rubber duck. It might actually work!",
            "Start with the error message. The compiler is trying to tell you something!",
            "Reduce the problem to a minimal reproducible example. Less code, fewer mysteries.",
            "Check your assumptions, inspect the logs, and trace the data flow.",
            "Try taking a short break. Your brain might find the answer after a restart.",
            "Still stuck? Ask ELIDE AI. That's what it's here for, baka!",
            "Don't rewrite the entire project just because one function misbehaves!",
        ],

        "--im-bored" => vec![
            "Bored? Build something ridiculous. A calculator for ducks, perhaps?",
            "Try a programming language you've never used. Ada? Rust? Something unusual?",
            "Find that messy file you've been avoiding. I know you know which one.",
            "Optimize something. But please don't optimize code that doesn't need it!",
            "Type -bro! for a game recommendation. Not that I prepared one or anything.",
            "Try making a tiny game. A weekend project sounds interesting!",
            "Challenge yourself to implement a small utility without external dependencies.",
        ],

        "--im-hungry" => vec![
            "FF Soup (4 servings): Make broth using Costco rotisserie chicken bones. Strain out every bone fragment, then add leftover chicken, 1/3 can Spam, dried vegetables, water, and seasonings. Stir in 4 beaten eggs to make egg ribbons. A warm meal with leftovers!",

            "La Económica: Spread ketchup over tortillas, add 2/3 can Spam and your homemade cheese. Heat with a little margarine until the tortilla is crisp and the cheese melts. Budget pizza, maximum flavour!",

            "Resource management tip: Use 1/3 can Spam for FF Soup and 2/3 can for La Económica. One can, two recipes. Efficient, isn't it?",

            "Got leftover rotisserie chicken? Save the meat for FF Soup and use the bones to make broth. Waste less, eat better!",

            "Budget cooking rule: Reuse ingredients across multiple meals. Your wallet deserves good architecture too.",

            "La Económica is proof that a pizza doesn't need expensive ingredients to be delicious. ¡Buen provecho!",

            "Need inspiration? Check your fridge before buying more ingredients. Leftovers are underrated!",
        ],

        "--im-tired" => vec![
            "Save your work, commit your changes, and call it a day. That's an order!",
            "Your brain isn't a server with infinite uptime. Schedule some maintenance.",
            "The bug will still be there tomorrow. Unfortunately.",
            "You don't need to finish every feature today. Rest matters too.",
            "Close the editor when you need to. Your project isn't going anywhere.",
            "Even Rust needs a break from borrowing resources. ...That's a joke. Mostly.",
        ],

        "--im-stressed" => vec![
            "Hey. Pause for a moment. Take a few slow breaths.",
            "One problem at a time. You don't have to defeat the entire codebase today.",
            "Save your progress and step away from the screen for a little while.",
            "A difficult project doesn't make you a bad developer. Everyone struggles sometimes.",
            "Try writing down what's bothering you. Make the problems smaller and more manageable.",
            "You are allowed to take a break. That's not failure, baka!",
        ],

        "--im-burntout" => vec![
            "You've been working hard. Give yourself permission to stop for today.",
            "No commits, no deadlines, no optimization. Just take a proper break.",
            "You don't have to be productive every second. Rest is part of the process!",
            "Your well-being matters more than another feature. Seriously.",
            "Step away from the project and spend some time doing something you enjoy.",
            "A project is a marathon, not a competition to see who can exhaust themselves first.",
        ],

        "--im-confused" => vec![
            "Let's untangle this. Identify the inputs, follow the data, and inspect the outputs.",
            "Draw the logic on paper. A diagram can be worth a thousand lines of code.",
            "Break the problem into smaller pieces. You don't need to understand everything at once.",
            "Check the documentation. There might be an explanation hiding in plain sight.",
            "Ask ELIDE AI to explain the concept in simpler terms. That's what it's for!",
            "Confusion is not failure. It's a sign that there's something new to understand.",
        ],

        "--im-angry" => vec![
            "Keyboard down. Take a breath. We're debugging software, not fighting a final boss.",
            "Save your work before doing anything else. Future you will appreciate it.",
            "Step away for a moment. Debugging while frustrated rarely helps.",
            "One stubborn bug doesn't invalidate all the progress you've made.",
            "Maybe the compiler isn't your enemy. ...Maybe.",
            "Take a break, get some water, and return when you're ready.",
        ],

        "--im-lonely" => vec![
            "Maybe message a friend or talk to someone you trust. You don't have to spend every moment alone.",
            "There are developers out there who share your interests. You could find a community!",
            "Take a break from coding and spend some time with someone you enjoy being around.",
            "You can open ELIDE AI with Alt+A to discuss an idea. And don't forget about real people, okay?",
            "Share something you've built with someone. Sometimes a little conversation makes a big difference.",
            "You don't need to be a developer every minute of the day. Just be yourself.",
        ],

        _ => vec![
            "I'm an IDE, not a miracle worker! ...But I'll still try to help.",
            "Every developer has rough days. Keep going at your own pace!",
            "Save your work. Drink some water. And please stop fighting the compiler.",
            "You've got this! N-Not that I was worried or anything.",
            "Remember to take care of yourself alongside your projects.",
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
        // 🎮 Games
        "🎮 Game Rec: Minecraft (Sandbox / Build & Relax)",
        "🎮 Game Rec: Touhou Project (Bullet Hell / STG - dodge bullets instead of bugs)",
        "🎮 Game Rec: Terraria (Sandbox / Adventure / Exploration)",
        "🎮 Game Rec: Stardew Valley (Farming / Relaxation)",
        "🎮 Game Rec: Celeste (Platformer / Precision)",
        "🎮 Game Rec: Portal 2 (Puzzle / Problem Solving)",
        "🎮 Game Rec: OpenTTD (Transport Simulation / Strategy)",
        "🎮 Game Rec: SuperTuxKart (Racing / Casual)",
        "🎮 Game Rec: The Battle for Wesnoth (Turn-Based Strategy)",
        "🎮 Game Rec: Slay the Spire (Deckbuilding / Strategy)",
        "🎮 Game Rec: Undertale (RPG / Story)",
        "🎮 Game Rec: osu! (Rhythm / Reflexes)",
        "🎮 Game Rec: Factorio (Automation / Engineering)",
        "🎮 Game Rec: Don't Starve (Survival / Exploration)",
        "🎮 Game Rec: Chess (Strategy / Mind Workout)",

        // 🧠 Non-game recommendations
        "🧠 Mind Rec: Solve a small puzzle or riddle.",
        "🎨 Creative Rec: Draw something without worrying about perfection.",
        "🎵 Music Rec: Listen to an album without multitasking.",
        "📚 Learning Rec: Explore a topic you've always been curious about.",
        "🛌 Meta Rec: Sleep (The Ultimate Recovery Meta)",
    ];

    let quests = [
        // 💧 Basic maintenance
        "⚔️ Side Quest: Drink a glass of water.",
        "⚔️ Side Quest: Stretch your back, neck, and wrists gently.",
        "⚔️ Side Quest: Rest your eyes by looking away from the screen.",
        "⚔️ Side Quest: Take a few slow breaths and relax your shoulders.",

        // 🌱 Outside world
        "⚔️ Side Quest: Look at the sky for at least 3 minutes.",
        "⚔️ Side Quest: Get some fresh air if it's safe and convenient.",
        "⚔️ Side Quest: Find something interesting in nature and observe it.",
        "⚔️ Side Quest: Take a short walk around your home.",

        // 🧹 Environment
        "⚔️ Side Quest: Clear some space on your desk.",
        "⚔️ Side Quest: Organize one folder on your computer.",
        "⚔️ Side Quest: Put away three things that are out of place.",
        "⚔️ Side Quest: Clean your keyboard gently and safely.",

        // 🎨 Creative and social
        "⚔️ Side Quest: Sketch a character, creature, or silly duck.",
        "⚔️ Side Quest: Listen to one song you haven't heard in a while.",
        "⚔️ Side Quest: Send a friend a message and see how they're doing.",
        "⚔️ Side Quest: Write down one idea for your next project.",

        // 💻 Developer quests
        "⚔️ Side Quest: Close unnecessary browser tabs.",
        "⚔️ Side Quest: Write one TODO for your next coding session.",
        "⚔️ Side Quest: Read one page of documentation just for curiosity.",
        "⚔️ Side Quest: Find one thing in your project that you're proud of.",
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
             Hey, fellow developer! Your random recommendations have arrived:\n\n\
             - {}\n\
             - {}\n\n\
             No deadlines, no pressure. Pick something fun and take a breather!\n\
             ELIDE will be here when you're ready.",
            chosen_game,
            chosen_quest
        ),
        true,
    )
}

            PaletteAction::RunBash(cmd) => match process::run_bash_cmd(&cmd).await {
                Ok(res) => (res.output, res.success),
                Err(e) => (format!("Bash Error: {}", e), false),
            },
            PaletteAction::SaveAll 
            | PaletteAction::Quit 
            | PaletteAction::SaveAndQuit => {
                ("Handled".to_string(), true)
            }
            PaletteAction::CopyTerm | PaletteAction::CopyLsp => {
                ("Handled".to_string(), true)
            }
            PaletteAction::New(_filename) => ("Handled".to_string(), true),
            PaletteAction::Code(_filename) | PaletteAction::Switch(_filename) => ("Handled".to_string(), true),
            PaletteAction::SetLsp(cmd) => {
                if cmd.is_empty() {
                    ("What do you tryna check it with, air?".to_string(), false)
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
                    ("Error: Specify a query or use 'connect: <token> <url>'\nOr use set-sysprompt / set-max-lim to update AI options.".to_string(), false)
                } else {
                    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    match crate::ai::run_check(&root, &query).await {
                        Ok(res) => (format!("🤖 [ELIDE AI AUDIT]\n\n{}", res), true),
                        Err(e) => (format!("❌ AI Error: {}", e), false),
                    }
                }
            }
            // Git Action Handlers
            PaletteAction::GitStatus => match GitManager::status() {
                Ok(out) => (out, true),
                Err(e) => (format!("Git Error: {}", e), false),
            },
            PaletteAction::GitInitRepo => match GitManager::init_new_repo() {
                Ok(out) => (out, true),
                Err(e) => (format!("Git Error: {}", e), false),
            },
            PaletteAction::GitChooseRepo(name_opt) => match GitManager::choose_repo(name_opt.as_deref()) {
                Ok(out) => (out, true),
                Err(e) => (format!("Git Error: {}", e), false),
            },
            PaletteAction::GitLogin => match GitManager::login_check() {
                Ok(out) => (out, true),
                Err(e) => (format!("Git Error: {}", e), false),
            },
            PaletteAction::GitFetch => match GitManager::fetch() {
                Ok(out) => (out, true),
                Err(e) => (format!("Git Error: {}", e), false),
            },
            PaletteAction::GitPull { branch, rebase } => {
                match GitManager::pull(branch.as_deref(), rebase) {
                    Ok(out) => (out, true),
                    Err(e) => (format!("Git Error: {}", e), false),
                }
            }
            PaletteAction::GitBranchList => match GitManager::list_branches() {
                Ok(out) => (out, true),
                Err(e) => (format!("Git Error: {}", e), false),
            },
            PaletteAction::GitSwitchBranch(branch_name) => {
                match GitManager::switch_branch(&branch_name, false) {
                    Ok(out) => (out, true),
                    Err(e) => (format!("Git Error: {}", e), false),
                }
            }
            PaletteAction::GitCommit { message, branch } => {
                match GitManager::quick_save(&message, &branch, None) {
                    Ok(out) => (out, true),
                    Err(e) => (format!("Git Error: {}", e), false),
                }
            }
            PaletteAction::UnknownCommand(cmd) => {
                (format!("tf is {}", cmd), false)
            }
            PaletteAction::Empty => (".Fym there's nothing".to_string(), true),
        }
    }
}

/// Helper to split command strings while respecting quoted sentences and mixed quote delimiters
fn parse_cli_tokens(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut active_quote: Option<char> = None;

    for c in input.chars() {
        match c {
            '"' | '\'' => {
                if let Some(q) = active_quote {
                    if q == c {
                        active_quote = None;
                    } else {
                        current.push(c);
                    }
                } else {
                    active_quote = Some(c);
                }
            }
            ' ' | '\t' if active_quote.is_none() => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

// ==========================================
// UNIT TESTS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_empty() {
        let palette = Palette { input_buffer: "".to_string(), is_active: false };
        assert_eq!(palette.parse_command(None), PaletteAction::Empty);
    }

    #[test]
    fn test_parse_command_exact() {
        let palette = Palette { input_buffer: ":w".to_string(), is_active: false };
        assert_eq!(palette.parse_command(None), PaletteAction::Save(None));

        let palette_tree = Palette { input_buffer: "tree --view-repo".to_string(), is_active: false };
        assert_eq!(palette_tree.parse_command(None), PaletteAction::ViewRepoTree);
    }

    #[test]
    fn test_parse_command_git() {
        let palette = Palette { input_buffer: "git status".to_string(), is_active: false };
        assert_eq!(palette.parse_command(None), PaletteAction::GitStatus);
    }

    #[test]
    fn test_parse_command_theme() {
        let palette = Palette { input_buffer: "theme catppuccin".to_string(), is_active: false };
        assert_eq!(palette.parse_command(None), PaletteAction::Theme("catppuccin".to_string()));
    }

    #[test]
    fn test_parse_cli_tokens() {
        let tokens = parse_cli_tokens("git commit -m \"hello world\"");
        assert_eq!(tokens, vec!["git", "commit", "-m", "hello world"]);
    }
}