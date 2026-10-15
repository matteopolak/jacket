//! Source-checking target selection. No backend/toolchain is needed to select compiler options.
use jaic::sema::{Options, TargetCpu, TargetOs, long_double_for};

#[derive(Clone, Debug, Default)]
pub struct Selection {
    pub os: Option<TargetOs>,
    pub cpu: Option<TargetCpu>,
    pub triple: Option<String>,
}

pub fn parse_os(value: &str) -> Result<TargetOs, String> {
    match value {
        "windows" => Ok(TargetOs::Windows),
        "linux" => Ok(TargetOs::Linux),
        "macos" => Ok(TargetOs::MacOS),
        "wasm" => Ok(TargetOs::Wasm),
        _ => Err(format!(
            "unknown OS `{value}` for `-os`\nhelp: use linux, windows, macos or wasm"
        )),
    }
}

pub fn parse_cpu(value: &str) -> Result<TargetCpu, String> {
    match value {
        "x64" | "x86_64" => Ok(TargetCpu::X64),
        "arm64" | "aarch64" => Ok(TargetCpu::Arm64),
        "wasm" => Ok(TargetCpu::Wasm),
        _ => Err(format!(
            "unknown CPU `{value}` for `-cpu`\nhelp: use x64 (x86_64) or arm64 (aarch64)"
        )),
    }
}

impl Selection {
    /// Match jaic's checking target defaults and triple precedence, preserving other options.
    pub fn apply(&self, options: &mut Options) -> Result<(), String> {
        if let Some(triple) = &self.triple {
            let parts: Vec<_> = triple.split('-').collect();
            let cpu = match parts.first().copied() {
                Some("x86_64") => TargetCpu::X64,
                Some("aarch64" | "arm64") => TargetCpu::Arm64,
                Some("wasm32" | "wasm64") => TargetCpu::Wasm,
                _ => return Err(format!("unsupported target architecture in `{triple}`")),
            };
            let os = if cpu == TargetCpu::Wasm {
                if !matches!(
                    parts.get(2).copied(),
                    Some("wasi" | "wasip1" | "wasip2" | "unknown")
                ) {
                    return Err(format!("unsupported WebAssembly target `{triple}`"));
                }
                TargetOs::Wasm
            } else if parts.iter().any(|p| matches!(*p, "windows" | "mingw32")) {
                TargetOs::Windows
            } else if parts
                .iter()
                .any(|p| p.starts_with("darwin") || p.starts_with("macos"))
            {
                TargetOs::MacOS
            } else if parts.contains(&"linux") {
                TargetOs::Linux
            } else {
                return Err(format!("unsupported target OS in `{triple}`"));
            };
            options.os = os;
            options.cpu = cpu;
            options.long_double = long_double_for(
                os,
                cpu,
                parts
                    .iter()
                    .any(|p| p.starts_with("gnu") || *p == "mingw32"),
            );
        } else if self.os.is_some() || self.cpu.is_some() {
            let host_os = options.os;
            options.os = self.os.unwrap_or(options.os);
            options.cpu = self.cpu.unwrap_or_else(|| match options.os {
                TargetOs::Wasm => TargetCpu::Wasm,
                TargetOs::Windows if host_os != TargetOs::Windows => TargetCpu::X64,
                _ => options.cpu,
            });
            options.long_double = long_double_for(
                options.os,
                options.cpu,
                options.os == TargetOs::Windows && host_os != TargetOs::Windows,
            );
        }
        Ok(())
    }
}

pub fn os_name(os: TargetOs) -> &'static str {
    match os {
        TargetOs::Windows => "windows",
        TargetOs::Linux => "linux",
        TargetOs::MacOS => "macos",
        TargetOs::Wasm => "wasm",
    }
}

pub fn cpu_name(cpu: TargetCpu) -> &'static str {
    match cpu {
        TargetCpu::X64 => "x64",
        TargetCpu::Arm64 => "arm64",
        TargetCpu::Wasm => "wasm",
    }
}
