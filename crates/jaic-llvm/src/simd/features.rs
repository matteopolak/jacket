//! Target features: which vector instruction sets the machine being compiled for has.
//!
//! LLVM's C API cannot say which features a named CPU implies (`-cpu skylake`): it only
//! reports the feature string it was given, and `LLVMGetHostCPUFeatures` for the build machine.
//! So the set is resolved here in three steps:
//!
//! 1. the CPU name gives a starting set (`cpu_features`, the one table in this file, keyed by
//!    the LLVM CPU names and listing only the features this module names);
//! 2. each `+x` of the feature string adds `x` and everything `x` implies (`avx2` brings
//!    `avx`, `sse4.2`, ... as LLVM does);
//! 3. each `-x` removes `x` and everything that implies `x`.
//!
//! `"native"` needs no table: its feature string is the host's complete list, applied on top.

use jaic::abi::Arch;

/// One vector instruction set extension, named as LLVM names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Feature {
    Sse2,
    Ssse3,
    Sse41,
    Sse42,
    Avx,
    Avx2,
    Avx512f,
    Avx512bw,
    Avx512vbmi,
    Neon,
    Sve,
    Sve2,
    Simd128,
    RelaxedSimd,
}

/// The architecture families with their own vector features.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    X86,
    Aarch64,
    Wasm,
}

impl Family {
    pub fn of(arch: Arch) -> Family {
        match arch {
            Arch::X86_64 | Arch::Win64 => Family::X86,
            Arch::Aarch64 | Arch::Win64Arm => Family::Aarch64,
            Arch::Wasm64 => Family::Wasm,
        }
    }
}

impl Feature {
    const ALL: [Feature; 14] = [
        Feature::Sse2,
        Feature::Ssse3,
        Feature::Sse41,
        Feature::Sse42,
        Feature::Avx,
        Feature::Avx2,
        Feature::Avx512f,
        Feature::Avx512bw,
        Feature::Avx512vbmi,
        Feature::Neon,
        Feature::Sve,
        Feature::Sve2,
        Feature::Simd128,
        Feature::RelaxedSimd,
    ];

    /// The name in an LLVM feature string (`+avx2`).
    pub fn name(self) -> &'static str {
        match self {
            Feature::Sse2 => "sse2",
            Feature::Ssse3 => "ssse3",
            Feature::Sse41 => "sse4.1",
            Feature::Sse42 => "sse4.2",
            Feature::Avx => "avx",
            Feature::Avx2 => "avx2",
            Feature::Avx512f => "avx512f",
            Feature::Avx512bw => "avx512bw",
            Feature::Avx512vbmi => "avx512vbmi",
            Feature::Neon => "neon",
            Feature::Sve => "sve",
            Feature::Sve2 => "sve2",
            Feature::Simd128 => "simd128",
            Feature::RelaxedSimd => "relaxed-simd",
        }
    }

    pub fn family(self) -> Family {
        match self {
            Feature::Sse2
            | Feature::Ssse3
            | Feature::Sse41
            | Feature::Sse42
            | Feature::Avx
            | Feature::Avx2
            | Feature::Avx512f
            | Feature::Avx512bw
            | Feature::Avx512vbmi => Family::X86,
            Feature::Neon | Feature::Sve | Feature::Sve2 => Family::Aarch64,
            Feature::Simd128 | Feature::RelaxedSimd => Family::Wasm,
        }
    }

    /// The feature this one requires (LLVM turns it on with this one).
    fn implies(self) -> Option<Feature> {
        match self {
            Feature::Ssse3 => Some(Feature::Sse2),
            Feature::Sse41 => Some(Feature::Ssse3),
            Feature::Sse42 => Some(Feature::Sse41),
            Feature::Avx => Some(Feature::Sse42),
            Feature::Avx2 => Some(Feature::Avx),
            Feature::Avx512f => Some(Feature::Avx2),
            Feature::Avx512bw => Some(Feature::Avx512f),
            Feature::Avx512vbmi => Some(Feature::Avx512bw),
            Feature::Sve2 => Some(Feature::Sve),
            Feature::RelaxedSimd => Some(Feature::Simd128),
            _ => None,
        }
    }

    fn from_name(name: &str) -> Option<Feature> {
        Feature::ALL.into_iter().find(|f| f.name() == name)
    }
}

/// A set of features of one family.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct FeatureSet(u32);

impl FeatureSet {
    pub fn has(self, f: Feature) -> bool {
        self.0 & (1 << f as u8) != 0
    }

    /// `f` and everything it implies.
    fn enable(&mut self, f: Feature) {
        let mut f = Some(f);
        while let Some(x) = f {
            self.0 |= 1 << x as u8;
            f = x.implies();
        }
    }

    /// `f` and everything that implies it.
    fn disable(&mut self, f: Feature) {
        self.0 &= !(1 << f as u8);
        for g in Feature::ALL {
            if self.has(g) && g.implies() == Some(f) {
                self.disable(g);
            }
        }
    }
}

/// What the target machine has: its architecture family and vector features.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub family: Family,
    pub features: FeatureSet,
}

impl Target {
    /// The features of `arch` with `cpu` (LLVM's `-mcpu` name, empty for the default) and the
    /// LLVM feature string `features` (`+avx2,-sse4.1`).
    pub fn resolve(arch: Arch, cpu: &str, features: &str) -> Target {
        let family = Family::of(arch);
        let mut set = FeatureSet::default();
        for &f in cpu_features(family, cpu) {
            set.enable(f);
        }
        for item in features.split(',') {
            let item = item.trim();
            let (on, name) = match (item.strip_prefix('+'), item.strip_prefix('-')) {
                (Some(n), _) => (true, n),
                (_, Some(n)) => (false, n),
                _ => continue,
            };
            match Feature::from_name(name) {
                Some(f) if f.family() == family && on => set.enable(f),
                Some(f) if f.family() == family => set.disable(f),
                _ => {}
            }
        }
        Target {
            family,
            features: set,
        }
    }

    pub fn has(&self, f: Feature) -> bool {
        self.features.has(f)
    }
}

/// The vector features a named CPU has, beyond the architecture's baseline (SSE2 on x86-64,
/// NEON on aarch64, nothing on wasm). Only the features this module names are listed. An
/// unknown name gets the baseline: code for it stays correct, just not the fastest.
fn cpu_features(family: Family, cpu: &str) -> &'static [Feature] {
    use Feature::*;
    match family {
        Family::X86 => {
            let cpu = cpu.to_ascii_lowercase();
            let is = |names: &[&str]| names.contains(&cpu.as_str());
            if is(&[
                "icelake-client",
                "icelake-server",
                "tigerlake",
                "rocketlake",
                "sapphirerapids",
                "emeraldrapids",
                "graniterapids",
                "graniterapids-d",
                "cannonlake",
                "znver4",
                "znver5",
                "diamondrapids",
            ]) {
                &[Avx512vbmi]
            } else if is(&[
                "skylake-avx512",
                "skx",
                "cascadelake",
                "cooperlake",
                "x86-64-v4",
            ]) {
                &[Avx512bw]
            } else if is(&["knl", "knm"]) {
                &[Avx512f]
            } else if is(&[
                "haswell",
                "core-avx2",
                "broadwell",
                "skylake",
                "x86-64-v3",
                "bdver4",
                "alderlake",
                "raptorlake",
                "meteorlake",
                "arrowlake",
                "arrowlake-s",
                "lunarlake",
                "pantherlake",
                "gracemont",
                "sierraforest",
                "grandridge",
                "clearwaterforest",
            ]) || cpu.starts_with("znver")
            {
                &[Avx2]
            } else if is(&[
                "sandybridge",
                "corei7-avx",
                "ivybridge",
                "core-avx-i",
                "bdver1",
                "bdver2",
                "bdver3",
                "btver2",
            ]) {
                &[Avx]
            } else if is(&[
                "nehalem",
                "corei7",
                "westmere",
                "silvermont",
                "slm",
                "goldmont",
                "goldmont-plus",
                "tremont",
                "x86-64-v2",
            ]) {
                &[Sse42]
            } else if is(&["penryn", "core2-45nm"]) {
                &[Sse41]
            } else if is(&["core2", "bonnell", "atom", "btver1"]) {
                // Also the macOS x86-64 baseline (`baseline_cpu`): every Intel Mac has SSSE3.
                &[Ssse3]
            } else {
                // x86-64, x86-64-v1, generic, the empty name, and the CPUs older than SSSE3
                // (pentium*, i386.., k6, k8, opteron, athlon, amdfam10, barcelona, nocona,
                // prescott, yonah), which share the baseline.
                &[Sse2]
            }
        }
        Family::Aarch64 => match cpu {
            "neoverse-n2" | "neoverse-v2" | "neoverse-v3" | "neoverse-v3ae" | "cortex-a510"
            | "cortex-a710" | "cortex-a715" | "cortex-a720" | "cortex-a520" | "cortex-x2"
            | "cortex-x3" | "cortex-x4" | "cortex-x925" => &[Neon, Sve2],
            "neoverse-v1" | "neoverse-512tvb" | "a64fx" => &[Neon, Sve],
            _ => &[Neon],
        },
        Family::Wasm => match cpu {
            "bleeding-edge" => &[Simd128],
            _ => &[],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn x86(cpu: &str, features: &str) -> Target {
        Target::resolve(Arch::X86_64, cpu, features)
    }

    #[test]
    fn x86_baselines() {
        // The Linux/Windows baseline predates SSSE3; the macOS one (core2) has it.
        assert!(!x86("x86-64", "").has(Feature::Ssse3));
        assert!(x86("x86-64", "").has(Feature::Sse2));
        assert!(!x86("", "").has(Feature::Ssse3));
        assert!(x86("core2", "").has(Feature::Ssse3));
        assert!(!x86("core2", "").has(Feature::Sse41));
        assert!(!x86("core2", "").has(Feature::Avx2));
    }

    #[test]
    fn x86_named_cpus_imply_their_features() {
        let skylake = x86("skylake", "");
        assert!(skylake.has(Feature::Avx2) && skylake.has(Feature::Ssse3));
        assert!(!skylake.has(Feature::Avx512f));
        assert!(x86("znver3", "").has(Feature::Avx2));
        assert!(x86("skylake-avx512", "").has(Feature::Avx512bw));
        assert!(!x86("skylake-avx512", "").has(Feature::Avx512vbmi));
        assert!(x86("icelake-client", "").has(Feature::Avx512vbmi));
        assert!(!x86("sandybridge", "").has(Feature::Avx2));
        assert!(x86("sandybridge", "").has(Feature::Ssse3));
    }

    #[test]
    fn feature_string_adds_with_implications() {
        let t = x86("x86-64", "+avx2");
        assert!(t.has(Feature::Avx2) && t.has(Feature::Ssse3) && t.has(Feature::Sse42));
        assert!(!t.has(Feature::Avx512f));
        assert!(x86("x86-64", "+sse4.1").has(Feature::Ssse3));
        assert!(x86("x86-64", "+avx512vbmi").has(Feature::Avx2));
        assert!(!x86("x86-64", "+bulk-memory,+nonsense").has(Feature::Ssse3));
    }

    #[test]
    fn feature_string_removes_dependents() {
        let t = x86("skylake", "-avx2");
        assert!(!t.has(Feature::Avx2) && t.has(Feature::Avx));
        // Removing a base feature removes everything built on it.
        let t = x86("icelake-client", "-ssse3");
        assert!(!t.has(Feature::Ssse3) && !t.has(Feature::Avx2) && !t.has(Feature::Avx512vbmi));
        assert!(t.has(Feature::Sse2));
        // Later items win, as in LLVM.
        assert!(!x86("x86-64", "+avx2,-avx2").has(Feature::Avx2));
        assert!(x86("x86-64", "-avx2,+avx2").has(Feature::Avx2));
    }

    #[test]
    fn host_style_feature_list_overrides_the_name() {
        // `native`: the host's CPU name plus its explicit list, which may be narrower.
        let t = x86("skylake", "+sse2,+ssse3,+sse4.1,+sse4.2,+avx,-avx2");
        assert!(t.has(Feature::Avx) && !t.has(Feature::Avx2));
    }

    #[test]
    fn features_of_other_families_are_ignored() {
        assert!(!x86("x86-64", "+neon,+simd128,+sve").has(Feature::Neon));
        let wasm = Target::resolve(Arch::Wasm64, "generic", "+avx2");
        assert!(!wasm.has(Feature::Avx2));
    }

    #[test]
    fn aarch64() {
        let a = |cpu, f| Target::resolve(Arch::Aarch64, cpu, f);
        assert!(a("generic", "").has(Feature::Neon));
        assert!(a("apple-m1", "").has(Feature::Neon) && !a("apple-m1", "").has(Feature::Sve));
        assert!(a("generic", "+sve2").has(Feature::Sve));
        assert!(a("neoverse-v1", "").has(Feature::Sve) && !a("neoverse-v1", "").has(Feature::Sve2));
        assert!(a("neoverse-n2", "").has(Feature::Sve2));
        assert!(!a("neoverse-n2", "-sve").has(Feature::Sve2));
        assert!(!a("generic", "-neon").has(Feature::Neon));
        assert_eq!(
            Target::resolve(Arch::Win64Arm, "", "").family,
            Family::Aarch64
        );
    }

    #[test]
    fn wasm() {
        let w = |cpu, f| Target::resolve(Arch::Wasm64, cpu, f);
        assert!(!w("generic", "+bulk-memory").has(Feature::Simd128));
        assert!(w("generic", "+simd128").has(Feature::Simd128));
        assert!(w("generic", "+relaxed-simd").has(Feature::Simd128));
        assert!(w("bleeding-edge", "").has(Feature::Simd128));
        assert!(!w("bleeding-edge", "-simd128").has(Feature::Simd128));
    }
}
