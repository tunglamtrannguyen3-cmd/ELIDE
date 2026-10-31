# 🥚 ELIDE — Easier Life @ IDE 🦆

<p align="center">
  <img src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" alt="ELIDE Duck Logo" width="220">
</p>

<p align="center">
  **Your code. Your tools. Your workflow.**
  <br>
  🦀 Built with Rust · ⚡ Terminal-native · 🤖 AI-integrated
  <br>
  <em>Less waiting. More control. More building.</em> 💜
</p>

---

## 👋 Meet ELIDE

**ELIDE (Easier Life @ IDE)** is a Rust-powered terminal IDE designed to bring essential development workflows into one focused environment.

Edit code, execute commands, explore your workspace, access language-server functionality, and use AI assistance—all from one terminal interface.

No unnecessarily complicated setup. No overwhelming interface. Just a development environment designed to get you building.

**Download. Launch. Create.** 🚀

## ✨ Features

- 📝 **Quick Editing** — Create, open, switch, and save files.
- 🌳 **Workspace Navigation** — Browse project files through the integrated workspace tree.
- ⚙️ **Command Execution** — Run shell commands and interactive programs.
- 🔍 **LSP Integration** — Access language-server workflows and diagnostics.
- 🤖 **AI Assistance** — Integrated AI chat and API-powered code auditing.
- 📋 **Clipboard Management** — Copy buffers, line ranges, terminal history, and diagnostics.
- 🧭 **Function Finder** — Locate Rust-style function declarations.
- 🎨 **ELIDE Serenity** — A cozy dark interface with lavender, sage, and misty-blue accents.
- 🎭 **Developer Wellness** — Mood-based suggestions, randomized responses, and recovery activities.
- 🦆 **Easter Eggs** — Because development doesn't always have to be serious.

## 🤖 AI-Powered Development

Your AI assistant belongs in your development environment.

ELIDE integrates AI functionality directly into its terminal workflow, allowing you to access AI chat and code auditing without constantly switching applications.

**Supported backend options:**
- Groq
- OpenAI
- Ollama
- ...
Features include:

- `Ctrl+A` — Open AI chat.
- `check <query>` — Run an AI-assisted check.
- Workspace-oriented AI auditing.

AI provider connectivity requires the appropriate API configuration. Keep your credentials private.

## 🎭 Developer Wellness — `--im-*`

Coding isn't just about writing code. It's also about the developer behind the keyboard.

ELIDE includes randomized mood-based suggestions for different situations.

| Command | Purpose |
|---|---|
| `--im-stuck` | 🔧 Debugging inspiration |
| `--im-bored` | 🎮 Coding challenges and entertainment |
| `--im-hungry` | 🍜 Food and hydration reminders |
| `--im-tired` | 😴 Rest and sleep reminders |
| `--im-stressed` | 🌿 Stress-relief suggestions |
| `--im-burntout` | 🔋 Burnout recovery |
| `--im-confused` | 🧠 Code comprehension tips |
| `--im-angry` | 🧊 Cooldown suggestions |
| `--im-lonely` | 💜 Encouragement and connection |

Every mode offers randomized responses to help make your development experience a little more human.

### 🦆 `-bro!` — Recovery Station

Need a break?

Run `-bro!` to receive:

- 🎮 A randomly selected game or mental activity.
- ⚔️ A randomly generated real-world side quest.

From Minecraft and Touhou to chess, stretching, and taking a walk, ELIDE reminds you that there's life outside the terminal.

**Your productivity matters. So does your well-being.**

## 🚀 Get Started

### Download

Get ELIDE from the official repository:

[**GitHub Repository — ELIDE**](https://github.com/tunglamtrannguyen3-cmd/ELIDE)

Download the appropriate release for your platform when available.

### Build from source

Requirements: Rust, Cargo, and a compatible terminal.

```bash
git clone https://github.com/tunglamtrannguyen3-cmd/ELIDE.git
cd ELIDE
cargo build --release
```

Run the executable produced by your checkout.

### Run tests

```bash
cargo test
```

## ⌨️ Command Reference

Open the command palette with `Alt+T`.

### File Management

| Command | Action |
|---|---|
| `save` / `:w` / `-s` | 💾 Save current file |
| `-sa` / `:wa` | 📚 Save all buffers |
| `-sl` / `:wq` | 💾 Save and quit |
| `-l` / `:q!` | 🚪 Quit without saving |
| `new <file>` | 📄 Create a file |
| `code <file>` | 📂 Open a file |
| `switch <file>` | 🔀 Switch files |
| `tree` / `:tree` | 🌳 Open workspace tree |

### Development Tools

| Command | Action |
|---|---|
| `sh <command>` | 🖥️ Execute shell command |
| `acel <command>` | ⚙️ Run interactive process |
| `lsp <command>` | 🔍 Start LSP workflow |
| `:funcs` | 🧭 Find functions |
| `search <term>` | 🔎 Search current buffer |

### Clipboard

| Command | Action |
|---|---|
| `copy` | 📋 Copy entire buffer |
| `copy 5-12` | 📋 Copy line range |
| `copy term` | 🖥️ Copy terminal history |
| `copy lsp` | 🔍 Copy LSP diagnostics |
| `paste` | 📥 Paste clipboard contents |

### AI & Utilities

| Command | Action |
|---|---|
| `check <query>` | 🤖 AI-assisted check |
| `-i` / `--info` | ℹ️ Editor information |
| `-h` / `--help` | 📖 Command manual |
| `-bro!` | 🦆 Recovery Station |
| `--im-*` | 🎭 Developer Wellness |

### Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Alt+T` | Command palette |
| `Alt+A` | AI assistant |
| `Alt+L` | LSP diagnostics |
| `Alt+D` | Function/symbol modal |
| `Esc` | Close active pane |

*Commands and shortcuts may vary between versions. Consult built-in help.*

## 🎨 ELIDE Serenity

A terminal IDE doesn't have to look boring.

ELIDE Serenity features a distinctive dark palette built around:

- 💜 Soft lavender.
- 🌿 Muted sage.
- 🌫️ Misty blue.
- 🌙 Comfortable dark backgrounds.

A visual identity designed to make development feel focused and enjoyable.

## 🧰 Technology Stack

- 🦀 Rust
- 🖥️ Ratatui
- ⌨️ Crossterm
- ⚡ Tokio
- 🔍 Language Server Protocol integrations
- 📋 Clipboard integration

## 🧑‍💻 Built Independently

ELIDE is independently developed by a **13-year-old developer**.

From terminal interface design to systems programming, language-server workflows, and AI API integration, ELIDE is an ongoing independent engineering project.

One developer. One ambitious project. A whole lot of Rust.

And, naturally, one duck. 🦆

## 🤝 Contributing

Have an idea or found a bug?

Contributions and suggestions are welcome!

- Open issues for bugs and feature requests.
- Submit pull requests with improvements.
- Include reproduction steps when reporting problems.
- Never expose API keys or other sensitive credentials.

## 📦 Repository

[**github.com/tunglamtrannguyen3-cmd/ELIDE**](https://github.com/tunglamtrannguyen3-cmd/ELIDE)

## 📄 License

ELIDE uses the **Eggchese Open Project License (EOPL-2026.2)**.

See [LICENSE](LICENSE) for complete terms.

---

<p align="center">
  **🦆 ELIDE — Your workflow. Your control. Your IDE.**
  <br>
  <em>Less waiting. More control. More building. 🚀</em>
</p>
