//! Where the LLVM install lives, for the tools that run LLVM's programs (`clang`, `wasm-ld`,
//! `llvm-dwarfdump`). See `docs/tools/llvm-setup.md`.
//!
//! The install is looked for in this order: the `llvm-sys` variable ([`SYS_PREFIX_VAR`]),
//! `LLVM_PREFIX` (which CI sets too), the prefix baked in when jaic was built, `llvm-config
//! --prefix`, then the platform's usual places (Homebrew's `llvm@<major>`, Debian's
//! `/usr/lib/llvm-<major>`).
use std::path::PathBuf;
use std::process::Command;

mod version;

pub use version::{MAJOR, SYS_PREFIX_VAR};

/// The install named by the environment: the `llvm-sys` variable, else `LLVM_PREFIX`.
pub fn configured_prefix() -> Option<PathBuf> {
    [&crate::env::LLVM_SYS_PREFIX, &crate::env::LLVM_PREFIX]
        .into_iter()
        .find_map(crate::env::Var::non_empty)
        .map(PathBuf::from)
}

/// What `llvm-config <flag>` prints (a path), trying the versioned program first.
fn llvm_config_query(flag: &str) -> Option<PathBuf> {
    [format!("llvm-config-{MAJOR}"), "llvm-config".to_string()]
        .iter()
        .find_map(|program| {
            let out = Command::new(program).arg(flag).output().ok()?;
            let text = String::from_utf8_lossy(&out.stdout);
            (out.status.success() && !text.trim().is_empty()).then(|| PathBuf::from(text.trim()))
        })
}

/// Every LLVM install to look in, best first. `baked` is the prefix recorded at build time.
pub fn prefixes(baked: Option<&str>) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    found.extend(configured_prefix());
    found.extend(baked.filter(|p| !p.is_empty()).map(PathBuf::from));
    found.extend(llvm_config_query("--prefix"));
    for root in ["/opt/homebrew/opt", "/usr/local/opt"] {
        found.push(PathBuf::from(root).join(format!("llvm@{MAJOR}")));
    }
    found.push(PathBuf::from(format!("/usr/lib/llvm-{MAJOR}")));
    found
}

/// The `bin` directory of each of [`prefixes`], plus the one `llvm-config --bindir` names.
pub fn bin_dirs(baked: Option<&str>) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = prefixes(baked).iter().map(|p| p.join("bin")).collect();
    dirs.extend(llvm_config_query("--bindir"));
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `.github/pins.json` (CI and the Python tools) and this crate must name the same release.
    #[test]
    fn matches_the_pinned_version() {
        let pins = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/pins.json");
        let Ok(text) = std::fs::read_to_string(pins) else {
            return;
        };
        let line = text
            .lines()
            .find(|l| l.trim_start().starts_with("\"llvm\":"))
            .expect("pins.json has an llvm pin");
        let version = line.split('"').nth(3).expect("llvm pin value");
        let mut parts = version.split('.');
        let (major, minor) = (parts.next().unwrap(), parts.next().unwrap());
        assert_eq!(SYS_PREFIX_VAR, format!("LLVM_SYS_{major}{minor}_PREFIX"));
        assert_eq!(MAJOR, major);
    }

    #[test]
    fn prefixes_end_with_platform_defaults() {
        let all = prefixes(Some("/baked"));
        assert!(all.contains(&PathBuf::from("/baked")));
        assert_eq!(
            all.last().unwrap(),
            &PathBuf::from(format!("/usr/lib/llvm-{MAJOR}"))
        );
    }
}
