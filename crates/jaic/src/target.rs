//! Target selection shared by the command-line tools: `-os`, `-cpu` and `-target` names, and
//! the `Options` they imply when a program is only checked.
use crate::layout::DataLayout;
use crate::sema::{Options, long_double_for};

/// The operating system a program is built for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetOs {
    Windows,
    Linux,
    MacOS,
    Wasm,
    /// No operating system: a freestanding program with no libc and no system entry point
    /// (`Operating_System_Tag.NONE`).
    None,
}

/// The CPU a program is built for (`Wasm` is the CPU of a WebAssembly target).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetCpu {
    X64,
    Arm64,
    /// 32-bit x86 (i686).
    X86,
    /// IBM z/Architecture (s390x), 64-bit big-endian.
    S390x,
    /// 32-bit ARM (armv7-a) with hard-float AAPCS.
    Arm,
    /// 8-bit AVR with 16-bit pointers (Harvard: code and data are separate address spaces).
    Avr,
    Wasm,
}

/// The names `-os` accepts.
pub const OS_NAMES: [&str; 5] = ["linux", "windows", "macos", "wasm", "none"];

pub fn parse_os(name: &str) -> Option<TargetOs> {
    match name {
        "linux" => Some(TargetOs::Linux),
        "windows" => Some(TargetOs::Windows),
        "macos" => Some(TargetOs::MacOS),
        "wasm" => Some(TargetOs::Wasm),
        "none" => Some(TargetOs::None),
        _ => None,
    }
}

/// Parse a `-cpu` value. `wasm` names the CPU of a WebAssembly target.
pub fn parse_cpu(name: &str) -> Option<TargetCpu> {
    match name {
        "x64" | "x86_64" => Some(TargetCpu::X64),
        "arm64" | "aarch64" => Some(TargetCpu::Arm64),
        "x86" | "i686" => Some(TargetCpu::X86),
        "s390x" => Some(TargetCpu::S390x),
        "arm" | "armv7" | "armhf" => Some(TargetCpu::Arm),
        "avr" => Some(TargetCpu::Avr),
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
        TargetOs::None => "none",
    }
}

pub fn cpu_name(cpu: TargetCpu) -> &'static str {
    match cpu {
        TargetCpu::X64 => "x64",
        TargetCpu::Arm64 => "arm64",
        TargetCpu::X86 => "x86",
        TargetCpu::S390x => "s390x",
        TargetCpu::Arm => "arm",
        TargetCpu::Avr => "avr",
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
        TargetOs::None => "NONE",
    }
}

/// The `CPU_Tag` member that names `cpu`. A wasm target's CPU is `.CUSTOM`.
pub fn cpu_member(cpu: TargetCpu) -> &'static str {
    match cpu {
        TargetCpu::X64 => "X64",
        TargetCpu::Arm64 => "ARM64",
        TargetCpu::X86 | TargetCpu::S390x | TargetCpu::Arm | TargetCpu::Avr | TargetCpu::Wasm => {
            "CUSTOM"
        }
    }
}

/// The `Architecture` member that names a target's CPU family, which `CPU_Tag` cannot: every
/// CPU the spec enum lacks is `.CUSTOM`. A WebAssembly target's memory width tells wasm32 from
/// wasm64.
pub fn architecture_member(cpu: TargetCpu, pointer_bytes: u32) -> &'static str {
    match (cpu, pointer_bytes) {
        (TargetCpu::X64, _) => "X64",
        (TargetCpu::Arm64, _) => "ARM64",
        (TargetCpu::X86, _) => "X86",
        (TargetCpu::S390x, _) => "S390X",
        (TargetCpu::Arm, _) => "ARM",
        (TargetCpu::Avr, _) => "AVR",
        (TargetCpu::Wasm, 4) => "WASM32",
        (TargetCpu::Wasm, _) => "WASM64",
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
    /// Nothing named: freestanding WebAssembly, bare metal (`*-none-elf`), or a Linux
    /// environment jaic has no name for.
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
            TargetOs::None => TargetAbi::Unknown,
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
            "i686" | "i586" | "i486" | "i386" => (TargetCpu::X86, false),
            "s390x" => (TargetCpu::S390x, false),
            "arm" | "armv7" | "armv7a" | "armv7l" => (TargetCpu::Arm, false),
            "avr" => (TargetCpu::Avr, false),
            "wasm32" => (TargetCpu::Wasm, true),
            "wasm64" => (TargetCpu::Wasm, false),
            _ => return Err(format!("unsupported target architecture in `{triple}`")),
        };
        let gnu = parts
            .iter()
            .any(|p| p.starts_with("gnu") || p.starts_with("mingw"));
        if matches!(cpu, TargetCpu::X86 | TargetCpu::S390x | TargetCpu::Arm)
            && !parts.contains(&"linux")
        {
            return Err(format!(
                "unsupported target OS in `{triple}`: {} is built for Linux only",
                match cpu {
                    TargetCpu::X86 => "32-bit x86",
                    TargetCpu::Arm => "32-bit ARM",
                    _ => "s390x",
                }
            ));
        }
        if cpu == TargetCpu::Arm && !parts.iter().any(|p| p.ends_with("eabihf")) {
            return Err(format!(
                "unsupported target environment in `{triple}`: 32-bit ARM is built for hard-float (`gnueabihf`, `musleabihf`)"
            ));
        }
        let (os, abi) = if cpu == TargetCpu::Wasm {
            // `wasm64-unknown-wasi`, `wasm32-wasi`, `wasm32-unknown-unknown`.
            let system = parts.get(2).or(parts.get(1)).copied();
            let abi = match system {
                Some("wasi" | "wasip1" | "wasip2") => TargetAbi::Wasi,
                Some("unknown") => TargetAbi::Unknown,
                _ => return Err(format!("unsupported WebAssembly target `{triple}`")),
            };
            (TargetOs::Wasm, abi)
        } else if cpu == TargetCpu::Avr {
            // `avr-none`, `avr-none-elf`: bare metal, the only kind of AVR target.
            if !parts.contains(&"none") {
                return Err(format!(
                    "unsupported target OS in `{triple}`: AVR is built bare metal (`avr-none`)"
                ));
            }
            (TargetOs::None, TargetAbi::Unknown)
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

    /// How this target lays out data. The one place a layout is chosen. `wasm32` has 4-byte
    /// pointers; the rest are 64-bit little-endian.
    pub fn data_layout(self) -> DataLayout {
        let mut layout = DataLayout::lp64_little();
        if self.cpu == TargetCpu::Wasm && self.memory32 {
            layout.pointer_size = 4;
            layout.pointer_align = 4;
            layout.c.long = 4;
            layout.c.size_t = 4;
        }
        if self.cpu == TargetCpu::X86 {
            // System V i386: `s64` and `float64` align to 4, and `long double` is 12 bytes.
            layout.pointer_size = 4;
            layout.pointer_align = 4;
            layout.int_align = [1, 2, 4, 4];
            layout.float_align = [4, 4];
            layout.f80_size = 12;
            layout.f80_align = 4;
            layout.c.long = 4;
            layout.c.size_t = 4;
        }
        if self.cpu == TargetCpu::Arm {
            // AAPCS: 4-byte pointers and `long`, 8-byte `s64` and `float64` alignment, and a
            // `long double` that is a plain `double`.
            layout.pointer_size = 4;
            layout.pointer_align = 4;
            layout.c.long = 4;
            layout.c.size_t = 4;
        }
        if self.cpu == TargetCpu::Avr {
            // avr-gcc: 16-bit pointers, `int` and `size_t`, and no alignment padding at all.
            layout.pointer_size = 2;
            layout.pointer_align = 1;
            layout.int_align = [1, 1, 1, 1];
            layout.float_align = [1, 1];
            layout.stack_align = 1;
            layout.c.int = 2;
            layout.c.long = 4;
            layout.c.size_t = 2;
        }
        if self.cpu == TargetCpu::S390x {
            // z/Architecture ELF: big-endian, `long double` is a 16-byte binary128 aligned to 8.
            layout.endian = crate::layout::Endian::Big;
        }
        // LLP64: `long` stays 32 bits on 64-bit Windows.
        if self.os == TargetOs::Windows {
            layout.c.long = 4;
        }
        layout
    }

    /// Make `options` target this.
    pub fn apply(self, options: &mut Options) {
        options.os = self.os;
        options.cpu = self.cpu;
        options.layout = self.data_layout();
        options.long_double = long_double_for(self.os, self.cpu, self.abi == TargetAbi::Gnu);
    }

    /// The C environment this target is built for and linked against.
    pub fn toolchain(self) -> &'static crate::toolchain::Toolchain {
        crate::toolchain::Toolchain::for_target(self)
    }

    pub fn is_windows(self) -> bool {
        self.os == TargetOs::Windows
    }

    pub fn is_wasm(self) -> bool {
        self.os == TargetOs::Wasm
    }

    /// A bare-metal target: no operating system, libc, threads or system entry point.
    pub fn is_freestanding(self) -> bool {
        self.os == TargetOs::None
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
            (TargetCpu::X86, _) => Some(Arch::X86),
            (TargetCpu::S390x, _) => Some(Arch::S390x),
            (TargetCpu::Arm, _) => Some(Arch::Arm32),
            (TargetCpu::Avr, _) => Some(Arch::Avr),
            (TargetCpu::Wasm, _) => Some(Arch::Wasm),
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
            TargetCpu::X86 => "i686",
            TargetCpu::S390x => "s390x",
            TargetCpu::Arm => "armv7",
            TargetCpu::Avr => return write!(f, "avr-none-elf"),
            TargetCpu::Wasm if self.memory32 => return write!(f, "wasm32-{}", self.wasm_system()),
            TargetCpu::Wasm => return write!(f, "wasm64-{}", self.wasm_system()),
        };
        let rest = match (self.os, self.abi) {
            (TargetOs::Windows, TargetAbi::Gnu) => "pc-windows-gnu",
            (TargetOs::Windows, _) => "pc-windows-msvc",
            (TargetOs::MacOS, _) => "apple-darwin",
            (TargetOs::Linux, TargetAbi::Gnu) if self.cpu == TargetCpu::Arm => {
                "unknown-linux-gnueabihf"
            }
            (TargetOs::Linux, TargetAbi::Musl) if self.cpu == TargetCpu::Arm => {
                "unknown-linux-musleabihf"
            }
            (TargetOs::Linux, TargetAbi::Gnu) => "unknown-linux-gnu",
            (TargetOs::Linux, TargetAbi::Musl) => "unknown-linux-musl",
            (TargetOs::Linux, _) => "unknown-linux",
            (TargetOs::None, _) => "none-elf",
            (TargetOs::Wasm, _) => unreachable!("a wasm OS has a wasm CPU"),
        };
        write!(f, "{arch}-{rest}")
    }
}

impl Target {
    fn wasm_system(self) -> &'static str {
        if self.abi == TargetAbi::Wasi {
            "unknown-wasi"
        } else {
            "unknown-unknown"
        }
    }
}

/// The target the `-os`, `-cpu` and `-target` flags of a checking tool select.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub os: Option<TargetOs>,
    pub cpu: Option<TargetCpu>,
    pub triple: Option<String>,
}

impl Selection {
    /// Resolve a command-line build target. Checking and interpretation permit any OS/CPU;
    /// native cross builds without a triple support Windows and WASI only.
    pub fn build_target(
        &self,
        host_os: TargetOs,
        host_cpu: TargetCpu,
        native: bool,
    ) -> Result<Option<Target>, String> {
        if let Some(triple) = &self.triple {
            return Target::parse(triple).map(Some);
        }
        let os = self.os.unwrap_or(host_os);
        let cpu = self.cpu.unwrap_or(host_cpu);
        if os == host_os && cpu == host_cpu {
            return Ok(None);
        }
        match os {
            TargetOs::Windows => Ok(Some(Target::windows(
                self.cpu.unwrap_or(if host_os == TargetOs::Windows { host_cpu } else { TargetCpu::X64 }),
                host_os == TargetOs::Windows,
            ))),
            _ if !native => Ok(None),
            TargetOs::Wasm => Ok(Some(Target::wasi())),
            _ => Err("native cross-compilation is only supported for -os windows and -os wasm; pass -target <triple> for others".into()),
        }
    }

    /// Apply the compiler's effective command-line target, including native build defaults.
    pub fn apply_for_mode(&self, options: &mut Options, native: bool) -> Result<(), String> {
        match self.build_target(options.os, options.cpu, native)? {
            Some(target) => target.apply(options),
            None => select_os_cpu(options, self.os, self.cpu),
        }
        Ok(())
    }

    /// Apply the selection to `options` as `jaic` would for the same flags: a triple wins over
    /// the OS and CPU; otherwise [`select_os_cpu`]. Other options are kept.
    pub fn apply(&self, options: &mut Options) -> Result<(), String> {
        match &self.triple {
            Some(triple) => Target::parse(triple)?.apply(options),
            None => select_os_cpu(options, self.os, self.cpu),
        }
        Ok(())
    }
}

/// Apply `-os` and `-cpu` for checking only: that OS and CPU, with its usual C compiler. A
/// wasm target has its own CPU (`CUSTOM`), as a `-os wasm` build and the browser give it, and
/// any other OS than the host's defaults to x64 (Windows with the MinGW-style `long double`).
/// This is the one place that default is decided: the linter, the language server and the
/// command line pass no CPU of their own.
pub fn select_os_cpu(options: &mut Options, os: Option<TargetOs>, cpu: Option<TargetCpu>) {
    if os.is_none() && cpu.is_none() {
        return;
    }
    let host_os = options.os;
    options.os = os.unwrap_or(options.os);
    options.cpu = match options.os {
        TargetOs::Wasm => TargetCpu::Wasm,
        os => cpu.unwrap_or(if os == host_os {
            options.cpu
        } else {
            TargetCpu::X64
        }),
    };
    options.layout = Target {
        memory32: false,
        ..Target::new(options.os, options.cpu)
    }
    .data_layout();
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
    fn effective_mode_targets_match_host_and_cross_defaults() {
        for host_cpu in [TargetCpu::X64, TargetCpu::Arm64] {
            let mut host = Options::host();
            host.os = TargetOs::MacOS;
            host.cpu = host_cpu;
            for (selection, native, os, cpu) in [
                (Selection::default(), true, TargetOs::MacOS, host_cpu),
                (
                    Selection {
                        os: Some(TargetOs::MacOS),
                        ..Selection::default()
                    },
                    false,
                    TargetOs::MacOS,
                    host_cpu,
                ),
                (
                    Selection {
                        os: Some(TargetOs::Linux),
                        ..Selection::default()
                    },
                    false,
                    TargetOs::Linux,
                    TargetCpu::X64,
                ),
                (
                    Selection {
                        os: Some(TargetOs::Windows),
                        ..Selection::default()
                    },
                    true,
                    TargetOs::Windows,
                    TargetCpu::X64,
                ),
                (
                    Selection {
                        os: Some(TargetOs::Wasm),
                        ..Selection::default()
                    },
                    true,
                    TargetOs::Wasm,
                    TargetCpu::Wasm,
                ),
                (
                    Selection {
                        os: Some(TargetOs::Wasm),
                        cpu: Some(TargetCpu::Arm64),
                        ..Selection::default()
                    },
                    false,
                    TargetOs::Wasm,
                    TargetCpu::Wasm,
                ),
                (
                    Selection {
                        os: Some(TargetOs::Wasm),
                        cpu: Some(TargetCpu::X64),
                        ..Selection::default()
                    },
                    false,
                    TargetOs::Wasm,
                    TargetCpu::Wasm,
                ),
                (
                    Selection {
                        os: Some(TargetOs::Windows),
                        cpu: Some(TargetCpu::Arm64),
                        triple: Some("wasm64-unknown-wasi".into()),
                    },
                    true,
                    TargetOs::Wasm,
                    TargetCpu::Wasm,
                ),
            ] {
                let mut options = host.clone();
                selection.apply_for_mode(&mut options, native).unwrap();
                assert_eq!((options.os, options.cpu), (os, cpu));
            }
            let invalid = Selection {
                triple: Some("not-a-target".into()),
                ..Selection::default()
            };
            assert!(invalid.apply_for_mode(&mut host, true).is_err());
        }
    }

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
    fn another_os_defaults_to_x64_and_the_hosts_os_keeps_its_cpu() {
        let mut host = Options::host();
        host.cpu = TargetCpu::Arm64;
        let other = if host.os == TargetOs::Linux {
            TargetOs::MacOS
        } else {
            TargetOs::Linux
        };
        let mut options = host.clone();
        select_os_cpu(&mut options, Some(other), None);
        assert_eq!((options.os, options.cpu), (other, TargetCpu::X64));
        let mut options = host.clone();
        select_os_cpu(&mut options, Some(host.os), None);
        assert_eq!(options.cpu, TargetCpu::Arm64);
        let mut options = host.clone();
        select_os_cpu(&mut options, Some(TargetOs::Wasm), None);
        assert_eq!(options.cpu, TargetCpu::Wasm);
        let mut options = host;
        select_os_cpu(&mut options, Some(other), Some(TargetCpu::Arm64));
        assert_eq!(options.cpu, TargetCpu::Arm64);
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
            ("wasm64-unknown-unknown", Some(Arch::Wasm)),
            ("wasm32-unknown-unknown", Some(Arch::Wasm)),
        ] {
            assert_eq!(Target::parse(triple).unwrap().arch(), arch, "{triple}");
        }
    }

    /// `i686` is 32-bit x86 on Linux with 4-byte pointers, `s64` and `float64` aligned to 4 and
    /// a 12-byte `long double`; any other OS is refused.
    #[test]
    fn i686_linux_is_a_32_bit_x86_target() {
        for triple in [
            "i686-unknown-linux-gnu",
            "i686-unknown-linux-musl",
            "i386-linux-gnu",
        ] {
            let target = Target::parse(triple).unwrap();
            assert_eq!((target.os, target.cpu), (TargetOs::Linux, TargetCpu::X86));
            let layout = target.data_layout();
            assert_eq!(
                (layout.pointer_size, layout.c.long, layout.c.size_t),
                (4, 4, 4)
            );
            assert_eq!((layout.int_align_of(8), layout.float_align_of(8)), (4, 4));
            assert_eq!((layout.f80_size, layout.f80_align), (12, 4));
        }
        assert_eq!(
            Target::parse("i686-unknown-linux-musl")
                .unwrap()
                .to_string(),
            "i686-unknown-linux-musl"
        );
        assert!(Target::parse("i686-pc-windows-msvc").is_err());
        assert!(Target::parse("i686-apple-darwin").is_err());
    }
}
