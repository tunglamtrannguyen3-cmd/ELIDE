# 🥚 ELIDE — Easier Life @ IDE 🦆

<p align="center">
  <img src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" alt="ELIDE Duck Logo" width="220">
</p>

<h3 align="center">Your code. Your tools. Your workflow.</h3>

<p align="center">
  🦀 Built with Rust · ⚡ Terminal-native · 🤖 AI-powered
</p>

<p align="center">
  <strong>Less friction. More control. More building.</strong>
</p>

---

## 👋 Meet ELIDE

**ELIDE (Easier Life @ IDE)** is a Rust-powered terminal IDE designed to bring your development workflow into one focused environment.

Edit code, run commands, work with language servers, manage clipboard operations, and connect AI services—all without being locked into a massive development ecosystem.

ELIDE follows one simple philosophy:

**Your tools should work for you, not the other way around.**

## ✨ Why ELIDE?

- 📝 **Fast Editing** — Create, open, switch, and save files from one terminal workspace.
- ⚙️ **Bring Your Own Tools** — Use your preferred compilers, debuggers, and development utilities.
- 🔍 **Language Server Workflows** — Access compatible LSP functionality and diagnostics.
- 🤖 **AI Integration** — Connect to an AI provider through its API for chat and code assistance.
- 📋 **Clipboard Support** — Copy buffers, select line ranges, and paste where supported.
- 🧭 **Function Finder** — Quickly locate Rust-style function declarations.
- 🎨 **ELIDE Serenity** — A distinctive dark interface with lavender, sage, and misty-blue accents.
- 🦆 **Personality Included** — Easter eggs, unexpected surprises, and occasional reminders to take a break.

## 🤖 AI Meets Your Terminal

AI shouldn't require abandoning your development environment.

ELIDE brings API-connected AI assistance into your terminal workflow, making it possible to interact with an AI provider while staying focused on your project.

Configure your provider and use ELIDE's AI chat and checking workflows.

## 🛠️ Bring Your Own Ecosystem

ELIDE doesn't pretend to bundle every development tool.

Instead, it works alongside external utilities you choose to install and configure.

Examples include:

- `clangd` — C/C++ language-server support.
- `ada_language_server` — Ada language-server support.
- `gnatprove` — Ada verification tooling.
- `gdb` — Debugging.

External tools are independently installed and configured. ELIDE's current workflow does not require a dedicated plugin-loading API.

## 🚀 Get Started

### Requirements

- Rust
- Cargo
- A compatible terminal

### Build from source

```bash
git clone https://github.com/tunglamtrannguyen3-cmd/ELIDE.git
cd ELIDE
cargo build --release
```

Run the executable produced by your checkout.

Run the test suite:

```bash
cargo test
```

## ⌨️ Your Terminal, Your Commands

Open the command palette using `Alt+T`.

| Command | Action |
|---|---|
| `save` / `:w` | 💾 Save file |
| `-sa` / `:wa` | 📚 Save all |
| `new <file>` | 📄 Create file |
| `code <file>` | 📂 Open file |
| `switch <file>` | 🔀 Switch files |
| `copy 5-12` | 📋 Copy line range |
| `paste` | 📥 Paste |
| `:funcs` | 🧭 Find functions |
| `lsp <command>` | 🔍 LSP workflow |
| `check <query>` | 🤖 AI-assisted check |
| `sh <command>` | 🖥️ Run shell command |
| `acel <command>` | ⚙️ Run interactive command |
| `-bro!` | 🦆 Discover a break-time side quest |

**Keyboard shortcuts**

- `Ctrl+A` — AI chat
- `Alt+L` — LSP diagnostics
- `Esc` — Close active pane

*Commands and shortcuts may vary between versions. Check built-in help for your checkout.*

## 🧰 Technology

Built with tools chosen for a responsive terminal experience.

- 🦀 Rust
- 🖥️ Ratatui
- ⌨️ Crossterm
- ⚡ Tokio
- 🔍 Language Server Protocol integrations
- 📋 Clipboard support

## 🧑‍💻 Independently Built

ELIDE is developed independently by a 13-year-old developer.

From designing terminal interfaces to integrating language tools and AI APIs, the project is an ongoing exercise in building practical developer software.

No giant team. No corporate machinery. Just curiosity, Rust, and a duck with a development roadmap. 🦆

## 🤝 Join the Project

Have an idea, found a bug, or want to improve ELIDE?

Contributions are welcome!

- Open an issue for bugs and feature requests.
- Submit pull requests with improvements.
- Include reproduction steps when reporting bugs.
- Never include API keys, passwords, or other secrets in logs.

## 📦 Repository

[**GitHub — ELIDE**](https://github.com/tunglamtrannguyen3-cmd/ELIDE)

## 📄 License

ELIDE is distributed under the **Eggchese Open Project License (EOPL-2026.2)**.

See [LICENSE](LICENSE) for the complete terms.

---

<p align="center">
  <strong>🦆 ELIDE — Less waiting. More control. More building. 🚀</strong>
</p>
