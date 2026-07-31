use anyhow::Result;
use nix::sys::resource::{getrusage, UsageWho};
use std::process::Stdio;
use tokio::process::Command;
use crate::editor::Editor;

#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub success: bool,
    pub output: String,
    pub max_rss_kb: i64,
}

pub async fn run_bash_cmd(cmd_str: &str) -> Result<ProcessResult> {
    let child = Command::new("/bin/bash")
        .arg("-c")
        .arg(cmd_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output().await?;
    let success = output.status.success();

    let max_rss_kb = match getrusage(UsageWho::RUSAGE_CHILDREN) {
        Ok(usage) => usage.max_rss(),
        Err(_) => 0,
    };

    let text = if success {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    let formatted_output = if max_rss_kb > 0 {
        format!("{}\n[Kernel Stats: Peak RAM {} KB]", text.trim(), max_rss_kb)
    } else {
        text.trim().to_string()
    };

    Ok(ProcessResult {
        success,
        output: formatted_output,
        max_rss_kb,
    })
}

pub async fn compile_file(editor: &Editor) -> Result<ProcessResult> {
    if let Some(ref custom_cmd) = editor.custom_build_cmd {
        return run_bash_cmd(custom_cmd).await;
    }

    let filename = editor.filename.as_deref().unwrap_or("main.rs");
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    let stem = filename.strip_suffix(&format!(".{}", extension)).unwrap_or(filename);

    let build_command = match extension.as_str() {
        "rs" => format!("rustc {}", filename),
        "c" | "h" => format!("gcc {} -o out && ./out", filename),
        "cpp" | "cxx" | "cc" | "c++" | "hpp" | "hxx" | "hh" => format!("g++ {} -o out && ./out", filename),
        "d" => format!("dmd -of=out {} && ./out", filename),
        "go" => format!("go run {}", filename),
        "nim" => format!("nim c -r {}", filename),
        "v" => format!("v run {}", filename),
        "zig" => format!("zig run {}", filename),
        "asm" | "s" | "S" => format!("nasm -f elf64 {} -o {}.o && ld -o out {}.o && ./out", filename, stem, stem),
        "py" | "pyw" | "py3" => format!("python3 {}", filename),
        "rb" | "ruby" => format!("ruby {}", filename),
        "pl" | "pm" => format!("perl {}", filename),
        "php" => format!("php {}", filename),
        "lua" => format!("lua {}", filename),
        "r" | "rmd" => format!("Rscript {}", filename),
        "sh" | "bash" | "zsh" | "ash" => format!("bash {}", filename),
        "fish" => format!("fish {}", filename),
        "js" | "mjs" | "cjs" | "jsx" => format!("node {}", filename),
        "ts" | "mts" | "cts" | "tsx" => format!("ts-node {}", filename),
        "java" => {
            let class_name = filename.strip_suffix(".java").unwrap_or(filename);
            format!("javac {} && java {}", filename, class_name)
        }
        "kt" | "kts" => format!("kotlinc {} -include-runtime -d out.jar && java -jar out.jar", filename),
        "scala" | "sc" => format!("scala {}", filename),
        "hs" | "lhs" => format!("runhaskell {}", filename),
        "ml" | "mli" => format!("ocaml {}", filename),
        "ex" | "exs" => format!("elixir {}", filename),
        "erl" | "hrl" => format!("escript {}", filename),
        "clj" | "cljs" | "cljc" => format!("clojure {}", filename),
        "lisp" | "lsp" => format!("sbcl --script {}", filename),
        "scm" | "ss" => format!("scheme --script {}", filename),
        "bf" | "b" => format!("brainfuck {}", filename),
        _ => return Ok(ProcessResult {
            success: false,
            output: format!("Unsupported extension '.{}'. Use 'set-build <cmd>' or 'sh <cmd>' in Alt+T", extension),
            max_rss_kb: 0,                                }),
    };
                                                      run_bash_cmd(&build_command).await
}
