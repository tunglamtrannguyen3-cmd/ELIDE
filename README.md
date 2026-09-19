<img width="1254" height="1254" alt="Duck-ai-image-2026-09-16-08-37" src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" />

# ⚡ ELIDE

### A lightweight terminal IDE that doesn't need a 32-gigabyte RAM donor to open a text file.

ELIDE is a Rust-based terminal IDE built around one indisputable truth:

> **Your editor should launch before you finish questioning your life choices, not after.**

No Electron bloat.
No telemetry harvesting your keystrokes.
No subscription model for saving a file.

Just **ELIDE.**

---

# 🥊 ELIDE vs. The Competition

Every editor has its place.

Unfortunately, some of those places appear to be directly between your RAM and your will to live. 💀

Let's talk about how the industry got here.

## 🟦 VS Code — The Electron Memory Vampire

VS Code is what happens when web developers get unsupervised access to systems programming.

**The Pitch:**

> "It's extensible! You can customize everything!"

**The Reality:**
You install 47 extensions just to format a JSON file, your laptop fans start screaming like a jet engine preparing for takeoff, and your RAM usage begins climbing faster than rent prices in downtown Tokyo.

**ELIDE's Take:**
If your text editor needs a substantial chunk of your computer's resources just to display:

```text
Hello World
```

something has gone horribly, irreversibly wrong with civilization.

---

## 🟥 JetBrains — The Corporate Enterprise Monolith

IntelliJ. CLion. PyCharm.

Take your pick.

They're incredibly capable pieces of software wrapped in a UI that sometimes feels like you're operating the cockpit of a Boeing 777.

**The Pitch:**

> "We index your entire codebase down to the atomic level!"

**The Reality:**
You open one `.rs` file and suddenly your IDE is indexing your project, your Git history, the known universe, and possibly your bloodline.

Then your computer freezes.

**ELIDE's Take:**
We didn't come here to build a micro-economy.

We came here to edit a file.

---

## 🟧 Android Studio — The Gradle Torture Chamber

Android Studio is basically JetBrains' heavier cousin who has personally decided that your afternoon is no longer yours.

**The Pitch:**

> "Everything you need for mobile development!"

**The Reality:**
You change one padding value and Android Studio responds with a 20-minute hostage situation called:

```text
BUILDING GRADLE...
```

**ELIDE's Take:**

```text
bro just compile the file
```

---

## 🟩 Helix — The Modal Masochism Club

To be fair: Helix is genuinely fast, modern, and built in Rust. It's a serious editor.

But then you discover modal editing.

**The Pitch:**

> "A modern editor with built-in tree-sitter and multi-selections!"

**The Reality:**
You press the wrong key while trying to delete a typo and suddenly you're learning an entirely new keyboard dialect.

One wrong move later:

```text
WHY IS MY CURSOR DOING THAT
```

**ELIDE's Take:**
Modal editing is not for everyone.

Some people just want to type text without accidentally opening a portal to another dimension.

---

## 🟪 Neovim — The Configuration Stockholmers

Neovim users don't actually write code.

They configure the environment in which they will eventually write code.

**The Pitch:**

> "It's infinitely extensible!"

**The Reality:**
You spend 84 consecutive hours on GitHub assembling a 5,000-line Lua configuration because your editor apparently needs a PhD before it can highlight a function.

Then one plugin gets deprecated.

Everything explodes.

**ELIDE's Take:**
If configuring your editor takes longer than installing an operating system, you don't have a tool.

You have a second job.

---

# ✨ Features

## 🖥️ Terminal-First

ELIDE stays where you already are: **the terminal.**

Perfect for:

* 🐧 Linux & Unix environments
* 🌐 Remote SSH sessions where latency is your only friend
* 💾 Low-spec hardware and lightweight containers
* 📱 Mobile/Termux environments
* 🚌 Coding from your phone when you probably should be paying attention to where you're going

---

## ⚡ Command Palette

ELIDE uses straightforward commands with convenient aliases:

| Command                               | What it does                |
| :------------------------------------ | :-------------------------- |
| `-s` / `--save` / `save` / `:w`       | Save the current file       |
| `-d` / `--debug` / `debug`            | Debug with GDB              |
| `-i` / `--info` / `info`              | Show editor information     |
| `-h` / `-?` / `--help` / `help` / `?` | Show help                   |
| `-bro!` / `bro` / `bro!`              | Random break generator      |
| `new <file>`                          | Create a new buffer         |
| `code <file>`                         | Open a file                 |
| `switch <file>`                       | Switch/open another file    |
| `lsp <cmd>`                           | Configure an LSP command    |
| `sh <cmd>`                            | Execute a shell command     |
| `acel <cmd>`                          | Run foreground tasks/builds |

And yes, `acel` is intentionally spelled like that.

No, we're not fixing it.

It has character. 🗿

---

# 🎨 SOTA Color System

ELIDE uses a relaxing color palette designed to make your terminal look sleek without turning it into a radioactive disco.

### Status Colors

* 🟢 **Success** — `137, 243, 54` — Clean operations
* 🟡 **Warning** — `255, 215, 0` — Friendly alerts
* 🔴 **Error** — `227, 83, 54` — Something went sideways
* 🔵 **Hint** — `193, 213, 240` — Helpful context
* ⚪ **Unknown** — `112, 128, 144` — Mysterious commands

> **Make the code readable without making the terminal scream at you.**

---

# 🧠 The ELIDE Philosophy

1. ⚡ **Lightweight** — Don't make the editor heavier than the project you're building.
2. 🖥️ **Terminal-native** — The terminal isn't a limitation; it's the environment.
3. 👀 **Low visual noise** — Your code should be the most interesting thing on screen.
4. ⌨️ **Familiar commands** — Quick aliases keep your fingers on the keyboard.
5. 🔧 **Tool-friendly** — ELIDE works with Unix tools instead of trying to replace everything.
6. 🌱 **Take breaks** — Sometimes the correct debugging strategy is closing the terminal, going outside, and touching grass.

---

# 🥚 Easter Eggs

Software doesn't have to be a joyless corporate product.

Try typing:

```text
-bro!
```

And if you're completely lost:

```text
also pls help on my README.md
```

Because apparently even ELIDE needs emotional support sometimes. 💀
