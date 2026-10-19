//! Installs another program owns. `jacket upgrade` replaces a folder made by `install.sh`,
//! `install.ps1` or an unpacked archive; a Homebrew keg, a Nix store path and the VS Code
//! extension's download are updated by their own tool, and replacing files under them would
//! fight it (the store is read-only, Homebrew would not know).
use std::path::{Component, Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Managed {
    Homebrew,
    Nix,
    VsCode,
}

impl Managed {
    /// What to run instead, as one line.
    pub fn advice(self) -> &'static str {
        match self {
            Managed::Homebrew => "this install is managed by Homebrew; run `brew upgrade jacket`",
            Managed::Nix => {
                "this install lives in the Nix store; update it with Nix, for example `nix profile upgrade jacket` (or update the flake input that provides it)"
            }
            Managed::VsCode => {
                "this toolchain was downloaded by the Jacket VS Code extension; update the extension, or run \"Jai: Download Toolchain\" from the command palette"
            }
        }
    }
}

/// Which package manager owns the install `exe` (a canonical path) belongs to, if any.
pub fn managed_by(exe: &Path) -> Option<Managed> {
    if exe.starts_with("/nix/store") {
        return Some(Managed::Nix);
    }
    let names: Vec<String> = exe
        .components()
        .filter_map(|c| match c {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    if names.iter().any(|n| n == "Cellar") {
        return Some(Managed::Homebrew);
    }
    let extension = |n: &str| {
        let n = n.to_ascii_lowercase();
        n == "matteopolak.jacket" || n.starts_with("matteopolak.jacket-")
    };
    if names.iter().any(|n| extension(n)) {
        return Some(Managed::VsCode);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(path: &str) -> Option<Managed> {
        managed_by(Path::new(path))
    }

    #[test]
    fn homebrew_kegs() {
        assert_eq!(
            kind("/opt/homebrew/Cellar/jacket/0.21.0/libexec/jacket"),
            Some(Managed::Homebrew)
        );
        assert_eq!(
            kind("/home/linuxbrew/.linuxbrew/Cellar/jacket/0.21.0/libexec/jacket"),
            Some(Managed::Homebrew)
        );
    }

    #[test]
    fn nix_store_paths() {
        assert_eq!(
            kind("/nix/store/abc123-jaic-0.21.0/libexec/jaic/jacket"),
            Some(Managed::Nix)
        );
        assert_eq!(kind("/home/me/nix/store/jacket"), None);
    }

    #[test]
    fn the_vs_code_extensions_download() {
        assert_eq!(
            kind(
                "/Users/me/Library/Application Support/Code/User/globalStorage/matteopolak.jacket/toolchain/0.21.0/jacket-macos-arm64/jacket"
            ),
            Some(Managed::VsCode)
        );
        assert_eq!(
            kind("/home/me/.vscode/extensions/matteopolak.jacket-0.5.0/bin/jacket"),
            Some(Managed::VsCode)
        );
    }

    #[test]
    fn plain_installs_are_not_managed() {
        assert_eq!(kind("/home/me/.local/share/jai/0.21.0/jacket"), None);
        assert_eq!(kind("/usr/local/bin/jacket"), None);
        assert_eq!(kind("/home/me/CellarDoor/jacket"), None);
    }

    #[test]
    fn every_kind_names_its_command() {
        assert!(Managed::Homebrew.advice().contains("brew upgrade jacket"));
        assert!(Managed::Nix.advice().contains("nix profile upgrade"));
        assert!(Managed::VsCode.advice().contains("extension"));
    }
}
