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

/// One Debian package of a sysroot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebPin {
    pub package: &'static str,
    pub version: &'static str,
    pub size: u64,
    pub sha256: &'static str,
    /// Where it is, under [`DebianPins::mirror`].
    pub path: &'static str,
}

/// The packages `jacket toolchain add` unpacks into the sysroot of one architecture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebianPins {
    /// A snapshot of Debian's archive: the same bytes for ever.
    pub mirror: &'static str,
    pub packages: Vec<DebPin>,
}

const SYSROOT_PINS: &str = include_str!("../data/sysroots.txt");

/// The pinned packages of `debian_arch`: `tools/embedded/pin_sysroots.py` writes
/// `data/sysroots.txt`, one `architecture package version size sha256 path` line each.
pub fn debian_packages(debian_arch: &str) -> DebianPins {
    pins_from(SYSROOT_PINS, debian_arch)
}

fn pins_from(text: &'static str, debian_arch: &str) -> DebianPins {
    let mut mirror = "";
    let mut packages = Vec::new();

    for line in text.lines() {
        let words: Vec<&'static str> = line.split_whitespace().collect();

        match words.as_slice() {
            [] => {}
            [first, ..] if first.starts_with('#') => {}
            ["mirror", url] => mirror = url,
            [arch, package, version, size, sha256, path] if *arch == debian_arch => {
                packages.push(DebPin {
                    package,
                    version,
                    size: size.parse().unwrap_or_default(),
                    sha256,
                    path,
                });
            }
            _ => {}
        }
    }

    DebianPins {
        mirror,
        packages,
    }
}

const COMPILER_RT_PINS: &str = include_str!("../data/compiler-rt.txt");

/// The compiler-rt release the builtins are built from, and the files each CPU's builtins are
/// compiled from. `tools/embedded/pin_compiler_rt.py` writes `data/compiler-rt.txt`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerRt {
    pub version: &'static str,
    pub sha256: &'static str,
    pub url: &'static str,
    /// Each source file under `lib/builtins` with the macros it is compiled with.
    pub sources: Vec<(&'static str, Vec<&'static str>)>,
}

impl CompilerRt {
    /// Names this release and this list of sources, so builtins built from another list are
    /// known to be out of date. FNV-1a over the text, which does not change between Rust releases.
    pub fn stamp(&self) -> String {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |text: &str| {
            for byte in text.bytes().chain([0]) {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
            }
        };

        feed(self.version);
        feed(self.sha256);

        for (source, definitions) in &self.sources {
            feed(source);

            for definition in definitions {
                feed(definition);
            }
        }

        format!("{hash:016x}")
    }
}

/// The pinned release and the sources of `family`: `aarch64`, `cortexm` or `avr`.
pub fn compiler_rt(family: &str) -> CompilerRt {
    let mut pin = CompilerRt {
        version: "",
        sha256: "",
        url: "",
        sources: Vec::new(),
    };

    for line in COMPILER_RT_PINS.lines() {
        let words: Vec<&'static str> = line.split_whitespace().collect();

        match words.as_slice() {
            [first, ..] if first.starts_with('#') => {}
            ["release", version, sha256, url] => {
                pin.version = version;
                pin.sha256 = sha256;
                pin.url = url;
            }
            [name, source, definitions @ ..] if *name == family => {
                pin.sources.push((source, definitions.to_vec()));
            }
            _ => {}
        }
    }

    pin
}

/// The family of pinned builtins sources for a freestanding `target`.
pub fn compiler_rt_family(target: Target) -> Option<&'static str> {
    match (target.cpu, target.abi) {
        (TargetCpu::Arm64, _) if target.is_freestanding() => Some("aarch64"),
        (TargetCpu::Arm, TargetAbi::CortexM(_)) => Some("cortexm"),
        (TargetCpu::Avr, _) => Some("avr"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sysroot_architecture_has_the_same_pinned_packages() {
        let arches = ["armhf", "arm64", "i386", "amd64", "s390x"];
        let first = debian_packages("armhf");

        assert!(
            first
                .mirror
                .starts_with("https://snapshot.debian.org/archive/debian/")
        );

        for arch in arches {
            let pins = debian_packages(arch);
            let names: Vec<_> = pins.packages.iter().map(|p| p.package).collect();
            let expected: Vec<_> = first.packages.iter().map(|p| p.package).collect();

            assert_eq!(names, expected, "{arch}");

            for pin in &pins.packages {
                assert_eq!(pin.sha256.len(), 64, "{arch} {}", pin.package);
                assert!(pin.path.ends_with(&format!("_{arch}.deb")), "{}", pin.path);
                assert!(pin.size > 0);
            }
        }
    }

    #[test]
    fn each_freestanding_family_has_sources_and_one_release() {
        for (triple, family) in [
            ("aarch64-none-elf", "aarch64"),
            ("thumbv7em-none-eabihf", "cortexm"),
            ("thumbv8m.main-none-eabihf", "cortexm"),
            ("avr-none-elf", "avr"),
        ] {
            let target = Target::parse(triple).unwrap();

            assert_eq!(compiler_rt_family(target), Some(family), "{triple}");
            let pin = compiler_rt(family);

            assert_eq!(pin.version, "21.1.8");
            assert_eq!(pin.sha256.len(), 64);
            assert!(pin.url.starts_with("https://"), "{}", pin.url);
            assert!(pin.sources.len() > 100, "{family}");
        }

        // Outline atomics: cas has five sizes and the other five operations four, each in five
        // memory models.
        let lse = compiler_rt("aarch64")
            .sources
            .iter()
            .filter(|(source, _)| *source == "aarch64/lse.S")
            .count();

        assert_eq!(lse, 25 + 5 * 20);
    }

    #[test]
    fn every_target_with_a_sysroot_has_pins() {
        for triple in [
            "armv7-unknown-linux-gnueabihf",
            "aarch64-unknown-linux-gnu",
            "i686-unknown-linux-gnu",
            "x86_64-unknown-linux-gnu",
            "s390x-unknown-linux-gnu",
        ] {
            let target = Target::parse(triple).unwrap();
            let pin = sysroot(target).unwrap_or_else(|| panic!("{triple}"));

            assert!(
                !debian_packages(pin.debian_arch).packages.is_empty(),
                "{triple}"
            );
        }
    }

    #[test]
    fn a_change_to_the_source_list_changes_the_stamp() {
        let pin = compiler_rt("avr");
        let mut other = pin.clone();

        other.sources.pop();
        assert_eq!(pin.stamp().len(), 16);
        assert_eq!(pin.stamp(), compiler_rt("avr").stamp());
        assert_ne!(pin.stamp(), other.stamp());
        assert_ne!(pin.stamp(), compiler_rt("cortexm").stamp());
    }
}
