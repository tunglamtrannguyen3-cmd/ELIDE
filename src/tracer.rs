#[derive(Debug, Clone)]
pub struct KernelTracer {
    #[allow(dead_code)]
    pub active: bool,
}

impl KernelTracer {
    pub fn init() -> Self {
        Self { active: true }
    }

    #[allow(dead_code)]
    pub fn log_event(&self, event: &str) {
        if self.active {
            let _ = event;
        }
    }
}
