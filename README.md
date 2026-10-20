
# ⚡ ELIDE (v2.1.2)

<p align="center">
  <img src="https://github.com/user-attachments/assets/46a16081-bff4-4e32-8506-5727a1076b5f" alt="ELIDE Duck Logo" width="300"/>
</p>

### A lightweight terminal IDE for developers who value their RAM and their sanity.

ELIDE is an ultra-fast, Rust-based terminal IDE built on one undeniable truth:
> **Your editor should launch before you finish questioning your life choices, not after.**

No Electron bloat. No mysterious telemetry harvesting your keystrokes. No multi-gigabyte memory footprint. Just pure, unadulterated speed.

---

# 🥊 ELIDE vs. The Competition

Let's be honest about why the industry standard feels like running a physics simulation on a toaster:

### 🟦 VS Code — The Electron Memory Vampire
* **The Pitch:** *"It's extensible! You can customize everything!"*[cite: 7]
* **The Reality:** You install 47 extensions just to format a single JSON file, your laptop fans scream like a jet engine preparing for takeoff, and your RAM usage climbs faster than rent prices.
* **ELIDE's Take:** If your text editor requires a 32GB RAM donor just to display `Hello World`, civilization has failed.

### 🟥 JetBrains — The Corporate Enterprise Monolith
* **The Pitch:** *"We index your entire codebase down to the atomic level!"*[cite: 7]
* **The Reality:** You open one `.rs` file and suddenly your IDE is indexing your project, your git history, the known universe, and your bloodline. Then your computer freezes.
* **ELIDE's Take:** We didn't come here to build a micro-economy. We came here to edit a file.

### 🟩 Helix & Neovim — The Modal Masochists
* **The Pitch:** *"Infinite customizability and modal keybindings!"*[cite: 7]
* **The Reality:** You press the wrong key while trying to delete a typo, and suddenly you're trapped in an alternate dimension writing a 5,000-line Lua config file.
* **ELIDE's Take:** If configuring your editor takes longer than installing an OS, you don't have a tool. You have a second job.

---

# ✨ Core Architecture & Features (v2.1.2)

ELIDE's codebase is partitioned into modular, zero-warning Rust components:
* **Terminal Guard & Event Polling**: Raw mode, alternate screen buffers, and bracketed paste mode are managed in `src/terminal.rs`[cite: 10].
* **System-Wide Clipboard Integration**: Seamlessly copy lines or entire files (`copy [a-b]`) and paste via `arboard` bindings inside `src/palette.rs`[cite: 8].
* **Conversational AI Co-Pilot (`Alt+A`)**: Multi-turn chat (`run_chat_stream`) and repository audits (`run_check`) with automatic HTTP connection pooling are implemented in `src/ai.rs`[cite: 1].
* **LSP Workspace Diagnostics (`Alt+L`)**: Real-time language server protocol communication, initialization, and background diagnostic streaming run in `src/lsp.rs`[cite: 5], backed by an asynchronous pipe writer in `src/lsp_writer.rs`[cite: 6].
* **Safe Buffer Engine**: Handles UTF-8 character-to-byte index mapping (`char_to_byte_idx`), visual terminal column width calculations for CJK/emojis, and POSIX trailing newlines in `src/editor.rs`[cite: 4].
* **Low-Noise Theme System**: SOTA Catppuccin-inspired syntax highlighting and status palettes are defined in `src/colors.rs`[cite: 2].
* **Process Execution & Kernel Stats**: Run background shell commands with RSS memory tracking (`run_bash_cmd`) or surrender the terminal for interactive tasks (`run_interactive_cmd`) in `src/process.rs`[cite: 9].

---

# ⌨️ Hotkeys & Keybindings

| Hotkey | Action | Source Module |
| :--- | :--- | :--- |
| `Alt + A` | Open / Close persistent AI Chat pane[cite: 7] | `src/main.rs`[cite: 7] |
| `Alt + T` | Open / Close Command Palette & Terminal pane[cite: 7] | `src/main.rs`[cite: 7] |
| `Alt + L` | Toggle LSP Diagnostics Pane[cite: 7] | `src/main.rs`[cite: 7] |
| `Alt + Up / Down` | Scroll LSP diagnostics window or terminal output[cite: 7] | `src/main.rs`[cite: 7] |
| `Esc` | Close active pane / Return focus to code editor[cite: 7] | `src/main.rs`[cite: 7] |

---

# ⚡ Savage Command Palette Reference

Press `Alt + T` to summon the command palette[cite: 7]. Type a command or test ELIDE's patience[cite: 8]:

| Command | Description |
| :--- | :--- |
| `code <file>` | Open or create a file buffer[cite: 8] |
| `switch <file>` | Switch active workspace file[cite: 8] |
| `save` / `:w` / `-s` | Save current file changes to disk[cite: 4, 8] |
| `copy [a-b]` / `paste` | Copy line ranges or sync directly with the OS system clipboard[cite: 8] |
| `lsp <cmd>` | Launch an LSP server (e.g., `lsp rust-analyzer`)[cite: 5, 8] |
| `check <query>` | Audit your full workspace codebase context with ELIDE AI[cite: 1, 8] |
| `check <token> <url>` | Inject API credentials on the fly[cite: 1, 8] |
| `sh <cmd>` | Execute shell commands with kernel resource stats[cite: 8, 9] |
| `acel <cmd>` | Surrender the terminal to run interactive foreground tasks[cite: 8, 9] |
| `-bro!` / `bro` | Take a mandatory break via the random activity generator[cite: 8] |

---

# 📦 Installation & Strong Verification

### Prerequisites
* [Rust & Cargo](https://www.rust-lang.org/tools/install) (1.70 or higher)

### Building & Verifying Locally
ELIDE enforces strict static analysis to ensure bulletproof reliability:

```bash
# Clone the repository
git clone [https://github.com/tunglamtrannguyen3-cmd/ELIDE.git](https://github.com/tunglamtrannguyen3-cmd/ELIDE.git)
cd ELIDE

# Add clippy for strict lint verification
rustup component add clippy

# Run zero-warning clippy check
cargo clippy --all-targets -- -D warnings

# Build in optimized release mode
cargo build --release

# Run ELIDE
./target/release/elide

```

---

# 🥚 Easter Eggs & Mental Health

Software doesn't have to be a joyless corporate product. If you're questioning your life choices at 3 AM while debugging a segmentation fault, try typing:

```text
-bro!

```

Or use mood-adjuster commands: `--im-stuck`, `--im-bored`, `--im-broken`, or `--im-genius`.

---

# 📄 License

Distributed under the **Eggchese Open Project License 2026 (EOPL-2026.2)**. See [`LICENSE.md`](LICENSE.md) for details.

---