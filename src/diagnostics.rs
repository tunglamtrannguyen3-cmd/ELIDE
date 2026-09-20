// diagnostics.rs
#[derive(Debug, Clone)]
pub struct Diagnostic {
    
    pub line: usize,
    pub message: String,
    pub severity: Option<u8>,
}