use anyhow::Result;
use aya::Ebpf;

pub struct KernelTracer {
    _bpf: Option<Ebpf>,
}

impl KernelTracer {
    pub fn init() -> Result<Self> {
        if !nix::unistd::Uid::effective().is_root() {
            anyhow::bail!("eBPF tracing requires root or CAP_BPF permissions.");
        }
        Ok(Self { _bpf: None })
    }
}

