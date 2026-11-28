//! The machine the lookups read, behind a trait so tests describe one instead of having it.
use jacket_config::env::Var;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The operating systems jacket runs on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Linux,
    Windows,
}

/// What a lookup asks of the machine.
pub trait Host {
    fn os(&self) -> Os;

    /// The CPU the machine's programs run on.
    fn cpu(&self) -> jaic::target::TargetCpu;

    /// The folders of `PATH`, in order.
    fn path_dirs(&self) -> Vec<PathBuf>;

    /// The `Program Files` folders (Windows only; empty elsewhere).
    fn program_files(&self) -> Vec<PathBuf>;

    /// The user cache folder jacket keeps `toolchains/` in.
    fn cache_root(&self) -> Option<PathBuf>;

    fn is_file(&self, path: &Path) -> bool;

    fn is_dir(&self, path: &Path) -> bool;

    /// The entries of a folder, sorted; empty when it cannot be read.
    fn read_dir(&self, path: &Path) -> Vec<PathBuf>;

    /// The text of a file, when it can be read as UTF-8.
    fn read_to_string(&self, path: &Path) -> Option<String>;

    /// Where the symbolic link `path` points.
    fn read_link(&self, path: &Path) -> Option<PathBuf>;

    /// The value of an environment variable, when set.
    fn var(&self, var: &Var) -> Option<OsString>;

    /// What `program args` prints (stdout, then stderr) when it starts and succeeds.
    fn run(&self, program: &Path, args: &[&str]) -> Option<String> {
        let (succeeded, text) = self.output(program, args)?;
        succeeded.then_some(text)
    }

    /// Whether `program args` succeeded and what it printed (stdout, then stderr), when it
    /// starts.
    fn output(&self, program: &Path, args: &[&str]) -> Option<(bool, String)>;

    /// The absolute form of `path` with links resolved, when it exists.
    fn canonicalize(&self, path: &Path) -> Option<PathBuf>;

    /// Microsoft's `tool` (`clang.exe`) from a Visual Studio installation, for the CPU folder
    /// `arch` (`x64`, `arm64`). Always `None` off Windows.
    fn visual_studio_tool(&self, tool: &str, arch: &str) -> Option<PathBuf>;
}

/// The machine this process runs on.
pub struct System;

impl Host for System {
    fn os(&self) -> Os {
        if cfg!(windows) {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOs
        } else {
            Os::Linux
        }
    }

    fn cpu(&self) -> jaic::target::TargetCpu {
        jaic::target::Target::host().cpu
    }

    fn path_dirs(&self) -> Vec<PathBuf> {
        jacket_config::env::PATH
            .os()
            .map(|path| std::env::split_paths(&path).collect())
            .unwrap_or_default()
    }

    fn program_files(&self) -> Vec<PathBuf> {
        if !cfg!(windows) {
            return Vec::new();
        }
        let mut dirs: Vec<PathBuf> = Vec::new();
        for (var, default) in [
            (jacket_config::env::PROGRAM_FILES, r"C:\Program Files"),
            (
                jacket_config::env::PROGRAM_FILES_X86,
                r"C:\Program Files (x86)",
            ),
        ] {
            let dir = var
                .non_empty()
                .map_or_else(|| PathBuf::from(default), PathBuf::from);
            if !dirs.contains(&dir) {
                dirs.push(dir);
            }
        }
        dirs
    }

    fn cache_root(&self) -> Option<PathBuf> {
        jacket_config::user_cache_dir().ok()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn read_dir(&self, path: &Path) -> Vec<PathBuf> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        entries
    }

    fn read_to_string(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn read_link(&self, path: &Path) -> Option<PathBuf> {
        std::fs::read_link(path).ok()
    }

    fn var(&self, var: &Var) -> Option<OsString> {
        var.os()
    }

    fn output(&self, program: &Path, args: &[&str]) -> Option<(bool, String)> {
        let out = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .output()
            .ok()?;
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        Some((out.status.success(), text))
    }

    fn canonicalize(&self, path: &Path) -> Option<PathBuf> {
        jaic::canonicalize(path).ok()
    }

    fn visual_studio_tool(&self, tool: &str, arch: &str) -> Option<PathBuf> {
        jaic::toolchain::discover::find_tool(tool, arch).map(|found| found.program)
    }
}
