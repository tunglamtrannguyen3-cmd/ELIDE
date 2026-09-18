// Replace the contents of tracer.rs with this:
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct KernelTracer {
    pub active: bool,
}

impl KernelTracer {
    pub fn init() -> Self {
        // Clear the log file on startup
        let _ = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("elide_debug.log");
            
        Self { active: true }
    }

    pub fn log_event(&self, event: &str) {
        if self.active {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("elide_debug.log") {
                let _ = writeln!(file, "[DEBUG] {}", event);
            }
        }
    }
}