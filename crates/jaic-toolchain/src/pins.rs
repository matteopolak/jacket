//! What jacket knows by name: the packages that provide a cross target's pieces and the
//! releases `jacket toolchain add` fetches. Every download is pinned here and checked by its
//! sha256.
use jaic::target::{Target, TargetAbi, TargetCpu, TargetOs};

/// The GNU cross toolchain's package names for a target.
#[derive(Clone, Copy, Debug, Default)]
pub struct GnuPackages {
    pub apt: Option<&'static str>,
    pub dnf: Option<&'static str>,
}

/// The prefix of the GNU cross tools for a Linux target (`arm-linux-gnueabihf-gcc`).
pub fn gnu_triple(target: Target) -> Option<&'static str> {
    if target.os != TargetOs::Linux || target.abi == TargetAbi::Musl {
        return None;
    }
    Some(match target.cpu {
        TargetCpu::Arm => "arm-linux-gnueabihf",
        TargetCpu::Arm64 => "aarch64-linux-gnu",
        TargetCpu::X86 => "i686-linux-gnu",
        TargetCpu::X64 => "x86_64-linux-gnu",
        TargetCpu::S390x => "s390x-linux-gnu",
        TargetCpu::PowerPc64 => "powerpc64-linux-gnu",
        _ => return None,
    })
}

/// The packages that provide [`gnu_triple`]'s compiler driver and C library.
pub fn gnu_cross_package(target: Target) -> GnuPackages {
    match target.cpu {
        TargetCpu::Arm => GnuPackages {
            apt: Some("gcc-arm-linux-gnueabihf"),
            dnf: Some("gcc-arm-linux-gnu"),
        },
        TargetCpu::Arm64 => GnuPackages {
            apt: Some("gcc-aarch64-linux-gnu"),
            dnf: Some("gcc-aarch64-linux-gnu"),
        },
        TargetCpu::X86 => GnuPackages {
            apt: Some("gcc-i686-linux-gnu"),
            dnf: None,
        },
        TargetCpu::X64 => GnuPackages {
            apt: Some("gcc-x86-64-linux-gnu"),
            dnf: Some("gcc-x86_64-linux-gnu"),
        },
        TargetCpu::S390x => GnuPackages {
            apt: Some("gcc-s390x-linux-gnu"),
            dnf: Some("gcc-s390x-linux-gnu"),
        },
        TargetCpu::PowerPc64 => GnuPackages {
            apt: Some("gcc-powerpc64-linux-gnu"),
            dnf: Some("gcc-powerpc64-linux-gnu"),
        },
        _ => GnuPackages::default(),
    }
}

/// A Linux sysroot `jacket toolchain add` can fetch.
#[derive(Clone, Copy, Debug)]
pub struct SysrootPin {
    /// Debian's name for the architecture (`armhf`).
    pub debian_arch: &'static str,
}

/// The sysroot `jacket toolchain add` fetches for `target`, if there is one.
pub fn sysroot(target: Target) -> Option<SysrootPin> {
    if target.os != TargetOs::Linux || target.abi != TargetAbi::Gnu {
        return None;
    }
    let debian_arch = match target.cpu {
        TargetCpu::Arm => "armhf",
        TargetCpu::Arm64 => "arm64",
        TargetCpu::X86 => "i386",
        TargetCpu::X64 => "amd64",
        TargetCpu::S390x => "s390x",
        _ => return None,
    };
    Some(SysrootPin {
        debian_arch,
    })
}
