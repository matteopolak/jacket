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

/// The `Operating_System_Tag` member that names `os` (`OS`, `os_target`, `#module_parameters`
/// assertions).
pub fn os_member(os: TargetOs) -> &'static str {
    match os {
        TargetOs::Windows => "WINDOWS",
        TargetOs::Linux => "LINUX",
        TargetOs::MacOS => "MACOS",
        TargetOs::Wasm => "WASM",
    }
}

/// The `CPU_Tag` member that names `cpu`. A wasm target's CPU is `.CUSTOM`.
pub fn cpu_member(cpu: TargetCpu) -> &'static str {
    match cpu {
        TargetCpu::X64 => "X64",
        TargetCpu::Arm64 => "ARM64",
        TargetCpu::Wasm => "CUSTOM",
    }
}

/// The C environment a target's code is built for and linked against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetAbi {
    /// Windows with the Microsoft toolchain and CRT.
    Msvc,
    /// Windows with MinGW-w64 (`-gnu`, `mingw32`), or Linux with glibc.
    Gnu,
    /// Linux with musl.
    Musl,
    /// macOS (Mach-O, Apple's frame-record and linker conventions).
    Apple,
    /// A WebAssembly System Interface command.
    Wasi,
    /// Nothing named: freestanding WebAssembly, or a Linux environment jaic has no name for.
    Unknown,
}

/// What a program is built for: OS, CPU and C environment, parsed once from `-os`/`-cpu`,
/// `-target` or a workspace's build options and passed to everything that depends on it.
/// `Display` writes the LLVM triple.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub os: TargetOs,
    pub cpu: TargetCpu,
    pub abi: TargetAbi,
    /// A `wasm32` target (32-bit memory); every other target, `wasm64` included, has 8-byte
    /// pointers.
    pub memory32: bool,
}

impl Target {
    /// The OS and CPU with the usual environment for it: MSVC on Windows, glibc on Linux, WASI on
    /// wasm.
    pub fn new(os: TargetOs, cpu: TargetCpu) -> Target {
        let abi = match os {
            TargetOs::Windows => TargetAbi::Msvc,
            TargetOs::Linux => TargetAbi::Gnu,
            TargetOs::MacOS => TargetAbi::Apple,
            TargetOs::Wasm => TargetAbi::Wasi,
        };
        Target {
            os,
            cpu,
            abi,
            memory32: false,
        }
    }

    /// The platform this compiler runs on.
    pub fn host() -> Target {
        let host = Options::host();
        let mut target = Target::new(host.os, host.cpu);
        if cfg!(target_env = "gnu") && host.os == TargetOs::Windows {
            target.abi = TargetAbi::Gnu;
        } else if cfg!(target_env = "musl") && host.os == TargetOs::Linux {
            target.abi = TargetAbi::Musl;
        }
        target
    }

    /// A Windows build for `cpu`: the MSVC environment on a Windows host, MinGW-w64 (whose
    /// cross toolchains exist for macOS and Linux) elsewhere.
    pub fn windows(cpu: TargetCpu, msvc: bool) -> Target {
        Target {
            abi: if msvc {
                TargetAbi::Msvc
            } else {
                TargetAbi::Gnu
            },
            ..Target::new(TargetOs::Windows, cpu)
        }
    }

    /// The WebAssembly target `-os wasm` builds for: a WASI command with 64-bit memory.
    pub fn wasi() -> Target {
        Target::new(TargetOs::Wasm, TargetCpu::Wasm)
    }

    /// The WebAssembly target of a workspace that names no triple: a module the host calls
    /// through its exports.
    pub fn wasm_module() -> Target {
        Target {
            abi: TargetAbi::Unknown,
            ..Target::wasi()
        }
    }

    /// What a target triple such as `aarch64-pc-windows-msvc` says about the target.
    pub fn parse(triple: &str) -> Result<Target, String> {
        let parts: Vec<&str> = triple.split('-').collect();
        let (cpu, memory32) = match parts[0] {
            "x86_64" | "x64" | "amd64" => (TargetCpu::X64, false),
            "aarch64" | "arm64" => (TargetCpu::Arm64, false),
            "wasm32" => (TargetCpu::Wasm, true),
            "wasm64" => (TargetCpu::Wasm, false),
            _ => return Err(format!("unsupported target architecture in `{triple}`")),
        };
        let gnu = parts
            .iter()
            .any(|p| p.starts_with("gnu") || p.starts_with("mingw"));
        let (os, abi) = if cpu == TargetCpu::Wasm {
            // `wasm64-unknown-wasi`, `wasm32-wasi`, `wasm32-unknown-unknown`.
            let system = parts.get(2).or(parts.get(1)).copied();
            let abi = match system {
                Some("wasi" | "wasip1" | "wasip2") => TargetAbi::Wasi,
                Some("unknown") => TargetAbi::Unknown,
                _ => return Err(format!("unsupported WebAssembly target `{triple}`")),
            };
            (TargetOs::Wasm, abi)
        } else if parts
            .iter()
            .any(|p| matches!(*p, "windows" | "mingw32" | "w64"))
        {
            (
                TargetOs::Windows,
                if gnu {
                    TargetAbi::Gnu
                } else {
                    TargetAbi::Msvc
                },
            )
        } else if parts
            .iter()
            .any(|p| p.starts_with("darwin") || p.starts_with("macos") || *p == "apple")
        {
            (TargetOs::MacOS, TargetAbi::Apple)
        } else if parts.contains(&"linux") {
            let abi = if gnu {
                TargetAbi::Gnu
            } else if parts.iter().any(|p| p.starts_with("musl")) {
                TargetAbi::Musl
            } else {
                TargetAbi::Unknown
            };
            (TargetOs::Linux, abi)
        } else {
            return Err(format!("unsupported target OS in `{triple}`"));
        };
        Ok(Target {
            os,
            cpu,
            abi,
            memory32,
        })
    }

    /// Make `options` target this.
    pub fn apply(self, options: &mut Options) {
        options.os = self.os;
        options.cpu = self.cpu;
        options.long_double = long_double_for(self.os, self.cpu, self.abi == TargetAbi::Gnu);
    }

    pub fn is_windows(self) -> bool {
        self.os == TargetOs::Windows
    }

    pub fn is_wasm(self) -> bool {
        self.os == TargetOs::Wasm
    }

    pub fn is_apple(self) -> bool {
        self.os == TargetOs::MacOS
    }

    /// The calling-convention family for native code on this target; `None` for a 32-bit
    /// WebAssembly target.
    pub fn arch(self) -> Option<crate::abi::Arch> {
        use crate::abi::Arch;
        match (self.cpu, self.os) {
            (TargetCpu::Arm64, TargetOs::Windows) => Some(Arch::Win64Arm),
            (TargetCpu::Arm64, _) => Some(Arch::Aarch64),
            (TargetCpu::X64, TargetOs::Windows) => Some(Arch::Win64),
            (TargetCpu::X64, _) => Some(Arch::X86_64),
            (TargetCpu::Wasm, _) if !self.memory32 => Some(Arch::Wasm64),
            (TargetCpu::Wasm, _) => None,
        }
    }
}

impl std::str::FromStr for Target {
    type Err = String;

    fn from_str(triple: &str) -> Result<Self, String> {
        Target::parse(triple)
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arch = match self.cpu {
            TargetCpu::X64 => "x86_64",
            TargetCpu::Arm64 => "aarch64",
            TargetCpu::Wasm if self.memory32 => return write!(f, "wasm32-{}", self.wasm_system()),
            TargetCpu::Wasm => return write!(f, "wasm64-{}", self.wasm_system()),
        };
        let rest = match (self.os, self.abi) {
            (TargetOs::Windows, TargetAbi::Gnu) => "pc-windows-gnu",
            (TargetOs::Windows, _) => "pc-windows-msvc",
            (TargetOs::MacOS, _) => "apple-darwin",
            (TargetOs::Linux, TargetAbi::Gnu) => "unknown-linux-gnu",
            (TargetOs::Linux, TargetAbi::Musl) => "unknown-linux-musl",
            (TargetOs::Linux, _) => "unknown-linux",
            (TargetOs::Wasm, _) => unreachable!("a wasm OS has a wasm CPU"),
        };
        write!(f, "{arch}-{rest}")
    }
}

impl Target {
    fn wasm_system(&self) -> &'static str {
        if self.abi == TargetAbi::Wasi {
            "unknown-wasi"
        } else {
            "unknown-unknown"
        }
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
    fn names_round_trip() {
        for os in [
            TargetOs::Windows,
            TargetOs::Linux,
            TargetOs::MacOS,
            TargetOs::Wasm,
        ] {
            assert_eq!(parse_os(os_name(os)), Some(os));
        }
        for cpu in [TargetCpu::X64, TargetCpu::Arm64, TargetCpu::Wasm] {
            assert_eq!(parse_cpu(cpu_name(cpu)), Some(cpu));
        }
    }

    #[test]
    fn triples_name_an_os_cpu_and_environment() {
        for (triple, os, cpu, abi, text) in [
            (
                "x86_64-pc-windows-msvc",
                TargetOs::Windows,
                TargetCpu::X64,
                TargetAbi::Msvc,
                "x86_64-pc-windows-msvc",
            ),
            (
                "aarch64-w64-mingw32",
                TargetOs::Windows,
                TargetCpu::Arm64,
                TargetAbi::Gnu,
                "aarch64-pc-windows-gnu",
            ),
            (
                "x86_64-apple-darwin",
                TargetOs::MacOS,
                TargetCpu::X64,
                TargetAbi::Apple,
                "x86_64-apple-darwin",
            ),
            (
                "arm64-apple-macosx",
                TargetOs::MacOS,
                TargetCpu::Arm64,
                TargetAbi::Apple,
                "aarch64-apple-darwin",
            ),
            (
                "aarch64-unknown-linux-gnu",
                TargetOs::Linux,
                TargetCpu::Arm64,
                TargetAbi::Gnu,
                "aarch64-unknown-linux-gnu",
            ),
            (
                "x86_64-unknown-linux-musl",
                TargetOs::Linux,
                TargetCpu::X64,
                TargetAbi::Musl,
                "x86_64-unknown-linux-musl",
            ),
            (
                "wasm32-unknown-unknown",
                TargetOs::Wasm,
                TargetCpu::Wasm,
                TargetAbi::Unknown,
                "wasm32-unknown-unknown",
            ),
            (
                "wasm64-unknown-wasi",
                TargetOs::Wasm,
                TargetCpu::Wasm,
                TargetAbi::Wasi,
                "wasm64-unknown-wasi",
            ),
        ] {
            let target = Target::parse(triple).unwrap();
            assert_eq!(
                (target.os, target.cpu, target.abi),
                (os, cpu, abi),
                "{triple}"
            );
            assert_eq!(target.to_string(), text, "{triple}");
            assert_eq!(Target::parse(text), Ok(target), "{triple}");
        }
        assert!(Target::parse("riscv64-unknown-linux-gnu").is_err());
        assert!(Target::parse("x86_64-unknown-freebsd").is_err());
    }

    #[test]
    fn calling_conventions_follow_the_target() {
        use crate::abi::Arch;
        for (triple, arch) in [
            ("x86_64-pc-windows-msvc", Some(Arch::Win64)),
            ("x86_64-w64-windows-gnu", Some(Arch::Win64)),
            ("aarch64-pc-windows-gnu", Some(Arch::Win64Arm)),
            ("aarch64-unknown-linux-gnu", Some(Arch::Aarch64)),
            ("x86_64-unknown-linux-gnu", Some(Arch::X86_64)),
            ("wasm64-unknown-unknown", Some(Arch::Wasm64)),
            ("wasm32-unknown-unknown", None),
        ] {
            assert_eq!(Target::parse(triple).unwrap().arch(), arch, "{triple}");
        }
    }
}
