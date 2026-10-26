
🥚 ELIDE — Easier Life @ IDE 🦆
<p align="center"> <img src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" alt="ELIDE Duck Logo" width="220"> </p> <p align="center"> <strong>🦀 Built with Rust. Powered by your tools. Driven by your ideas.</strong><br> 💻 Your code. Your terminal. Your rules. 💜 </p>
👋 What is ELIDE?

ELIDE is a Rust-powered terminal IDE that brings editing, shell commands, language-server workflows, clipboard tools, and optional AI assistance into one place.

No giant ecosystem forced on you. No pretending every tool works out of the box. You choose your tools; ELIDE gives you a place to use them. 🛠️
✨ Features

    📝 Quick editing — create, open, switch, and save files.
    ⚙️ Bring your own tools — run shell commands and interactive programs.
    🔍 LSP workflows — work with compatible language servers and diagnostics.
    🤖 AI assistance — use AI chat and checks with a configured provider.
    📋 Clipboard tools — copy buffers or line ranges and paste where supported.
    🧭 Function finder — locate Rust-style function declarations.
    🎨 ELIDE Serenity — a cozy dark palette with soft lavender, sage, and misty blue.
    🦆 A little chaos — Easter eggs and reminders to take a break.

External tools like clangd, ada_language_server, gnatprove, and gdb are tools you install and configure—not automatically bundled ELIDE plugins. The current workflow does not imply a dedicated plugin-loading API.
🚀 Build from source

You'll need Rust, Cargo, and a compatible terminal.
git clone https://github.com/tunglamtrannguyen3-cmd/ELIDE.git
cd ELIDE
cargo build --release

Run the executable produced by your checked-out version. To run tests:
cargo test
⌨️ Handy commands

Open the palette with Alt+T.
Command	What it does
save / :w	💾 Save
-sa / :wa	📚 Save all
new <file>	📄 Create a file
code <file>	📂 Open a file
switch <file>	🔀 Switch files
copy 5-12	📋 Copy a line range
paste	📥 Paste
:funcs	🧭 Find function declarations
lsp <command>	🔍 Request the LSP workflow
check <query>	🤖 AI-assisted check
sh <command>	🖥️ Run a shell command
acel <command>	⚙️ Run an interactive command
-bro!	🦆 Get a game suggestion or break-time side quest

Shortcuts: Alt+A AI chat · Alt+L LSP diagnostics · Esc close the active pane.

Commands and shortcuts can vary by version; use the built-in help for your checkout.
🧰 Built with

🦀 Rust · 🖥️ Ratatui + Crossterm · ⚡ Tokio · 🔍 LSP integrations · 📋 Clipboard support
🧑‍💻 Built by one developer

I'm 13, and I'm building ELIDE independently. Making a terminal IDE means learning and connecting a lot of moving parts—from Rust and terminal UI to tool integration and usability. It's a big project, and I'm improving it one feature (and one bug) at a time. 💪
🤝 Contribute

Found a bug or have an idea? Open an issue or pull request! Please include reproduction steps and remove API keys or other secrets from logs.

🔗 Repository:
https://github.com/tunglamtrannguyen3-cmd/ELIDE
📄 License

ELIDE uses the Eggchese Open Project License (EOPL-2026.2). See LICENSE for the terms.
<p align="center"> <strong>🦆 Less waiting. More control. More building. 🚀</strong> </p>
