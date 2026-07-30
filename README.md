# ELIDE 🚀
> A multi-featured, surprisingly lightweight Terminal User Interface (TUI) IDE.
ELIDE is built for speed and simplicity. It doesn't bloat your system with bundled compilers or runtime environments—instead, it leverages your existing system tools to give you a clean, fast terminal-based coding experience.
---
## 📋 Prerequisites
ELIDE relies on your system's existing toolchain to compile and run code. Before getting started, make sure you have:
* [Rust & Cargo](https://www.rust-lang.org/tools/install) installed.
* Compilers and tools for whichever programming languages you plan to use (e.g., `gcc`, `python`, `node`, etc.).
---
## 🛠️ Getting Started
Launch ELIDE directly from your shell using the following commands:

| Command | Action |
| :--- | :--- |
| `elide` | Opens the main ELIDE application interface. |
| `elide new <filename>` | Creates and opens a new file. |
| `elide code <filename>` | Opens an existing file for editing. |
| `elide switch <filename>` | Switches active editing buffer to another file. |

---
## ⌨️ Shortkey & In-App Commands
Once inside ELIDE, you can execute the following flags and actions:
* **`-c`** — Compile current file
* **`-d`** — Debug session
* **`-s`** — Save current file
* **`-i`** — Show file/project info
* **`-?`** or **`-h`** — Help menu
* **`-bro!`** — File complaining mode *(recommends a game when you need a break)*
* **Shell Commands** — You can also execute standard system shell commands directly within ELIDE.
---
## 🎨 Note on Interface
ELIDE is currently a **TUI (Terminal User Interface)** application to keep resource usage as low as possible. 
> 💡 *A standalone GUI version is planned for future releases!*
---
## 📜 License
Distributed under the MIT License. See `LICENSE` for more information.