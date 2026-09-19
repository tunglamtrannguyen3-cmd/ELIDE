<img width="1254" height="1254" alt="Duck-ai-image-2026-09-16-08-37" src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" />

# ⚡ ELIDE

### A lightweight terminal IDE that doesn't need a 32-gigabyte RAM donor to open a text file.

ELIDE is a Rust-based terminal IDE built around one indisputable truth:

> **Your editor should launch before you finish questioning your life choices, not after.**

No Electron bloat.\
No telemetry harvesting your keystrokes.\
No subscription model for saving a file.

Just **ELIDE**.

---

## 🥊 ELIDE vs The Competition

Every editor has its place. Unfortunately, most of them belong in a museum of modern architectural failures. Let's talk about how the industry got so deeply lost.

### 🟦 VS Code (The Electron Memory Vampire)
VS Code is what happens when web developers get unsupervised access to systems programming. 
* **The Pitch:** "It's extensible! You can customize everything!"
* **The Reality:** You install 47 extensions just to format a JSON file, your laptop fans start screaming like a jet engine preparing for takeoff, and your RAM usage climbs higher than rent prices in downtown Tokyo. 
* **ELIDE's Take:** If your text editor requires more resources than a modern 3D video game just to display "Hello World", something has gone horribly, irreversibly wrong with civilization.

### 🟥 JetBrains (The Corporate Enterprise Monolith)
IntelliJ, CLion, PyCharm—take your pick. They are magnificent, hyper-intelligent pieces of engineering wrapped in a UI that feels like navigating the cockpit of a Boeing 777.
* **The Pitch:** "We index your entire codebase down to the atomic level!"
* **The Reality:** You click open a single `.rs` file, and JetBrains immediately begins indexing the universe, downloading a JVM, indexing your git history from 2018, and freezing your entire desktop for three business days.
* **ELIDE's Take:** We didn't come here to build a micro-economy; we came to edit a file.

### 🟧 Android Studio (The Gradle Torture Chamber)
Android Studio is basically JetBrains' heavier, more aggressive cousin who hates you and wants you to suffer.
* **The Pitch:** "Everything you need for mobile development!"
* **The Reality:** You change a single padding value in an XML file, and Android Studio traps you in a 20-minute hostage situation called *Building Gradle*. 
* **ELIDE's Take:** `bro just compile the file`

### 🟩 Helix (The Modal Masochism Club)
Helix is genuinely fast, modern, and built in Rust. It’s a great piece of software. But it forces you into a modal editing mindset where you need a degree in theoretical mathematics and three hands just to type a comma.
* **The Pitch:** "A modern editor with built-in tree-sitter and multi-selections!"
* **The Reality:** You press the wrong keybinding while trying to delete a typo, accidentally re-architect the entire codebase, open a Vim portal to another dimension, and have to pull the power plug on your machine.
* **ELIDE's Take:** Modal editing is a cult disguised as ergonomics. Normal people just want to type text.

### 🟪 Neovim (The Configuration Stockholmers)
Neovim users don't actually write code; they spend 84 consecutive hours on GitHub copying other people's 5,000-line Lua configuration files just to get syntax highlighting to work on a Tuesday.
* **The Pitch:** "It's infinitely extensible if you learn Lua and write your own window manager!"
* **The Reality:** Your editor breaks every time a plugin gets deprecated, and your entire personality becomes telling strangers on Reddit why your custom dotfiles are superior.
* **ELIDE's Take:** If your editor configuration takes longer to set up than a fresh Linux install, you don't have a tool—you have a second job.

---

## ✨ Features

### 🖥️ Terminal-First
ELIDE stays right where you are. Perfect for:
- Linux & Unix environments
- Remote SSH sessions where latency is your only friend
- Low-spec hardware and lightweight containers
- Mobile/Termux setups when you're coding from your phone on the bus

### ⚡ Command Palette
ELIDE uses a straightforward command palette with clean aliases:

| Command | What it does |
| :--- | :--- |
| `-s` / `--save` / `save` / `:w` | Save the current file |
| `-d` / `--debug` / `debug` | Debug with GDB |
| `-i` / `--info` / `info` | Show editor information |
| `-h` / `-?` / `--help` / `help` / `?` | Show help |
| `-bro!` / `bro` / `bro!` | Random break generator |
| `new <file>` | Create a new buffer |
| `code <file>` | Open a file |
| `switch <file>` | Switch/open another file |
| `lsp <cmd>` | Configure an LSP command |
| `sh <cmd>` | Execute a shell command |
| `acel <cmd>` | Run foreground tasks/builds |

---

## 🎨 SOTA Color System

ELIDE uses a state-of-the-art color palette designed to look sleek without turning your terminal into a radioactive discotheque.

### Status Colors
* **🟢 Success (`137, 243, 54`)** — Clean operations
* **🟡 Warning (`255, 215, 0`)** — Friendly alerts
* **🔴 Error (`227, 83, 54`)** — Something went sideways
* **🔵 Hint (`193, 213, 240`)** — Helpful context
* **⚪ Unknown (`112, 128, 144`)** — Mysterious commands

> **Make the code readable without making the terminal scream at you.**

---

## 🧠 The ELIDE Philosophy

1. **Lightweight:** Don't make the editor heavier than the project you're building.
2. **Terminal-native:** The terminal isn't a limitation; it's the environment.
3. **Low visual noise:** Your code should be the most interesting thing on screen.
4. **Familiar commands:** Quick aliases keep your fingers on the home row.
5. **Tool-friendly:** ELIDE plays nicely with Unix tools instead of trying to replace them.
6. **Take breaks:** Sometimes the correct debugging strategy is closing the terminal, going outside, and touching grass. 🌱

---

# 🥚 Easter Eggs

Software doesn't have to be a joyless corporate product. Try typing:

```text
-bro!<img width="1254" height="1254" alt="Duck-ai-image-2026-09-16-08-37" src="https://github.com/user-attachments/assets/0f45a8b3-aae9-4809-b8b8-b27e015e9fa0" />
