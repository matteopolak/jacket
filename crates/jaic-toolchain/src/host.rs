//! The machine the lookups read, behind a trait so tests describe one instead of having it.
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

    /// The CPU as Rust names it (`aarch64`, `x86_64`).
    fn arch(&self) -> &str;

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

    /// What `program args` prints (stdout, then stderr) when it starts and succeeds.
    fn run(&self, program: &Path, args: &[&str]) -> Option<String>;
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

    fn arch(&self) -> &str {
        std::env::consts::ARCH
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

    fn run(&self, program: &Path, args: &[&str]) -> Option<String> {
        let out = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .output()
            .ok()?;
        out.status.success().then(|| {
            format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        })
    }
}
