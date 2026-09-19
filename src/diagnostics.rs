// diagnostics.rs
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub message: String,
    pub severity: Option<u8>, // Add this to capture the LSP severity integer
}