# 🥚 ELIDE — Easier Life @ IDE 🦆

<p align="center">
  <img src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" alt="ELIDE Duck Logo" width="220">
</p>

<p align="center">
  <strong>Your code. Your tools. Your workflow.</strong>
  <br>
  🦀 Built with Rust · ⚡ Terminal-native · 🤖 AI-integrated
  <br>
  <em>"I-It's not like I want to make your coding life easier or anything... Baka!"</em> 💜
</p>

---

## 🌸 01. H-Hey! Welcome to ELIDE!

**ELIDE (Easier Life @ IDE)** is a Rust-powered terminal IDE designed to make your development workflow simpler, more comfortable, and less annoying.

It brings essential development tools together in one terminal-native environment.

Edit your code, manage Git repositories, communicate with language servers, execute commands, and even chat with AI.

All in one place!

W-Wait, what? You still switch between five different applications just to write a program?

...Never mind. I'll help you with that. It's not like I care or anything! 

**Download. Launch. Create.** ✨

## 💜 02. Features — N-Not bad, right?

| Feature | Description |
|---|---|
| 📝 Code Editor | Edit, create, open, and save your precious source files. |
| 🌳 Workspace Explorer | Navigate your project without getting lost. |
| 🔍 LSP Integration | Language-server communication and diagnostics. |
| 🤖 AI Assistant | Chat with AI and perform workspace-oriented code audits. |
| 🐙 Git Integration | Manage repositories without leaving ELIDE. |
| ⚙️ Process Execution | Run shell commands and interactive applications. |
| 📋 Clipboard | Copy and paste buffers, line ranges, logs, and diagnostics. |
| 🧭 Function Finder | Find function declarations in your code. |
| 🎨 Custom Themes | Eight themes to make your terminal look adorable. |
| 🎭 Developer Wellness | Suggestions when you're stuck, tired, or frustrated. |
| 🦆 Easter Eggs | H-Hey! Developers deserve some fun too! |

## 🤖 03. AI Assistant — I-I can help you!

Your AI assistant belongs in your development environment.

ELIDE integrates AI functionality directly into its workflow, so you don't have to keep switching windows whenever you need assistance.

### Supported providers

- Groq
- OpenAI
- Ollama
- Other compatible API endpoints

### Features

- Interactive AI chat.
- Workspace-aware code auditing.
- Configurable system prompts.
- Adjustable context limits.
- Custom API endpoints and models.

**Keyboard shortcut:** `Alt+A`

Configuration:

| Variable | Purpose |
|---|---|
| `AI_API_KEY` | Authentication token |
| `AI_API_URL` | API endpoint |
| `AI_MODEL` | Model identifier |
| `AI_SYSTEM_PROMPT` | Custom instructions |
| `AI_MAX_CONTEXT_CHARS` | Context size limit |

AI functionality requires a compatible provider and valid credentials.

A-And don't expose your API keys! Even I can't protect you from that kind of mistake!

## 🐙 04. Git Integration — Version control, obviously!

Managing Git repositories shouldn't be unnecessarily complicated.

ELIDE provides a beginner-friendly command interface for common Git operations.

| Command | Action |
|---|---|
| `git status` | Check repository status |
| `git new repo` | Initialize a repository |
| `git choose repo` | Select a repository |
| `git login` | Check GitHub CLI authentication |
| `git fetch` | Fetch remote updates |
| `git pull` | Pull remote changes |
| `git branch` | List branches |
| `git switch <branch>` | Switch branches |
| `git commit -m "message"` | Commit and attempt to push |

See? Version control isn't that scary.

...W-What? I wasn't worried about you!

## 🎭 05. Developer Wellness — Take care of yourself, dummy!

Coding isn't just about writing code.

You're a human being, not a CPU running at 100% utilization!

ELIDE includes randomized, situation-based suggestions to help developers navigate different working conditions.

### The `--im-*` commands

| Command | Purpose |
|---|---|
| `--im-stuck` | 🔧 Debugging inspiration |
| `--im-bored` | 🎮 Coding challenges and entertainment |
| `--im-hungry` | 🍜 Food and hydration reminders |
| `--im-tired` | 😴 Rest and sleep suggestions |
| `--im-stressed` | 🌿 Stress-relief suggestions |
| `--im-burntout` | 🔋 Recovery suggestions |
| `--im-confused` | 🧠 Code comprehension tips |
| `--im-angry` | 🧊 Cooldown suggestions |
| `--im-lonely` | 💜 Encouragement and connection |

Each mode provides randomized responses based on your selected situation.

### 🦆 ELIDE BRO! — Recovery Station

Feeling exhausted?

Run:

```bash
-bro!
```

You'll receive:

- 🎮 A randomly selected game or mental activity.
- ⚔️ A randomly generated real-world side quest.

Maybe play Minecraft. Maybe try Touhou. Maybe go outside and look at the sky.

Just... close your terminal for a moment, okay?

**Your productivity matters. But your well-being matters too.**

I-It's an order! 💢

## 🎨 06. Themes — Choose something pretty!

Who said terminal applications have to look boring?

ELIDE comes with eight built-in themes.

| Theme | Atmosphere |
|---|---|
| 💜 Serenity | Lavender, sage, and peaceful pastels |
| 🌸 Spring at the Shrine | Sakura and warm shrine colors |
| 🌃 Tokyo Night | Neon city atmosphere |
| 🪦 Graveyard of the Firefly | Quiet, nostalgic atmosphere |
| 🌊 Deep Ocean | Deep and peaceful blues |
| 🌲 Magic Forest | Magical greens and nature |
| 🌿 Swampy | Earthy greens and muted tones |
| 🍂 Harvest Time | Warm autumn colors |

Switch themes using:

```bash
theme Serenity
theme Tokyo Night
theme Deep Ocean
```

**ELIDE Serenity** is the default theme.

Its design revolves around comfortable dark backgrounds, lavender, muted sage, and misty blue.

P-Please don't spend three hours choosing a theme instead of coding...

Actually, that's relatable.

## 🌐 07. Language Support — So many languages!

ELIDE recognizes a broad range of programming and markup languages.

Some examples:

- **Systems:** Rust, C, C++, Zig, Go, Ada, Assembly, D, Nim.
- **Application development:** Java, Kotlin, Swift, C#, Scala.
- **Scripting:** Python, Ruby, Lua, PHP, Perl.
- **Web:** JavaScript, TypeScript, HTML, CSS, Svelte, Vue, Astro.
- **Functional:** Haskell, OCaml, Elixir, Erlang, Clojure, Gleam.
- **Data and configuration:** SQL, JSON, YAML, TOML, XML, Nix.
- **Graphics:** GLSL, HLSL, WGSL.

For language intelligence, ELIDE supports external Language Server Protocol workflows.

You'll need to install and configure your own compatible language servers.

I provide the environment. You bring your favorite tools.

F-Fair enough, right?

## ⌨️ 08. Command Reference

Open the command interface with `Alt+T`.

### Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Alt+T` | Toggle command interface |
| `Alt+A` | Toggle AI Assistant |
| `Alt+L` | Toggle LSP diagnostics |
| `Alt+D` | Open function/symbol interface |
| `Esc` | Close active pane |

### File management

| Command | Action |
|---|---|
| `save` / `:w` / `-s` | Save current file |
| `-sa` / `:wa` | Save all buffers |
| `-sl` / `:wq` | Save and quit |
| `-l` / `:q!` | Quit without saving |
| `new <file>` | Create a file |
| `code <file>` | Open a file |
| `switch <file>` | Switch files |
| `tree` / `:tree` | Open workspace explorer |

### Development tools

| Command | Action |
|---|---|
| `sh <command>` | Execute background shell command |
| `acel <command>` | Run interactive foreground process |
| `lsp <command>` | Initialize language server |
| `:funcs` | Find function declarations |
| `search <term>` | Search current buffer |

### Clipboard

| Command | Action |
|---|---|
| `copy` | Copy entire buffer |
| `copy 5-12` | Copy line range |
| `copy term` | Copy terminal history |
| `copy lsp` | Copy LSP diagnostics |
| `paste` | Paste clipboard contents |

### AI and utilities

| Command | Action |
|---|---|
| `check <query>` | AI-assisted check |
| `-i` / `--info` | Editor information |
| `-h` / `--help` | Command manual |
| `-bro!` | Recovery Station |
| `--im-*` | Developer Wellness |

*Commands and shortcuts may vary between releases. Consult built-in help.*

## 🧰 09. Technology Stack

Built with Rust, because performance matters!

| Technology | Purpose |
|---|---|
| Rust | Core application |
| Ratatui | Terminal interface |
| Crossterm | Terminal interaction |
| Tokio | Asynchronous runtime |
| Serde | Serialization |
| LSP / JSON-RPC | Language-server communication |
| Arboard | Clipboard integration |
| Reqwest | AI API communication |

## 🚀 10. Installation

### Download

Visit the official repository:

[**ELIDE — GitHub Repository**](https://github.com/tunglamtrannguyen3-cmd/ELIDE)

Download a suitable release for your platform when available.

### Build from source

Requirements:

- Rust toolchain
- Cargo
- Compatible terminal emulator

```bash
git clone https://github.com/tunglamtrannguyen3-cmd/ELIDE.git

cd ELIDE

cargo build --release
```

Run the executable produced by your build.

### Run tests

```bash
cargo test
```

There! You're ready.

W-Well, what are you waiting for? Go build something!

## 🧑‍💻 11. Development Philosophy

ELIDE follows five principles:

1. **Simplicity:** Development tools should be easy to approach.
2. **Flexibility:** Developers choose their own toolchains and providers.
3. **Control:** Your environment should adapt to your workflow.
4. **Accessibility:** Beginners and experienced developers should both feel welcome.
5. **Well-being:** Developers deserve breaks, too.

ELIDE isn't trying to become an unnecessarily complicated ecosystem.

It wants to be a focused, flexible development environment.

## 🦆 12. About the Developer

ELIDE is independently developed by a **13-year-old developer**.

From Rust programming and terminal interface design to LSP communication, Git integration, and AI functionality, this project is an ongoing independent engineering effort.

One developer.

One ambitious project.

A whole lot of Rust.

And one very important duck.

...D-Don't underestimate the duck!

## 🤝 13. Contributing

Found a bug? Have an idea?

Contributions and suggestions are welcome!

- Open issues for bugs and feature requests.
- Submit pull requests with improvements.
- Include reproduction steps for reported problems.
- Keep changes focused and understandable.
- Never expose API keys or sensitive credentials.

Even small contributions are appreciated!

## 📦 14. Repository

[**github.com/tunglamtrannguyen3-cmd/ELIDE**](https://github.com/tunglamtrannguyen3-cmd/ELIDE)

## 📄 15. License

ELIDE uses the **Eggchese Open Project License (EOPL-2026.2)**.

Please read [LICENSE](LICENSE) for the complete licensing terms.

---

<p align="center">
  <strong>🦆 ELIDE — Your workflow. Your control. Your IDE.</strong>
  <br>
  <em>"I-It's not like I want you to become a better developer... Baka!"</em>
  <br>
  💜 Less waiting. More control. More building. 🚀
</p>