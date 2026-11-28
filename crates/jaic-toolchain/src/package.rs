//! What a missing piece is, and the one command that gets it on each system.
//!
//! This table is the only place install advice lives. Jacket never runs these commands and never
//! offers to: an error note prints one.
use jaic::target::Target;

use crate::host::Os;
use crate::lookup::Lookup;

/// A system package manager jacket knows the names of packages for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageManager {
    /// Homebrew.
    Brew,
    /// `apt`, for Debian and Ubuntu.
    Apt,
    /// `dnf`, for Fedora.
    Dnf,
    /// `pacman`, for Arch.
    Pacman,
    /// `winget`, for Windows.
    Winget,
}

impl PackageManager {
    /// The manager of `host`, from the tools on it: `winget` on Windows, `brew` on macOS, and on
    /// Linux the first of `apt-get`, `dnf`, `pacman`, `brew`.
    pub fn detect(lookup: &Lookup) -> Option<PackageManager> {
        let candidates: &[(&str, PackageManager)] = match lookup.host.os() {
            Os::Windows => &[("winget", PackageManager::Winget)],
            Os::MacOs => &[("brew", PackageManager::Brew)],
            Os::Linux => &[
                ("apt-get", PackageManager::Apt),
                ("dnf", PackageManager::Dnf),
                ("pacman", PackageManager::Pacman),
                ("brew", PackageManager::Brew),
            ],
        };

        candidates
            .iter()
            .find(|(tool, _)| lookup.tool(tool).is_some())
            .map(|&(_, manager)| manager)
    }

    /// The command that installs `packages` (already spelled for this manager).
    fn install(self, packages: &str) -> String {
        match self {
            PackageManager::Brew => format!("brew install {packages}"),
            PackageManager::Apt => format!("apt install {packages}"),
            PackageManager::Dnf => format!("dnf install {packages}"),
            PackageManager::Pacman => format!("pacman -S {packages}"),
            PackageManager::Winget => format!("winget install {packages}"),
        }
    }
}

/// How a piece is installed with each manager, and where to get it with none.
struct Row {
    label: &'static str,
    brew: Option<&'static str>,
    apt: Option<&'static str>,
    dnf: Option<&'static str>,
    pacman: Option<&'static str>,
    winget: Option<&'static str>,
    /// Said when the manager has no package for it, or none was detected.
    elsewhere: &'static str,
}

const NONE: Row = Row {
    label: "",
    brew: None,
    apt: None,
    dnf: None,
    pacman: None,
    winget: None,
    elsewhere: "",
};

/// A piece the machine can be missing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Need {
    /// The C driver (`cc`) that links host programs.
    CDriver,
    /// `ld.lld`, `wasm-ld`, `lld-link`.
    Lld,
    /// `clang`.
    Clang,
    /// The archiver (`ar`).
    Archiver,
    /// A GNU cross driver and C library for a Linux target, or its sysroot from the cache.
    CrossLinux {
        /// The Linux target built for.
        target: Target,
    },
    /// MinGW-w64, for a Windows target built on another system.
    Mingw {
        /// Whether the Windows target is Arm64 rather than x64.
        aarch64: bool,
    },
    /// The MSVC linker (Visual Studio build tools) or an LLVM that can stand in for it.
    Msvc,
    /// Apple's SDK, which cannot be redistributed: only a Mac has it.
    AppleSdk,
    /// compiler-rt builtins (or `libgcc.a`) for a freestanding target.
    Builtins {
        /// The freestanding target.
        target: Target,
    },
    /// The GNU Arm embedded toolchain.
    ArmEmbedded,
    /// The AVR GNU toolchain.
    Avr,
    /// A QEMU program (`qemu-system-arm`, `qemu-arm`).
    Qemu {
        program: String,
    },
    /// Node.js, a runner program.
    Node,
    /// `wasmtime`, a runner program.
    Wasmtime,
    /// `git`, which `jacket migrate` and `jacket dep` ask about the work tree.
    Git,
    /// `curl`, which `jacket upgrade` and `jacket toolchain add` download with.
    Curl,
    /// `tar`, which unpacks what they download.
    Tar,
    /// Python 3, which builds the native libraries a test needs.
    Python,
    /// `dsymutil`, which collects debug information on macOS.
    Dsymutil,
    /// A runner jacket has no package for.
    Other {
        program: String,
    },
}

impl Need {
    /// The piece a runner program belongs to.
    pub fn for_program(program: &str) -> Need {
        match program {
            "node" => Need::Node,
            "wasmtime" => Need::Wasmtime,
            "avrdude" | "avr-gcc" => Need::Avr,
            "arm-none-eabi-gcc" => Need::ArmEmbedded,
            _ if program.starts_with("qemu-") => Need::Qemu {
                program: program.to_string(),
            },
            _ => Need::Other {
                program: program.to_string(),
            },
        }
    }

    fn row(&self) -> Row {
        match self {
            Need::CDriver => Row {
                label: "a C compiler",
                apt: Some("build-essential"),
                dnf: Some("gcc"),
                pacman: Some("gcc"),
                winget: Some("LLVM.LLVM"),
                elsewhere: "install clang or gcc",
                ..NONE
            },
            Need::Lld => Row {
                label: "LLD",
                brew: Some("lld"),
                apt: Some("lld"),
                dnf: Some("lld"),
                pacman: Some("lld"),
                winget: Some("LLVM.LLVM"),
                elsewhere: "download LLVM from https://github.com/llvm/llvm-project/releases",
            },
            Need::Clang => Row {
                label: "clang",
                brew: Some("llvm"),
                apt: Some("clang"),
                dnf: Some("clang"),
                pacman: Some("clang"),
                winget: Some("LLVM.LLVM"),
                elsewhere: "download LLVM from https://github.com/llvm/llvm-project/releases",
            },
            Need::Archiver => Row {
                label: "an archiver",
                apt: Some("binutils"),
                dnf: Some("binutils"),
                pacman: Some("binutils"),
                winget: Some("LLVM.LLVM"),
                elsewhere: "install binutils or LLVM",
                ..NONE
            },
            Need::CrossLinux {
                target,
            } => {
                let gnu = crate::pins::gnu_cross_package(*target);

                Row {
                    label: "a Linux linker and C library",
                    apt: gnu.apt,
                    dnf: gnu.dnf,
                    elsewhere: "name a sysroot with `-sysroot`",
                    ..NONE
                }
            }
            Need::Mingw {
                aarch64: false,
            } => Row {
                label: "MinGW-w64",
                brew: Some("mingw-w64"),
                apt: Some("gcc-mingw-w64"),
                dnf: Some("mingw64-gcc"),
                pacman: Some("mingw-w64-gcc"),
                winget: Some("MartinStorsjo.LLVM-MinGW.UCRT"),
                elsewhere: "download llvm-mingw from https://github.com/mstorsjo/llvm-mingw/releases",
            },
            Need::Mingw {
                aarch64: true,
            } => Row {
                label: "llvm-mingw",
                winget: Some("MartinStorsjo.LLVM-MinGW.UCRT"),
                elsewhere: "download llvm-mingw from https://github.com/mstorsjo/llvm-mingw/releases",
                ..NONE
            },
            Need::Msvc => Row {
                label: "LLVM (clang) or the Visual Studio build tools",
                winget: Some("LLVM.LLVM"),
                elsewhere: "install the \"Desktop development with C++\" workload of the Visual Studio build tools, or LLVM",
                ..NONE
            },
            Need::AppleSdk => Row {
                label: "Apple's SDK",
                elsewhere: "it cannot be redistributed, so build for macOS on a Mac",
                ..NONE
            },
            Need::Builtins {
                ..
            } => NONE,
            Need::ArmEmbedded => Row {
                label: "the GNU Arm embedded toolchain",
                brew: Some("--cask gcc-arm-embedded"),
                apt: Some("gcc-arm-none-eabi"),
                dnf: Some("arm-none-eabi-gcc-cs"),
                pacman: Some("arm-none-eabi-gcc"),
                winget: Some("Arm.GnuArmEmbeddedToolchain"),
                elsewhere: "download it from https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads",
            },
            Need::Avr => Row {
                label: "the AVR GNU toolchain",
                brew: Some("osx-cross/avr/avr-gcc"),
                apt: Some("gcc-avr avr-libc"),
                dnf: Some("avr-gcc"),
                pacman: Some("avr-gcc"),
                winget: None,
                elsewhere: "download it from https://www.microchip.com/en-us/tools-resources/develop/microchip-studio/gcc-compilers",
            },
            Need::Qemu {
                program,
            } => {
                let debian = if program.starts_with("qemu-system-avr") {
                    "qemu-system-misc"
                } else if program.starts_with("qemu-system-") {
                    "qemu-system-arm"
                } else {
                    "qemu-user"
                };

                Row {
                    label: "QEMU",
                    brew: Some("qemu"),
                    apt: Some(debian),
                    dnf: Some("qemu"),
                    pacman: Some("qemu-full"),
                    winget: Some("SoftwareFreedomConservancy.QEMU"),
                    elsewhere: "download it from https://www.qemu.org/download/",
                }
            }
            Need::Node => Row {
                label: "Node.js",
                brew: Some("node"),
                apt: Some("nodejs"),
                dnf: Some("nodejs"),
                pacman: Some("nodejs"),
                winget: Some("OpenJS.NodeJS"),
                elsewhere: "download it from https://nodejs.org",
            },
            Need::Wasmtime => Row {
                label: "wasmtime",
                brew: Some("wasmtime"),
                pacman: Some("wasmtime"),
                winget: Some("BytecodeAlliance.Wasmtime"),
                elsewhere: "download it from https://wasmtime.dev",
                ..NONE
            },
            Need::Git => Row {
                label: "git",
                brew: Some("git"),
                apt: Some("git"),
                dnf: Some("git"),
                pacman: Some("git"),
                winget: Some("Git.Git"),
                elsewhere: "download it from https://git-scm.com/downloads",
            },
            Need::Curl => Row {
                label: "curl",
                brew: Some("curl"),
                apt: Some("curl"),
                dnf: Some("curl"),
                pacman: Some("curl"),
                elsewhere: "download it from https://curl.se/download.html (Windows 10 and later ship `curl.exe` in System32: put that folder on PATH)",
                ..NONE
            },
            Need::Tar => Row {
                label: "tar",
                apt: Some("tar"),
                dnf: Some("tar"),
                pacman: Some("tar"),
                elsewhere: "install GNU tar or bsdtar (macOS and Windows 10 and later ship one: check it is on PATH)",
                ..NONE
            },
            Need::Python => Row {
                label: "Python 3",
                brew: Some("python"),
                apt: Some("python3"),
                dnf: Some("python3"),
                pacman: Some("python"),
                winget: Some("Python.Python.3.13"),
                elsewhere: "download it from https://www.python.org/downloads/",
            },
            Need::Dsymutil => Row {
                label: "dsymutil",
                brew: Some("llvm"),
                apt: Some("llvm"),
                dnf: Some("llvm"),
                pacman: Some("llvm"),
                winget: Some("LLVM.LLVM"),
                elsewhere: "download LLVM from https://github.com/llvm/llvm-project/releases",
            },
            Need::Other {
                ..
            } => NONE,
        }
    }

    /// The note for a missing piece: one command for `manager`, or where to download it.
    pub fn note(&self, os: Os, manager: Option<PackageManager>) -> String {
        match self {
            Need::Builtins {
                target,
            } => return builtins_note(*target, os, manager),
            Need::Other {
                program,
            } => return format!("install `{program}`"),
            Need::CDriver if os == Os::MacOs => {
                return "install the Xcode command line tools with `xcode-select --install`".into();
            }
            // Homebrew's `llvm` is keg-only, so installing it would not put `dsymutil` on PATH;
            // the Xcode tools do.
            Need::Dsymutil if os == Os::MacOs => {
                return "install the Xcode command line tools with `xcode-select --install`".into();
            }
            Need::Git if os == Os::MacOs && manager.is_none() => {
                return "install the Xcode command line tools with `xcode-select --install`".into();
            }
            Need::Archiver if os == Os::MacOs => {
                return "install the Xcode command line tools with `xcode-select --install`".into();
            }
            Need::CrossLinux {
                target,
            } => return cross_linux_note(*target, manager, self.row()),
            _ => {}
        }

        let row = self.row();
        let packages = manager.and_then(|manager| match manager {
            PackageManager::Brew => row.brew,
            PackageManager::Apt => row.apt,
            PackageManager::Dnf => row.dnf,
            PackageManager::Pacman => row.pacman,
            PackageManager::Winget => row.winget,
        });

        match (manager, packages) {
            (Some(manager), Some(packages)) => {
                format!("install {} with `{}`", row.label, manager.install(packages))
            }
            _ => row.elsewhere.to_string(),
        }
    }
}

fn cross_linux_note(target: Target, manager: Option<PackageManager>, row: Row) -> String {
    let package = manager.and_then(|manager| match manager {
        PackageManager::Apt => row.apt,
        PackageManager::Dnf => row.dnf,
        _ => None,
    });

    if let (Some(manager), Some(package)) = (manager, package) {
        return format!(
            "install a Linux linker and C library with `{}`",
            manager.install(package)
        );
    }

    if crate::pins::sysroot(target).is_some() {
        return format!("run `jacket toolchain add {target}` to fetch its C library");
    }

    format!("{} for `{target}`", row.elsewhere)
}

fn builtins_note(target: Target, os: Os, manager: Option<PackageManager>) -> String {
    let system = match target.cpu {
        jaic::target::TargetCpu::Avr => Some(Need::Avr),
        jaic::target::TargetCpu::Arm => Some(Need::ArmEmbedded),
        _ => None,
    };
    let add = format!("run `jacket toolchain add {target}` to build its builtins");

    match system {
        Some(system) => format!("{add}, or {}", system.note(os, manager)),
        None => add,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeHost;

    fn note(need: &Need, os: Os, manager: Option<PackageManager>) -> String {
        need.note(os, manager)
    }

    #[test]
    fn the_manager_is_detected_from_the_tools_on_the_machine() {
        let detect = |host: &FakeHost| PackageManager::detect(&Lookup::new(host));

        assert_eq!(detect(&FakeHost::debian()), Some(PackageManager::Apt));
        let mut fedora = FakeHost::new(Os::Linux, "x86_64");

        fedora.path = vec!["/usr/bin".into()];
        fedora.file("/usr/bin/dnf");
        assert_eq!(detect(&fedora), Some(PackageManager::Dnf));
        let mut arch = FakeHost::new(Os::Linux, "x86_64");

        arch.path = vec!["/usr/bin".into()];
        arch.file("/usr/bin/pacman");
        assert_eq!(detect(&arch), Some(PackageManager::Pacman));
        let mut mac = FakeHost::mac();

        assert_eq!(detect(&mac), None);
        mac.file("/opt/homebrew/bin/brew");
        assert_eq!(detect(&mac), Some(PackageManager::Brew));
        let mut windows = FakeHost::new(Os::Windows, "x86_64");

        windows.path = vec![r"C:\WindowsApps".into()];
        windows.file(r"C:\WindowsApps/winget.exe");
        assert_eq!(detect(&windows), Some(PackageManager::Winget));
        assert_eq!(detect(&FakeHost::new(Os::Linux, "x86_64")), None);
    }

    #[test]
    fn lld_has_a_command_for_every_manager() {
        for (manager, command) in [
            (PackageManager::Brew, "brew install lld"),
            (PackageManager::Apt, "apt install lld"),
            (PackageManager::Dnf, "dnf install lld"),
            (PackageManager::Pacman, "pacman -S lld"),
            (PackageManager::Winget, "winget install LLVM.LLVM"),
        ] {
            assert_eq!(
                note(&Need::Lld, Os::Linux, Some(manager)),
                format!("install LLD with `{command}`")
            );
        }
    }

    #[test]
    fn embedded_toolchains_use_the_right_package_names() {
        let arm = |manager| note(&Need::ArmEmbedded, Os::Linux, Some(manager));

        assert_eq!(
            arm(PackageManager::Brew),
            "install the GNU Arm embedded toolchain with `brew install --cask gcc-arm-embedded`"
        );
        assert!(arm(PackageManager::Apt).ends_with("`apt install gcc-arm-none-eabi`"));
        assert!(arm(PackageManager::Dnf).ends_with("`dnf install arm-none-eabi-gcc-cs`"));
        assert!(
            note(&Need::Avr, Os::Linux, Some(PackageManager::Apt))
                .ends_with("`apt install gcc-avr avr-libc`")
        );
        // Windows has no AVR package: the note says where to download it.
        assert!(
            note(&Need::Avr, Os::Windows, Some(PackageManager::Winget)).contains("microchip.com")
        );
    }

    #[test]
    fn qemu_packages_differ_by_program_on_debian() {
        let qemu = |program: &str| {
            note(
                &Need::Qemu {
                    program: program.into(),
                },
                Os::Linux,
                Some(PackageManager::Apt),
            )
        };

        assert!(qemu("qemu-system-arm").ends_with("`apt install qemu-system-arm`"));
        assert!(qemu("qemu-system-avr").ends_with("`apt install qemu-system-misc`"));
        assert!(qemu("qemu-arm").ends_with("`apt install qemu-user`"));
    }

    #[test]
    fn without_a_manager_a_note_says_where_to_download() {
        assert!(note(&Need::Lld, Os::Linux, None).contains("github.com/llvm"));
        assert!(note(&Need::Node, Os::Linux, None).contains("nodejs.org"));
        assert_eq!(
            note(&Need::for_program("my-flasher"), Os::Linux, None),
            "install `my-flasher`"
        );
    }

    #[test]
    fn builtins_name_the_toolchain_add_command_and_the_system_package() {
        let target = Target::parse("thumbv7em-none-eabihf").unwrap();

        assert_eq!(
            note(
                &Need::Builtins {
                    target
                },
                Os::MacOs,
                Some(PackageManager::Brew)
            ),
            "run `jacket toolchain add thumbv7em-none-eabihf` to build its builtins, or install the GNU Arm embedded toolchain with `brew install --cask gcc-arm-embedded`"
        );
        let target = Target::parse("aarch64-none-elf").unwrap();

        assert_eq!(
            note(
                &Need::Builtins {
                    target
                },
                Os::Linux,
                None
            ),
            "run `jacket toolchain add aarch64-none-elf` to build its builtins"
        );
    }

    /// One of every `Need`. The match has no wildcard, so a new variant stops this from compiling
    /// until it is listed here and therefore covered by the table test below.
    fn every_need() -> Vec<Need> {
        let target = Target::parse("aarch64-unknown-linux-gnu").unwrap();
        let all = [
            Need::CDriver,
            Need::Lld,
            Need::Clang,
            Need::Archiver,
            Need::CrossLinux {
                target,
            },
            Need::Mingw {
                aarch64: false,
            },
            Need::Mingw {
                aarch64: true,
            },
            Need::Msvc,
            Need::AppleSdk,
            Need::Builtins {
                target,
            },
            Need::ArmEmbedded,
            Need::Avr,
            Need::Qemu {
                program: "qemu-arm".into(),
            },
            Need::Node,
            Need::Wasmtime,
            Need::Git,
            Need::Curl,
            Need::Tar,
            Need::Python,
            Need::Dsymutil,
            Need::Other {
                program: "x".into(),
            },
        ];

        for need in &all {
            match need {
                Need::CDriver
                | Need::Lld
                | Need::Clang
                | Need::Archiver
                | Need::CrossLinux {
                    ..
                }
                | Need::Mingw {
                    ..
                }
                | Need::Msvc
                | Need::AppleSdk
                | Need::Builtins {
                    ..
                }
                | Need::ArmEmbedded
                | Need::Avr
                | Need::Qemu {
                    ..
                }
                | Need::Node
                | Need::Wasmtime
                | Need::Git
                | Need::Curl
                | Need::Tar
                | Need::Python
                | Need::Dsymutil
                | Need::Other {
                    ..
                } => {}
            }
        }

        all.to_vec()
    }

    #[test]
    fn every_need_says_what_to_do_on_every_system() {
        let managers = [
            None,
            Some(PackageManager::Brew),
            Some(PackageManager::Apt),
            Some(PackageManager::Dnf),
            Some(PackageManager::Pacman),
            Some(PackageManager::Winget),
        ];

        for need in every_need() {
            for os in [Os::MacOs, Os::Linux, Os::Windows] {
                for manager in managers {
                    let text = note(&need, os, manager);

                    assert!(
                        !text.trim().is_empty(),
                        "{need:?} on {os:?} with {manager:?} has no note"
                    );
                }
            }

            // A manager with no package for it falls back to where to get the piece: the row's
            // own text, which is what says why there is no command.
            if !matches!(
                need,
                Need::Builtins { .. } | Need::Other { .. } | Need::CrossLinux { .. }
            ) {
                assert!(
                    !need.row().elsewhere.is_empty(),
                    "{need:?} has no text for a manager without its package"
                );
            }
        }
    }

    #[test]
    fn the_programs_jacket_starts_have_a_command_per_manager() {
        let apt = Some(PackageManager::Apt);

        assert_eq!(
            note(&Need::Git, Os::Linux, apt),
            "install git with `apt install git`"
        );
        assert_eq!(
            note(&Need::Curl, Os::Linux, Some(PackageManager::Pacman)),
            "install curl with `pacman -S curl`"
        );
        assert_eq!(
            note(&Need::Tar, Os::Linux, Some(PackageManager::Dnf)),
            "install tar with `dnf install tar`"
        );
        assert_eq!(
            note(&Need::Python, Os::MacOs, Some(PackageManager::Brew)),
            "install Python 3 with `brew install python`"
        );
        assert_eq!(
            note(&Need::Git, Os::Windows, Some(PackageManager::Winget)),
            "install git with `winget install Git.Git`"
        );
        // Without a manager the note says where to get it; a Mac has these in the Xcode tools.
        assert!(note(&Need::Git, Os::Linux, None).contains("git-scm.com"));
        assert!(note(&Need::Git, Os::MacOs, None).contains("xcode-select --install"));

        // Homebrew's llvm is keg-only: the Xcode tools are the way to get `dsymutil` on a Mac.
        for manager in [None, Some(PackageManager::Brew)] {
            assert!(note(&Need::Dsymutil, Os::MacOs, manager).contains("xcode-select --install"));
        }
    }

    #[test]
    fn a_runner_program_maps_to_its_package() {
        assert_eq!(Need::for_program("node"), Need::Node);
        assert_eq!(Need::for_program("wasmtime"), Need::Wasmtime);
        assert!(matches!(
            Need::for_program("qemu-system-aarch64"),
            Need::Qemu { .. }
        ));
    }
}
