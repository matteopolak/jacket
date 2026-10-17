//! Source-checking target selection. No backend/toolchain is needed to select compiler options.
use jaic::sema::{Options, TargetCpu, TargetOs};
pub use jaic::target::{cpu_name, os_name};

#[derive(Clone, Debug, Default)]
pub struct Selection {
    pub os: Option<TargetOs>,
    pub cpu: Option<TargetCpu>,
    pub triple: Option<String>,
}

pub fn parse_os(value: &str) -> Result<TargetOs, String> {
    jaic::target::parse_os(value).ok_or_else(|| {
        format!("unknown OS `{value}` for `-os`\nhelp: use linux, windows, macos or wasm")
    })
}

pub fn parse_cpu(value: &str) -> Result<TargetCpu, String> {
    jaic::target::parse_cpu(value).ok_or_else(|| {
        format!("unknown CPU `{value}` for `-cpu`\nhelp: use x64 (x86_64) or arm64 (aarch64)")
    })
}

impl Selection {
    /// Match jaic's checking target defaults and triple precedence, preserving other options.
    pub fn apply(&self, options: &mut Options) -> Result<(), String> {
        match &self.triple {
            Some(triple) => jaic::target::Target::parse(triple)?.apply(options),
            None => jaic::target::select_os_cpu(options, self.os, self.cpu),
        }
        Ok(())
    }
}
