//! Target selection shared by the command-line tools: `-os`, `-cpu` and `-target` names, and
//! the `Options` they imply when a program is only checked.
use crate::sema::{Options, TargetCpu, TargetOs, long_double_for};

/// The names `-os` accepts.
pub const OS_NAMES: [&str; 4] = ["linux", "windows", "macos", "wasm"];

pub fn parse_os(name: &str) -> Option<TargetOs> {
    match name {
        "linux" => Some(TargetOs::Linux),
        "windows" => Some(TargetOs::Windows),
        "macos" => Some(TargetOs::MacOS),
        "wasm" => Some(TargetOs::Wasm),
        _ => None,
    }
}

/// Parse a `-cpu` value. `wasm` names the CPU of a WebAssembly target.
pub fn parse_cpu(name: &str) -> Option<TargetCpu> {
    match name {
        "x64" | "x86_64" => Some(TargetCpu::X64),
        "arm64" | "aarch64" => Some(TargetCpu::Arm64),
        "wasm" => Some(TargetCpu::Wasm),
        _ => None,
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

/// What a target triple such as `aarch64-pc-windows-msvc` says about `OS` and `CPU`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Triple {
    pub os: TargetOs,
    pub cpu: TargetCpu,
    /// A GNU environment (`-gnu`, MinGW), which gives `long double` its wide layout.
    pub gnu: bool,
}

impl Triple {
    pub fn parse(triple: &str) -> Result<Self, String> {
        let parts: Vec<&str> = triple.split('-').collect();
        let cpu = match parts[0] {
            "x86_64" | "x64" | "amd64" => TargetCpu::X64,
            "aarch64" | "arm64" => TargetCpu::Arm64,
            "wasm32" | "wasm64" => TargetCpu::Wasm,
            _ => return Err(format!("unsupported target architecture in `{triple}`")),
        };
        let os = if cpu == TargetCpu::Wasm {
            // `wasm64-unknown-wasi`, `wasm32-wasi`, `wasm32-unknown-unknown`.
            let system = parts.get(2).or(parts.get(1)).copied();
            if !matches!(system, Some("wasi" | "wasip1" | "wasip2" | "unknown")) {
                return Err(format!("unsupported WebAssembly target `{triple}`"));
            }
            TargetOs::Wasm
        } else if parts
            .iter()
            .any(|p| matches!(*p, "windows" | "mingw32" | "w64"))
        {
            TargetOs::Windows
        } else if parts
            .iter()
            .any(|p| p.starts_with("darwin") || p.starts_with("macos") || *p == "apple")
        {
            TargetOs::MacOS
        } else if parts.contains(&"linux") {
            TargetOs::Linux
        } else {
            return Err(format!("unsupported target OS in `{triple}`"));
        };
        let gnu = parts
            .iter()
            .any(|p| p.starts_with("gnu") || p.starts_with("mingw"));
        Ok(Triple {
            os,
            cpu,
            gnu,
        })
    }

    /// Make `options` target this triple.
    pub fn apply(self, options: &mut Options) {
        options.os = self.os;
        options.cpu = self.cpu;
        options.long_double = long_double_for(self.os, self.cpu, self.gnu);
    }
}

/// Apply `-os` and `-cpu` for checking only: that OS and CPU, with its usual C compiler. A
/// wasm target has its own CPU (`CUSTOM`), as a `-os wasm` build and the browser give it, and
/// Windows on another host defaults to x64 with the MinGW-style `long double`.
pub fn select_os_cpu(options: &mut Options, os: Option<TargetOs>, cpu: Option<TargetCpu>) {
    if os.is_none() && cpu.is_none() {
        return;
    }
    let host_os = options.os;
    options.os = os.unwrap_or(options.os);
    options.cpu = cpu.unwrap_or(match options.os {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triples_name_an_os_cpu_and_environment() {
        for (triple, os, cpu, gnu) in [
            (
                "x86_64-pc-windows-msvc",
                TargetOs::Windows,
                TargetCpu::X64,
                false,
            ),
            (
                "aarch64-w64-mingw32",
                TargetOs::Windows,
                TargetCpu::Arm64,
                true,
            ),
            (
                "x86_64-apple-darwin",
                TargetOs::MacOS,
                TargetCpu::X64,
                false,
            ),
            (
                "arm64-apple-macosx",
                TargetOs::MacOS,
                TargetCpu::Arm64,
                false,
            ),
            (
                "aarch64-unknown-linux-gnu",
                TargetOs::Linux,
                TargetCpu::Arm64,
                true,
            ),
            (
                "wasm32-unknown-unknown",
                TargetOs::Wasm,
                TargetCpu::Wasm,
                false,
            ),
            (
                "wasm64-unknown-wasi",
                TargetOs::Wasm,
                TargetCpu::Wasm,
                false,
            ),
        ] {
            assert_eq!(
                Triple::parse(triple),
                Ok(Triple {
                    os,
                    cpu,
                    gnu
                }),
                "{triple}"
            );
        }
        assert!(Triple::parse("riscv64-unknown-linux-gnu").is_err());
        assert!(Triple::parse("x86_64-unknown-freebsd").is_err());
    }
}
