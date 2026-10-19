# ⚡ ELIDE

<p align="center">
  <img src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" alt="ELIDE Duck Logo" width="300"/>
</p>

### A lightweight terminal IDE that doesn't need a 32-gigabyte RAM donor to open a text file.

ELIDE is a fast, Rust-based terminal IDE built around one indisputable truth:

> **Your editor should launch before you finish questioning your life choices, not after.**

No Electron bloat. No telemetry harvesting your keystrokes. No subscription model for saving a file. Just **ELIDE.**

---

# 🥊 ELIDE vs. The Competition

Every editor has its place. Unfortunately, some of those places appear to be directly between your RAM and your will to live. 💀

Let's talk about how the industry got here.

### 🟦 VS Code — The Electron Memory Vampire
**The Pitch:** *"It's extensible! You can customize everything!"*  
**The Reality:** You install 47 extensions just to format a JSON file, your laptop fans start screaming like a jet engine preparing for takeoff, and your RAM usage begins climbing faster than rent prices in downtown Tokyo.  
**ELIDE's Take:** If your text editor needs a substantial chunk of your computer's resources just to display `Hello World`, something has gone horribly, irreversibly wrong with civilization.

---

### 🟥 JetBrains — The Corporate Enterprise Monolith
**The Pitch:** *"We index your entire codebase down to the atomic level!"*  
**The Reality:** You open one `.rs` file and suddenly your IDE is indexing your project, your Git history, the known universe, and possibly your bloodline. Then your computer freezes.  
**ELIDE's Take:** We didn't come here to build a micro-economy. We came here to edit a file.

---

### 🟧 Android Studio — The Gradle Torture Chamber
**The Pitch:** *"Everything you need for mobile development!"*  
**The Reality:** You change one padding value and Android Studio responds with a 20-minute hostage situation called `BUILDING GRADLE...`  
**ELIDE's Take:** *bro just compile the file*

---

### 🟩 Helix — The Modal Masochism Club
**The Pitch:** *"A modern editor with built-in tree-sitter and multi-selections!"*  
**The Reality:** You press the wrong key while trying to delete a typo and suddenly you're learning an entirely new keyboard dialect. One wrong move later: `WHY IS MY CURSOR DOING THAT`  
**ELIDE's Take:** Modal editing is not for everyone. Some people just want to type text without accidentally opening a portal to another dimension.

---

### 🟪 Neovim — The Configuration Stockholmers
**The Pitch:** *"It's infinitely extensible!"*  
**The Reality:** You spend 84 consecutive hours on GitHub assembling a 5,000-line Lua configuration because your editor apparently needs a PhD before it can highlight a function. Then one plugin gets deprecated. Everything explodes.  
**ELIDE's Take:** If configuring your editor takes longer than installing an operating system, you don't have a tool. You have a second job.

---

# ✨ Core Features

- ⚡ **Instant Startup:** Zero loading screens, zero Electron runtime overhead.
- 💬 **Multi-Turn AI Chat (`Alt+A`):** Built-in workspace-aware AI assistant powered by DeepSeek/OpenAI.
- 🛠️ **Command Palette & Terminal Pane (`Alt+T`):** Run background or foreground shell tasks directly inside the editor.
- 🩺 **LSP Integration (`Alt+L`):** Real-time code diagnostics, errors, and hints for Rust, Python, TypeScript, C/C++, and more.
- 📂 **Workspace Tree Navigator:** Easily switch directories, view workspace layouts, and manage buffers.
- 🎨 **SOTA Catppuccin-Inspired Color System:** Relaxing, low-eyestrain, high-contrast palette.

---

# ⌨️ Hotkeys & Keybindings

| Hotkey | Action |
| :--- | :--- |
| `Alt + A` | Open / Close persistent AI Chat pane |
| `Alt + T` | Open / Close Command Palette & Terminal pane |
| `Alt + L` | Toggle LSP Diagnostics Pane |
| `Alt + Up / Down` | Scroll LSP diagnostics window |
| `Esc` | Close active pane / Return focus to code editor |

---

# ⚡ Command Palette Reference

Press `Alt + T` to activate the command palette and execute commands:

| Command | Description |
| :--- | :--- |
| `code <file>` | Open or create a file buffer |
| `switch <file>` | Switch active workspace file |
| `save` / `:w` / `-s` | Save current file changes |
| `lsp <cmd>` | Start an LSP server (e.g., `lsp rust-analyzer`) |
| `check <query>` | Audit your codebase context with the AI engine |
| `check <token> <url>` | Dynamically set AI API Key and endpoint URL |
| `sh <cmd>` | Execute a system shell command |
| `acel <cmd>` | Run foreground tasks or build scripts |
| `debug` / `-d` | Start debugging with GDB |
| `-bro!` / `bro` | Take a break (Random break generator) |

*And yes, `acel` is intentionally spelled like that. No, we're not fixing it. It has character. 🗿*

---

# 🎨 SOTA Color Palette

ELIDE uses a high-contrast, low-eyestrain color palette designed to look sleek without turning your terminal into a radioactive disco:

- 🟢 **Success** — `137, 243, 54` — Clean operations
- 🟡 **Warning** — `255, 215, 0` — Friendly alerts
- 🔴 **Error** — `227, 83, 54` — Something went sideways
- 🔵 **Hint** — `193, 213, 240` — Helpful context
- ⚪ **Unknown** — `112, 128, 144` — Mysterious commands

---

# 🧠 The ELIDE Philosophy

1. ⚡ **Lightweight** — Don't make the editor heavier than the project you're building.
2. 🖥️ **Terminal-native** — The terminal isn't a limitation; it's the optimal environment.
3. 👀 **Low visual noise** — Your code should be the most interesting thing on screen.
4. ⌨️ **Familiar commands** — Quick aliases keep your fingers on the keyboard.
5. 🔧 **Tool-friendly** — ELIDE works with Unix tools instead of trying to replace everything.
6. 🌱 **Take breaks** — Sometimes the correct debugging strategy is closing the terminal, going outside, and touching grass.

---

# 📦 Installation & Setup

### Prerequisites
- [Rust & Cargo](https://www.rust-lang.org/tools/install) (1.70 or higher)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/tunglamtrannguyen3-cmd/ELIDE.git
cd ELIDE

# Build in release mode
cargo build --release

# Launch ELIDE
./target/release/elide
```

---

# 🤖 Configuring AI Assistance

ELIDE includes a built-in AI assistant. You can configure credentials via environment variables or on-the-fly inside the editor:

### Option 1: Environment Variables
```bash
export AI_API_KEY="your_api_key_here"
export AI_API_URL="https://api.deepseek.com/v1/chat/completions" # Or OpenAI / Ollama endpoint
export AI_MODEL="deepseek-coder"
```

### Option 2: On-the-Fly via Command Palette
In ELIDE, press `Alt + T` and type:
```text
check <your_api_key> <your_api_url>
```

---

# 🥚 Easter Eggs

Software doesn't have to be a joyless corporate product. Try typing:

```text
-bro!
```

And if you're completely lost:

```text
also pls help on my README.md
```

Because apparently even ELIDE needs emotional support sometimes. 💀

---

# 📄 License

Distributed under the **Eggchese Open Project License 2026 (EOPL-2026.2)**.  
See [`LICENSE.md`](LICENSE.md) for full non-commercial and private deployment terms.

---

<p align="center">
  <i>© 2026 Tung Lam Tran Nguyen. Built with 🦀 Rust for a lighter, faster developer experience.</i>
</p>