# ⚡🥚 ELIDE v2.1.3 — THE POLISH UPDATE 🦆

<p align="center">
  <img src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" alt="ELIDE Duck Logo" width="280"/>
</p>

<h3 align="center">🦆 Your Code. Your Terminal. YOUR RAM. ⚡</h3>

<p align="center">
  A lightweight, Rust-powered terminal IDE for developers who want to code more and fight their editor less.
</p>

<p align="center">
  🦀 Rust-powered · ⚡ Terminal-native · 🤖 AI-assisted · 🔍 LSP-enabled
</p>

<p align="center">
  <strong>🚫 No Electron empire. 🧠 No unnecessary complexity. 💻 Just CODE.</strong>
</p>

---

## 👋 Yo, welcome to ELIDE!

Ever opened an IDE to fix one tiny bug, only to discover that your computer has decided to simulate the heat death of the universe? 💀🔥

Ever installed so many editor extensions that configuring your editor became a whole separate software engineering career? 😭

Yeah. We know the vibe.

**ELIDE (Easier Life @ IDE)** is a Rust-based terminal IDE built around one simple idea:

> 🗣️ Your editor should launch before you finish questioning your life choices, not after.

ELIDE brings file editing, AI assistance, language-server integration, clipboard support, and command execution into one terminal-focused environment.

No need to launch an entire graphical universe just to edit a file. 🌌

Just open your terminal, summon the duck, and get to work. 🦆⌨️

### 💖 Why ELIDE?

- 🦀 **Built with Rust** — because systems programming deserves some love.
- ⚡ **Resource-conscious by design** — your RAM has better things to do than babysit your editor.
- 🤖 **AI co-pilot** — ask questions and audit your code without leaving your workflow.
- 🔍 **LSP integration** — bring language-server diagnostics into your terminal.
- 📋 **Clipboard support** — copy and paste code without fighting your environment.
- 🖥️ **Command execution** — run shell commands and interactive programs.
- 🎨 **Catppuccin-inspired colors** — pretty colors, fewer terminal-induced existential crises.
- 🥚 **Easter eggs** — because an IDE doesn't have to be a soulless rectangle.
- 🛌 **Developer recovery station** — even programmers need maintenance.

---

## ✨ Features that actually matter

### 📝 1. Code first. Complicated nonsense later.

Create, open, switch, and save files directly from the command palette.

```text
new main.rs
code main.rs
switch main.rs
save
```

Need to save everything? Quit? Save and quit?

ELIDE has commands for those, too.

Because sometimes the most advanced feature is simply **letting you get your work done.** 🫡

### 🤖 2. Your AI co-pilot lives in your terminal

Press `Alt + A` to open the AI chat interface.

Need help understanding a suspicious function? Want an AI-assisted audit of your workspace? Have a bug that makes absolutely no sense?

Give your configured AI provider some context and start investigating. 🔎

```text
check Review this Rust module for possible bugs
```

ELIDE's AI subsystem includes multi-turn chat streaming and workspace-audit functionality.

⚠️ You'll need a compatible AI provider, valid configuration, and network access. API keys are secrets, not decorations for your README. 🔐

### 🔍 3. LSP diagnostics without the treasure hunt

Press `Alt + L` to toggle the diagnostics pane.

```text
lsp rust-analyzer
```

Connect a compatible language server and inspect the diagnostics it reports.

No more playing hide-and-seek with every compiler complaint. 🫠

Actual language features depend on the language server you install and configure.

### 📋 4. Clipboard integration, because Ctrl+C shouldn't be a boss fight

Copy a line, grab a range, or paste text into your editor.

```text
copy 5
copy 5-12
paste
```

ELIDE integrates with a system clipboard abstraction, an internal buffer, and OSC 52 support.

Your terminal environment still matters, but at least copying code doesn't have to involve summoning ancient magic. 🧙‍♂️

### 🖥️ 5. The terminal is your playground

Run shell commands or launch interactive foreground programs.

```text
sh cargo test
acel cargo run
```

ELIDE's process subsystem supports command execution and resource-statistics reporting, including RSS tracking where supported.

Build. Test. Debug. Repeat.

The traditional developer lifestyle, now with fewer reasons to leave your terminal. 🦀⚙️

### 🧭 6. Find your functions before your patience expires

```text
:funcs
```

ELIDE can list matching Rust-style function declarations in the current buffer with their line numbers.

It's not a complete semantic code browser, but when you just need to find that function you wrote three hours ago, every second counts. 🔎

### 🎨 7. A color palette with actual vibes

ELIDE uses a custom Catppuccin-inspired RGB palette. 💜

| Element | Color | Vibe |
|---|---|---|
| 🔑 Keywords | Mauve | Magical |
| 🟡 Types | Yellow | Warm |
| 🍑 Numbers | Peach | Soft |
| 🌿 Strings | Green | Peaceful |
| 💙 Functions | Blue | Focused |
| 📝 Default text | Lavender | Classic ELIDE |
| 🌫️ Comments | Gray | Quiet |
| 🟥 Errors | Red | FIX THIS, BRO |
| 🟢 Success | Green | WE COOKED |
| 🟡 Warnings | Yellow | Something's suspicious |
| 🩵 Hints | Teal | A friendly suggestion |

The palette is defined in `src/colors.rs`, so the colors aren't just random decorations thrown into a terminal and called a theme.

**A little color. A little clarity. A lot less visual chaos.** 🎨

---

## 🥊 ELIDE vs. the usual suspects

| Editor | The good stuff 💚 | The trade-off 💀 |
|---|---|---|
| 🟦 VS Code | Massive extension ecosystem | Electron-based architecture and potentially substantial resource use |
| 🟥 JetBrains | Powerful language tooling | Indexing and analysis can be resource-intensive |
| 🟩 Helix | Modern terminal-native modal editing | Has a learning curve |
| 🟪 Neovim | Incredible customization | Advanced setups can become a hobby of their own |
| 🥚 **ELIDE** | Terminal IDE with AI, LSP, clipboard, and command execution | A developing ecosystem with fewer integrations than established IDEs |

We're not claiming every other editor is trash. 😭

VS Code is useful. JetBrains is powerful. Neovim is ridiculously customizable.

**ELIDE simply takes a different route:** bring useful development tools into a focused, terminal-native environment.

We're here to build software, not participate in an editor configuration speedrun. 🏃‍♂️💨

---

## ⌨️ Keybindings — learn these and cook

| Shortcut | Action |
|---|---|
| `Alt + A` 🤖 | Toggle AI chat |
| `Alt + T` 🛠️ | Toggle command palette / terminal pane |
| `Alt + L` 🔍 | Toggle LSP diagnostics |
| `Alt + Up` ⬆️ | Scroll diagnostics or terminal output upward |
| `Alt + Down` ⬇️ | Scroll diagnostics or terminal output downward |
| `Esc` 🏃 | Close the active pane or return focus to the editor |

## 🛠️ Command palette — the good stuff

Summon it with `Alt + T`. ⚡

| Command | What it does |
|---|---|
| `new <file>` 📄 | Create a buffer |
| `code <file>` 📂 | Open a file |
| `switch <file>` 🔀 | Switch files |
| `save`, `-s`, `:w` 💾 | Save the current buffer |
| `-sa`, `wa`, `:wa` 📚 | Save all open buffers |
| `-sl`, `:wq` 🚪 | Save and quit |
| `-l`, `:q!` 💨 | Quit without saving |
| `copy [a-b]` 📋 | Copy lines |
| `paste` 📥 | Paste clipboard contents |
| `lsp <cmd>` 🔍 | Request language-server startup |
| `check <query>` 🤖 | Run an AI-assisted audit |
| `sh <cmd>` 🖥️ | Execute a shell command |
| `acel <cmd>` 🎮 | Launch an interactive foreground command |
| `:funcs` 🧭 | Find matching function declarations |
| `-bro!` 🦆 | Summon the recovery station |
| `--im-<mood>` 💭 | Get a mood-related message |

---

## 🦆 The legendary Bro! Recovery Station

Listen up, fellow developer.

Have you been staring at the same bug for 47 minutes? Is your code refusing to compile? Has your brain started returning `SegmentationFault` instead of coherent thoughts? 💀

ELIDE has a solution.

```text
-bro!
```

🎲 **What you get:**

- 🎮 A random game recommendation.
- ⚔️ A side quest.
- 💧 A reminder to drink water, stretch, or touch grass.
- 🛌 A completely valid excuse to stop staring at your screen for a bit.

Feeling a particular way? Try a mood command:

```text
--im-stuck
--im-bored
--im-hungry
--im-tired
--im-stressed
--im-burntout
```

The responses are randomized. The bugs, unfortunately, are not. 😭

---

## 📦 Installation — let's get cooking

### 🧰 Prerequisites

- 🦀 Rust and Cargo.
- 💻 A compatible terminal environment.
- 🔍 Optional: compatible language servers.
- 🤖 Optional: an AI provider and credentials.
- 📋 Optional: system clipboard facilities.

Rust 1.70 or newer is the stated minimum; check compatibility with the current dependency versions before relying on that minimum.

### 🚀 Build from source

```bash
git clone https://github.com/tunglamtrannguyen3-cmd/ELIDE.git
cd ELIDE

cargo build --release

./target/release/elide
```

### 🔬 Run Clippy

```bash
rustup component add clippy
cargo clippy --all-targets -- -D warnings
```

### 🧪 Run tests

```bash
cargo test
```

A successful build or lint check is a good sign, not a magical guarantee that every feature works flawlessly on every machine. 🧪

---

## 🧱 Under the hood

| Component | Technology |
|---|---|
| 🦀 Primary language | Rust |
| 🖥️ Terminal handling | Crossterm |
| 🤖 AI integration | Asynchronous provider communication |
| 🔍 Language tooling | Language Server Protocol |
| 📋 Clipboard | System clipboard abstraction and OSC 52 support |
| ⚙️ Process execution | Shell and interactive command workflows |
| 🎨 Color system | Custom RGB palette |

ELIDE's source is organized into focused modules for editing, terminal handling, AI, LSP, clipboard, colors, command parsing, and process execution.

The goal is to keep the project understandable and maintainable as it grows. 🛠️

---

## 🗺️ The ELIDE philosophy

I built ELIDE around a simple idea:

> 💜 Development tools should help you build software, not become another project you have to maintain.

I want an IDE that respects system resources, keeps useful tools close at hand, and doesn't make the developer fight the environment before getting any actual work done.

ELIDE is my take on that idea: a terminal-first, Rust-powered workspace with its own personality.

It's still evolving. There's room for bugs, improvements, and features I haven't thought of yet.

That's part of building software. 🚀

**Small tools. Big ambitions. One suspiciously powerful duck.** 🦆⚡

---

## 🤝 Join the project

Found a bug? Have a feature idea? Want to help make ELIDE better?

You're welcome to explore the code, report issues, improve the documentation, or contribute changes.

1. 🍴 Fork the repository.
2. 🌱 Create a branch.
3. 🛠️ Make your changes.
4. 🧪 Test them.
5. 🚀 Open a pull request.

When reporting bugs, include reproduction steps and relevant environment information. Please redact API keys and other secrets from logs.

🔗 **Repository:** [github.com/tunglamtrannguyen3-cmd/ELIDE](https://github.com/tunglamtrannguyen3-cmd/ELIDE)

## 📄 License

ELIDE is distributed under the **Eggchese Open Project License 2026 (EOPL-2026.2)**.

📜 See [`LICENSE.md`](LICENSE.md) for the complete terms.

---

<p align="center">
  <strong>⚡ LESS WAITING. 💻 LESS BLOAT. 🚀 MORE BUILDING.</strong>
  <br/><br/>
  🦆 ELIDE — Your terminal, your workflow.
  <br/><br/>
  ⭐ If ELIDE looks interesting, check out the repository!
</p>
