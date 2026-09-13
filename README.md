# ⚡ ELIDE

### A lightweight terminal IDE that doesn't need a spaceship to edit a file.

ELIDE is a Rust-based terminal IDE built around one simple idea:

> **The terminal should be a comfortable development environment, not
> just a place where you launch another IDE.**

No giant workspace.\
No 47 panels fighting for your attention.\
No need to summon an entire software ecosystem just to edit `main.rs`.

Just **ELIDE**.

------------------------------------------------------------------------

## ✨ Features

### 🖥️ Terminal-first

ELIDE is designed from the ground up for terminal environments.

It is especially suited for:

-   Linux
-   Terminal workflows
-   Lightweight systems
-   Remote environments
-   Mobile/Termux-style environments

### ⚡ Command Palette

ELIDE uses a simple command palette with short aliases and familiar
commands.

  Command                                           What it does
  ------------------------------------------------- ----------------------------
  `-s` / `--save` / `save` / `:w`                   Save the current file
  `-c` / `--compile` / `compile` / `build` / `:b`   Compile/build
  `-d` / `--debug` / `debug`                        Debug with GDB
  `-i` / `--info` / `info`                          Show editor information
  `-h` / `-?` / `--help` / `help` / `?`             Show help
  `-bro!` / `bro` / `bro!`                          Random break generator
  `new <file>`                                      Create a new buffer
  `code <file>`                                     Open a file
  `switch <file>`                                   Switch/open another file
  `set-build <cmd>`                                 Set a custom build command
  `lsp <cmd>`                                       Configure an LSP command
  `sh <cmd>`                                        Execute a shell command

ELIDE also understands Vim-style commands such as:

``` text
:w
:b
```

------------------------------------------------------------------------

## 🎨 SOTA Color System

ELIDE uses its **SOTA (state of the art) color system** for status messages and syntax
highlighting.

The palette intentionally avoids turning the terminal into a radioactive
Christmas tree.

### Status colors

  Status       RGB               Purpose
  ------------ ----------------- -----------------------
  🟢 Success   `137, 243, 54`    Successful operations
  🟡 Warning   `255, 215, 0`     Warnings
  🔴 Error     `227, 83, 54`     Errors
  🔵 Hint      `193, 213, 240`   Informational hints
  ⚪ Unknown   `112, 128, 144`   Unknown commands/code

Status messages use different text attributes:

-   **Bold** → success, warning, error, hint
-   **Dim** → unknown commands/code

### Syntax colors

  Element       RGB
  ------------- -----------------
  Normal text   `205, 214, 244`
  Keywords      `203, 166, 247`
  Strings       `166, 227, 161`
  Comments      `147, 153, 178`
  Functions     `137, 180, 250`
  Types         `249, 226, 175`
  Numbers       `250, 179, 135`

> **Make the code readable without making the terminal scream at you.**

------------------------------------------------------------------------

# 🥊 ELIDE vs The Competition

Every editor has its place.

But that doesn't mean we can't have a little fun. 💀

## 🟦 VS Code

VS Code is extremely extensible and has a massive ecosystem.

ELIDE takes a rather different approach.

**VS Code:**

> "You can customize everything!"

**ELIDE:**

> "Cool. I wanted to edit a file."

The joke writes itself:

``` text
Install editor
↓
Install extensions
↓
Configure extensions
↓
Configure settings
↓
Install another extension
↓
Extension conflict
↓
Restart
↓
"Why did I install 37 extensions?"
```

ELIDE's philosophy:

> **If the terminal can do it, why build a spaceship around it?**

## 🟧 Android Studio

Android Studio is built for the enormous Android development ecosystem,
so it naturally brings a lot of specialized tooling.

ELIDE looks at the whole thing and says:

``` text
ELIDE:
"bro just compile the file"
```

If you're building a serious Android application, Android Studio's
specialized tooling makes sense.

If you are working in a lightweight terminal environment, ELIDE wants to
stay out of your way.

## 🟪 Solar

Solar can provide a much more feature-rich editor experience.

ELIDE deliberately goes the other direction:

``` text
Less UI
Less noise
Less ceremony
More terminal
```

ELIDE isn't trying to win by having **more buttons**.

It's trying to win by needing **fewer buttons**.

------------------------------------------------------------------------

# 🧠 The ELIDE Philosophy

### 1. Lightweight

Don't make the editor heavier than the project.

### 2. Terminal-native

The terminal isn't a limitation.

**It's the interface.**

### 3. Low visual noise

Your code should be the most interesting thing on screen.

### 4. Familiar commands

Short aliases make common operations quick.

### 5. Tool-friendly

ELIDE doesn't try to replace every Unix tool.

It works alongside them.

### 6. Take breaks

Yes, the IDE literally has:

``` text
-bro!
```

Because sometimes the correct debugging strategy is:

> **Go touch grass and come back.** 🌱

The command randomly suggests activities such as gaming, chess, or
simply sleeping.

------------------------------------------------------------------------

# 🛠️ Tech Stack

ELIDE is primarily written in:

-   🦀 Rust
-   `crossterm` for terminal interaction and styling
-   GDB for debugging workflows
-   Shell tooling through the command palette
-   LSP-related configuration/integration

------------------------------------------------------------------------

# 📦 Example

Common operations can be performed directly from the command palette:

``` text
-s
```

Save.

``` text
-c
```

Compile.

``` text
-d
```

Debug.

``` text
-i
```

Inspect the current editor state.

``` text
-bro!
```

Stop coding because your brain has filed a formal complaint. 💀

------------------------------------------------------------------------

# 🚧 Project Status

**Current version: v1.2.0**

ELIDE is actively evolving.

Some functionality is still being developed and refined, including the
broader LSP workflow.

------------------------------------------------------------------------

# 🎯 Long-Term Vision

ELIDE isn't trying to become:

> **VS Code but inside a terminal.**

That would defeat the point.

The goal is to build something with its **own identity**:

``` text
┌──────────────────────────────┐
│            ELIDE             │
├──────────────────────────────┤
│                              │
│  lightweight                 │
│  terminal-first              │
│  low visual noise            │
│  developer-focused           │
│                              │
│  Code → Build → Debug        │
│                              │
└──────────────────────────────┘
```

A development environment that feels natural on machines where
installing a massive IDE would be unnecessary---or simply impractical.

------------------------------------------------------------------------

# 🤝 Contributing

Issues, ideas, experiments, and improvements are welcome.

If you find something broken:

1.  Reproduce it.
2.  Describe what happened.
3.  Include relevant terminal output.
4.  Open an issue or submit a pull request.

And please don't report:

> "It doesn't work."

with absolutely nothing else.

I am begging you. 😭

------------------------------------------------------------------------

# 📜 License

See [`LICENSE.md`](LICENSE.md) for the project's current license terms.

------------------------------------------------------------------------

# 🥚 Easter Eggs

ELIDE contains a few things that aren't strictly necessary for an IDE.

That's intentional.

Because software doesn't have to be boring.

Try:

``` text
-bro!
```

You might get:

``` text
🎲 RANDOM BREAK GENERATOR

Time to step away from the keyboard.

Your assigned activity:

• Chess (Strategy / Mental Workout)
```

The compiler isn't going to run away.

Probably.

------------------------------------------------------------------------

# ⚡ ELIDE

**A terminal IDE for people who looked at a 20-panel IDE and said:**

> *"Bro, I just wanted to edit the file."*

Made with Rust 🦀, terminals 💻, questionable sleep schedules, and an
unhealthy amount of debugging.
