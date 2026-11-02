use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::TempDir;

fn target_dir() -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_jacket"))
        .parent()
        .expect("jacket lives in a directory")
        .to_path_buf()
}

pub fn stdlib() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
}

/// A scratch directory of its own, so tests do not share files.
pub fn scratch(name: &str) -> TempDir {
    let dir = TempDir::new(&format!("jacket-test-{name}"));
    std::fs::create_dir_all(&dir).expect("create the scratch directory");
    dir
}

/// `jacket` and `jaic` copied into a fresh directory, as an unpacked archive has them. `None`
/// when `jaic` has not been built (`cargo test -p jacket` alone does not build it).
pub fn archive(root: &Path) -> Option<PathBuf> {
    let suffix = std::env::consts::EXE_SUFFIX;
    let jaic = target_dir().join(format!("jaic{suffix}"));
    if !jaic.is_file() {
        eprintln!("skipped: build `jaic` first (`cargo build -p jaic-cli`)");
        return None;
    }
    let bin = root.join("archive");
    std::fs::create_dir_all(&bin).expect("create the archive directory");
    std::fs::copy(&jaic, bin.join(format!("jaic{suffix}"))).expect("copy jaic");
    let jacket = bin.join(format!("jacket{suffix}"));
    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).expect("copy jacket");
    Some(jacket)
}

pub fn jacket(exe: &Path, dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(exe)
        .args(args)
        .current_dir(dir)
        .env("JAIC_STDLIB", stdlib())
        .output()
        .expect("start jacket")
}
