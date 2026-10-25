# 🥚 ELIDE — Easier Life @ IDE 🦆⚡

<p align="center">
  <strong>🦀 Built with Rust. Powered by your tools. Driven by your ideas.</strong>
</p>

<p align="center">
  <img src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" alt="ELIDE Duck Logo" width="240"/>
</p>

<p align="center">
  💻 Your Code. Your Terminal. Your RAM. 💜
</p>

<p align="center">
  <strong>Less waiting. Less bloat. More building. 🚀</strong>
</p>

---

## 👋 Welcome to ELIDE!

Ever opened an IDE to fix one tiny bug, only to end up configuring the editor instead of fixing the bug? 😭

Ever wanted a development environment that stays in your terminal, works with the tools *you* choose, and doesn't force you into somebody else's workflow?

**That's the idea behind ELIDE.**

ELIDE (Easier Life @ IDE) is a Rust-powered, terminal-based development environment built to bring useful coding tools into one place. It combines code editing, command execution, language-server integration, clipboard support, and AI-assisted workflows in a focused terminal interface.

I'm building ELIDE because I want development tools to help me build software—not become another project I have to maintain. 🛠️

And yes, an IDE can have a little personality, too. 🦆

## ✨ What makes ELIDE different?

- 🦀 **Rust-powered:** A systems-language foundation for a terminal-native development environment.
- 🖥️ **Terminal-first:** Keep your coding workflow close to your shell and existing toolchain.
- 🔌 **Bring your own tools:** Choose and add the external tools that fit your workflow. ELIDE doesn't need to dictate your entire development ecosystem.
- 🤖 **AI-assisted workflows:** Get help with coding questions and workspace audits when a compatible provider is configured.
- 🔍 **LSP integration:** Work with compatible language servers for diagnostics and language tooling.
- 📋 **Clipboard support:** Copy and paste code through supported clipboard mechanisms.
- ⚙️ **Command execution:** Run shell commands and interactive programs from your workflow.
- 🎨 **Custom RGB palette:** A Catppuccin-inspired color scheme with distinct syntax and status colors.
- 🥚 **A little chaos:** Easter eggs, playful messages, and reminders that developers are humans, too.

No claim that ELIDE replaces every IDE. No claim that every external tool works automatically.

Just a development environment with a clear direction. 💜

---

## 🔌 Your tools. Your rules. Your ecosystem.

Here's an important part of ELIDE's philosophy:

**The user chooses the tools.**

Different developers need different workflows. One person might want a debugger, another might want a compiler, and someone else might have a completely different collection of utilities.

Why should an IDE decide that everyone must use the same set?

With ELIDE, you can build your workflow around the tools you want to use. Run external commands, use compatible language servers, configure an AI provider, and keep using the software that makes sense for your projects.

🛠️ Want a different compiler? Use the one that fits your project.

🔍 Need a language server? Configure a compatible one.

🐛 Want to debug? Use your preferred debugger where your environment and workflow support it.

⚙️ Have a specialized utility? Make it part of your toolchain.

**Your environment should adapt to your work—not the other way around.**

> Note: ELIDE's current command and integration mechanisms should not be confused with a dedicated plugin-loading API. External tools and integrations depend on how they're configured and invoked.

---

## 📝 Features

### 1. 💾 File editing without the unnecessary detours

Create files, open existing ones, switch between files, and save your work through the command palette.

```text
new main.rs
code main.rs
switch main.rs
save
```

Need to save all open buffers? There's a command for that, too.

Need to quit without saving? ELIDE has a command for that as well.

Because losing your work is a bug nobody wants to debug. 💀

### 2. 🤖 AI-assisted development

Sometimes the compiler explains the problem. Sometimes it gives you a message that looks like it was written by an ancient oracle.

That's where AI assistance can help. 🧠

ELIDE includes an AI chat interface and an AI-assisted workspace-check workflow.

Example:

```text
check Review this Rust module for possible bugs
```

AI integration depends on a compatible provider and valid configuration. Network access may also be required.

🔐 **Keep your API keys private.** Never commit credentials to your repository or paste them into public logs.

AI can help you investigate problems, but always review its suggestions before trusting them.

### 3. 🔍 Language-server integration

ELIDE supports language-server workflows through compatible LSP implementations.

The intended workflow includes:

- Diagnostics from configured language servers.
- A diagnostics pane.
- Language tooling supplied by the server you choose.

Example command:

```text
lsp rust-analyzer
```

The actual capabilities depend on the configured language server, its availability, and your environment.

ELIDE doesn't magically provide every language's compiler or language server. Bring the appropriate tools and configure them for your setup. 🦀

### 4. 📋 Copy, paste, and keep moving

Code is meant to move between files, terminals, and applications. Your clipboard shouldn't become the final boss. 😭

```text
copy
copy 5
copy 5-12
paste
```

These commands support copying the current buffer or selected line ranges and pasting clipboard contents.

ELIDE's clipboard implementation can work with its internal buffer, system clipboard integration, and OSC 52 where supported.

Your terminal and desktop environment may affect which clipboard mechanisms are available.

### 5. 🖥️ Run commands from your workflow

Already have a compiler, debugger, formatter, or other utility installed?

Use your existing toolchain instead of expecting an IDE to bundle the entire software universe. 🌌

```text
sh cargo test
acel cargo run
```

ELIDE includes shell-command and interactive-command workflows.

The tools you can run depend on what's installed and available in your environment.

**ELIDE gives you a place to work with your tools. You decide which tools belong there.**

### 6. 🧭 Find functions faster

Need to find where a function is declared?

```text
:funcs
```

ELIDE can list matching Rust-style function declarations from the current buffer, including their line numbers.

It's a handy shortcut when you just need to locate a function without manually scanning the whole file. 🔎

### 7. 🎨 A custom palette with actual personality

ELIDE uses a custom RGB palette defined in `src/colors.rs`.

| Element | Color | RGB |
|---|---|---|
| 🔑 Keywords | Mauve | `203, 166, 247` |
| 🟡 Types | Yellow | `249, 226, 175` |
| 🍑 Numbers | Peach | `250, 179, 135` |
| 🌿 Strings | Green | `166, 227, 161` |
| 💙 Functions | Blue | `137, 180, 250` |
| 💜 Default text | Lavender | `205, 214, 244` |
| 🌫️ Comments | Gray | `147, 153, 178` |
| 🟥 Errors | Red | `243, 139, 168` |
| 🟢 Success | Green | `166, 227, 161` |
| 🟡 Warnings | Yellow | `250, 227, 176` |
| 🩵 Hints | Teal | `148, 226, 213` |

The status system also distinguishes different message categories, with bold styling applied to selected statuses.

It's not just about making the terminal look pretty. Consistent visual cues help make information easier to scan. 🎨

### 8. 🦆 The Bro! Recovery Station

Coding for too long can turn even a simple bug into a personal enemy.

ELIDE has a playful recovery feature for those moments.

```text
-bro!
```

Depending on the response, you might get a random game suggestion or a small real-world side quest.

🎮 Play something.

💧 Drink some water.

🧍 Stretch a little.

🌳 Touch grass.

😴 Take a break when you need one.

You can also try mood-related commands:

```text
--im-stuck
--im-bored
--im-hungry
--im-tired
--im-stressed
--im-burntout
```

The goal isn't to make your editor another productivity coach. It's just a little humor in the middle of a debugging session.

The bugs may be persistent. Your editor doesn't have to be boring. 🦆

---

## ⌨️ Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Alt + A` 🤖 | Toggle AI chat |
| `Alt + T` 🛠️ | Toggle the command palette / terminal pane |
| `Alt + L` 🔍 | Toggle LSP diagnostics |
| `Alt + Up` ⬆️ | Scroll the relevant output pane upward |
| `Alt + Down` ⬇️ | Scroll the relevant output pane downward |
| `Esc` 🏃 | Close the active pane or return to the editor |

Exact behavior may depend on the active pane and current application state.

---

## 🧰 Command palette cheat sheet

Summon the palette with `Alt + T` and use the commands you need.

| Command | Action |
|---|---|
| `-s`, `--save`, `save`, `:w` 💾 | Save the current buffer |
| `-sa`, `--save-all`, `wa`, `:wa` 📚 | Save all buffers |
| `-l`, `:q!`, `q!`, `quit!` 🚪 | Quit without saving |
| `-sl`, `:wq`, `wq` 💾 | Save and quit |
| `-i`, `--info`, `info` ℹ️ | Show editor information |
| `-h`, `--help`, `help`, `?` 📖 | Show help |
| `new <file>` 📄 | Create a buffer |
| `code <file>` 📂 | Open a file |
| `switch <file>` 🔀 | Switch files |
| `copy [a-b]` 📋 | Copy the buffer, a line, or a range |
| `paste` 📥 | Paste clipboard contents |
| `:funcs` 🧭 | List matching function declarations |
| `lsp <cmd>` 🔍 | Request LSP startup |
| `check <query>` 🤖 | Run an AI-assisted check |
| `sh <cmd>` 🖥️ | Run a shell command |
| `acel <cmd>` ⚙️ | Run an interactive command |
| `-bro!` 🦆 | Open the recovery station |
| `--im-<mood>` 💭 | Show a mood-related message |

The command parser also recognizes several alternative spellings and aliases. Check the current source or help output for the exact behavior of your build.

---

## 📦 Installation

### 🧰 Requirements

- 🦀 Rust and Cargo for building from source.
- 💻 A compatible terminal environment.
- ⚙️ The external development tools you want to use.
- 🔍 Optional: compatible language servers.
- 🤖 Optional: an AI provider and its configuration.
- 📋 Optional: clipboard facilities supported by your environment.

ELIDE is designed to work alongside your toolchain. You generally don't need an IDE to bundle every compiler, debugger, and runtime you might ever use.

### 🚀 Build from source

```bash
git clone https://github.com/tunglamtrannguyen3-cmd/ELIDE.git
cd ELIDE

cargo build --release
```

Run the resulting executable:

```bash
./target/release/elide
```

If the executable name or startup behavior differs in your checked-out version, follow the project's current source and build configuration.

### 🧪 Run tests

```bash
cargo test
```

### 🔬 Run Clippy

```bash
rustup component add clippy
cargo clippy --all-targets -- -D warnings
```

These commands are useful for checking the project, but a passing build or lint run doesn't guarantee that every integration works on every system.

---

## 🧱 Under the hood

| Component | Technology / approach |
|---|---|
| 🦀 Core implementation | Rust |
| 🖥️ Terminal interface | Crossterm and Ratatui |
| ⚡ Asynchronous workflows | Tokio |
| 🔍 Language tooling | Language Server Protocol |
| 🤖 AI functionality | Configurable provider integration |
| 📋 Clipboard | System clipboard abstraction and OSC 52 support |
| ⚙️ External tools | Shell and interactive process execution |
| 🎨 Visual design | Custom RGB palette |

The project uses Rust's ecosystem to bring terminal interaction, asynchronous operations, error handling, and application functionality together.

The intention is to keep ELIDE practical, understandable, and open to future development. 🛠️

---

## 🥊 How does ELIDE compare?

Different editors solve different problems. Here's where ELIDE fits.

| Editor | Strength | Trade-off |
|---|---|---|
| 🟦 VS Code | Huge extension ecosystem and broad language support | Electron-based architecture; resource use varies with workload and extensions |
| 🟥 JetBrains IDEs | Powerful language-specific tooling | Some workloads can require substantial memory and indexing time |
| 🟩 Helix | Terminal-native modal editing and built-in language tooling | Different editing model and learning curve |
| 🟪 Neovim | Highly customizable and extensible | Advanced configurations can take time to maintain |
| 🥚 **ELIDE** | Terminal-focused editing, external tools, AI assistance, and LSP workflows | Smaller ecosystem and evolving feature set |

This isn't a scientific performance ranking. Actual resource usage and responsiveness depend on hardware, workload, configuration, and enabled integrations.

ELIDE's goal isn't to win every comparison.

**It's to provide a development environment that fits the way I want to build software—and give other developers room to make it their own.** 💜

---

## 🗺️ The roadmap mindset

ELIDE is an evolving project, not a finished universe.

The long-term direction is to keep improving the core editing experience, tool integration, language-server workflows, AI assistance, and overall usability.

I'm especially interested in making the development environment more useful without forcing users to adopt a single predefined stack.

Feature requests, bug reports, and contributions are welcome. Just remember: ideas are not the same as implemented features, and the actual roadmap should reflect the code that ships.

One step at a time. One bug at a time. One questionable duck at a time. 🦆

---

## 🤝 Contributing

Want to help improve ELIDE? Awesome! 🚀

You can contribute by:

- 🐛 Reporting reproducible bugs.
- 💡 Suggesting useful features.
- 📝 Improving documentation.
- 🧪 Testing changes.
- 🛠️ Submitting code improvements.

A good bug report includes reproduction steps, expected behavior, actual behavior, and relevant environment details.

Please remove API keys, tokens, and other private information before sharing logs.

### A simple contribution workflow

1. 🍴 Fork the repository.
2. 🌱 Create a branch for your change.
3. 🛠️ Implement the improvement.
4. 🧪 Run relevant tests.
5. 🚀 Open a pull request describing what changed.

🔗 **Repository:** https://github.com/tunglamtrannguyen3-cmd/ELIDE

---

## 📄 License

ELIDE uses the **Eggchese Open Project License (EOPL-2026.2)**.

Please consult the repository's `LICENSE` file for the authoritative license terms.

---

## 💜 A note from the developer

I built ELIDE around a simple idea: development tools should make building software easier, not make the developer fight the environment before getting started.

I want a terminal IDE that respects the tools people already use, supports different workflows, and has enough personality to feel like more than another generic editor.

It's a project that will keep evolving. There will be things to improve, features to rethink, and bugs to hunt down.

That's software development. You build something, learn from it, and make it better.

Thanks for checking out ELIDE. Whether you're here to use it, explore the code, report a bug, or contribute something new, I appreciate the interest. 💜

<p align="center">
  <strong>🦀 LESS WAITING. 🛠️ MORE CONTROL. 🚀 MORE BUILDING.</strong>
  <br/><br/>
  <strong>🥚 ELIDE — Your Code. Your Terminal. Your Rules. 🦆</strong>
</p>
