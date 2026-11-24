//! Finding programs: `PATH` first, then the places tools are installed without being put on it.
use crate::host::{Host, Os};
use std::path::PathBuf;

/// The one place programs are searched for.
pub struct Lookup<'a> {
    pub host: &'a dyn Host,
    /// The `bin` folders of the LLVM installs jaic knows (`jacket_config::llvm::bin_dirs`), best
    /// first. Asked for only when an LLVM program is looked up: finding them starts `llvm-config`.
    llvm_bins: Box<dyn Fn() -> Vec<PathBuf> + 'a>,
    /// The LLVM installs themselves (`jacket_config::llvm::prefixes`), best first.
    llvm_prefixes: Box<dyn Fn() -> Vec<PathBuf> + 'a>,
}

impl<'a> Lookup<'a> {
    pub fn new(host: &'a dyn Host) -> Lookup<'a> {
        Lookup {
            host,
            llvm_bins: Box::new(Vec::new),
            llvm_prefixes: Box::new(Vec::new),
        }
    }

    #[must_use]
    pub fn with_llvm_bins(mut self, bins: impl Fn() -> Vec<PathBuf> + 'a) -> Lookup<'a> {
        self.llvm_bins = Box::new(bins);
        self
    }

    #[must_use]
    pub fn with_llvm_prefixes(mut self, prefixes: impl Fn() -> Vec<PathBuf> + 'a) -> Lookup<'a> {
        self.llvm_prefixes = Box::new(prefixes);
        self
    }

    /// The LLVM installs jaic knows, best first.
    pub fn llvm_prefixes(&self) -> Vec<PathBuf> {
        (self.llvm_prefixes)()
    }

    /// The file names `name` goes by in a folder.
    fn file_names(&self, name: &str) -> Vec<String> {
        if self.host.os() == Os::Windows && !name.to_ascii_lowercase().ends_with(".exe") {
            vec![name.to_string(), format!("{name}.exe")]
        } else {
            vec![name.to_string()]
        }
    }

    /// `name` in the first of `dirs` that has it.
    pub fn in_dirs(&self, dirs: &[PathBuf], name: &str) -> Option<PathBuf> {
        let names = self.file_names(name);
        dirs.iter().find_map(|dir| {
            names
                .iter()
                .map(|file| dir.join(file))
                .find(|candidate| self.host.is_file(candidate))
        })
    }

    /// `name` in a folder of `PATH`.
    pub fn which(&self, name: &str) -> Option<PathBuf> {
        self.in_dirs(&self.host.path_dirs(), name)
    }

    /// Where tools are installed without being on `PATH`: Homebrew's prefixes, and QEMU's and
    /// LLVM's folders under `Program Files`.
    fn installed_dirs(&self) -> Vec<PathBuf> {
        match self.host.os() {
            Os::MacOs => vec!["/opt/homebrew/bin".into(), "/usr/local/bin".into()],
            Os::Linux => vec!["/home/linuxbrew/.linuxbrew/bin".into()],
            Os::Windows => self
                .host
                .program_files()
                .into_iter()
                .flat_map(|base| [base.join("qemu"), base.join("LLVM").join("bin")])
                .collect(),
        }
    }

    /// `name` on `PATH`, else in an installed-but-not-on-`PATH` place.
    pub fn tool(&self, name: &str) -> Option<PathBuf> {
        self.which(name)
            .or_else(|| self.in_dirs(&self.installed_dirs(), name))
    }

    /// An LLVM program (`ld.lld`, `wasm-ld`, `clang`): the LLVM installs jaic knows, Homebrew's
    /// `lld` kegs (a separate package from `llvm`), `Program Files\LLVM`, then `PATH` with the
    /// versioned name (`ld.lld-23`) before the plain one.
    pub fn llvm_tool(&self, name: &str) -> Option<PathBuf> {
        let major = jacket_config::llvm::MAJOR;
        let mut dirs = (self.llvm_bins)();
        for keg in ["lld".to_string(), format!("lld@{major}")] {
            for root in ["/opt/homebrew/opt", "/usr/local/opt"] {
                dirs.push(PathBuf::from(root).join(&keg).join("bin"));
            }
        }
        for base in self.host.program_files() {
            dirs.push(base.join("LLVM").join("bin"));
        }
        self.in_dirs(&dirs, name)
            .or_else(|| self.which(&format!("{name}-{major}")))
            .or_else(|| self.which(name))
    }
}

/// The version a tool prints for `--version`: the first word that is digits and dots with at
/// least one dot, after any `v` and punctuation (`Homebrew LLD 21.1.8 (compatible with GNU
/// linkers)` is `21.1.8`; `(Build arm-14.52)` is not a version).
pub fn version_of(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        line.split_whitespace()
            .map(|word| {
                let word = word.trim_matches(|c: char| "(),;:".contains(c));
                word.strip_prefix('v').unwrap_or(word)
            })
            .find(|word| {
                word.starts_with(|c: char| c.is_ascii_digit())
                    && word.contains('.')
                    && word.chars().all(|c| c.is_ascii_digit() || c == '.')
            })
            .map(str::to_string)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeHost;

    #[test]
    fn path_comes_before_the_places_tools_hide_in() {
        let mut host = FakeHost::mac();
        host.file("/opt/homebrew/bin/qemu-system-arm");
        host.path = vec!["/usr/bin".into()];
        let lookup = Lookup::new(&host);
        assert_eq!(lookup.which("qemu-system-arm"), None);
        assert_eq!(
            lookup.tool("qemu-system-arm"),
            Some("/opt/homebrew/bin/qemu-system-arm".into())
        );
        drop(lookup);
        host.file("/usr/bin/qemu-system-arm");
        assert_eq!(
            Lookup::new(&host).tool("qemu-system-arm"),
            Some("/usr/bin/qemu-system-arm".into())
        );
    }

    #[test]
    fn windows_tries_the_exe_name() {
        let mut host = FakeHost::new(Os::Windows, "x86_64");
        host.path = vec![r"C:\bin".into()];
        host.file(r"C:\bin/node.exe");
        host.program_files = vec![r"C:\Program Files".into()];
        host.file(r"C:\Program Files/qemu/qemu-system-arm.exe");
        let lookup = Lookup::new(&host);
        assert_eq!(lookup.which("node"), Some(r"C:\bin/node.exe".into()));
        assert_eq!(
            lookup.tool("qemu-system-arm"),
            Some(r"C:\Program Files/qemu/qemu-system-arm.exe".into())
        );
    }

    #[test]
    fn an_llvm_tool_is_found_in_the_installs_then_the_kegs_then_on_path() {
        let major = jacket_config::llvm::MAJOR;
        let mut host = FakeHost::mac();
        host.file("/opt/homebrew/bin/ld.lld");
        host.path.push("/opt/homebrew/bin".into());
        host.file("/opt/homebrew/opt/lld/bin/ld.lld");
        let lookup = Lookup::new(&host).with_llvm_bins(|| vec!["/llvm/bin".into()]);
        assert_eq!(
            lookup.llvm_tool("ld.lld"),
            Some("/opt/homebrew/opt/lld/bin/ld.lld".into())
        );
        drop(lookup);
        host.file("/llvm/bin/ld.lld");
        let lookup = Lookup::new(&host).with_llvm_bins(|| vec!["/llvm/bin".into()]);
        assert_eq!(lookup.llvm_tool("ld.lld"), Some("/llvm/bin/ld.lld".into()));
        // On a Debian machine the versioned name beats the plain one.
        let mut host = FakeHost::debian();
        host.file("/usr/bin/wasm-ld");
        host.file(&format!("/usr/bin/wasm-ld-{major}"));
        assert_eq!(
            Lookup::new(&host).llvm_tool("wasm-ld"),
            Some(format!("/usr/bin/wasm-ld-{major}").into())
        );
        assert_eq!(Lookup::new(&host).llvm_tool("ld.lld"), None);
    }

    #[test]
    fn versions_are_read_from_the_first_line() {
        assert_eq!(
            version_of("Homebrew LLD 21.1.8 (compatible with GNU linkers)\n").as_deref(),
            Some("21.1.8")
        );
        assert_eq!(
            version_of(
                "Apple clang version 17.0.0 (clang-1700.0.13.3)\nTarget: arm64-apple-darwin25.0.0\n"
            )
            .as_deref(),
            Some("17.0.0")
        );
        assert_eq!(
            version_of("arm-none-eabi-gcc (Arm GNU Toolchain 14.2.Rel1) 14.2.1 20241119")
                .as_deref(),
            Some("14.2.1")
        );
        assert_eq!(
            version_of(
                "arm-none-eabi-gcc (Arm GNU Toolchain 14.2.Rel1 (Build arm-14.52)) 14.2.1 20241119"
            )
            .as_deref(),
            Some("14.2.1")
        );
        assert_eq!(version_of("v22.23.1").as_deref(), Some("22.23.1"));
        assert_eq!(version_of("no numbers here"), None);
    }
}
