//! Native library discovery for the compiler binary.
use super::crash_runtime;
use jaic::target::Target;
use std::path::{Path, PathBuf};

/// Where `tools/native/build_native_libs.py` (or a release package) puts third-party libraries for this
/// host, or the `JAIC_NATIVE_LIBS` path list.
pub(super) fn native_lib_dirs(stdlib: &Path) -> Vec<PathBuf> {
    // The crash runtime is built into jaic itself (docs/native/crash-runtime-build.md). It comes
    // first so an older copy left in a configured folder never shadows the one this stdlib needs;
    // the folder holds only those runtimes, so no other library is affected.
    let mut dirs: Vec<PathBuf> = crash_runtime::dir().into_iter().collect();
    dirs.extend(configured_native_lib_dirs(stdlib));
    dirs.extend(apple_toolchain_lib_dir());
    dirs
}

/// The Xcode (or Command Line Tools) toolchain's library folder on a Mac, when it holds
/// `libclang.dylib`: the `Bindings_Generator` parses headers with it, and the release archive does
/// not carry a second copy (the `Bindings_Generator` section of docs/tools/releases.md). Only a folder with
/// that library is searched, so the toolchain's other libraries are never picked up by name.
fn apple_toolchain_lib_dir() -> Option<PathBuf> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let selected = jacket_config::env::DEVELOPER_DIR.os().map(PathBuf::from);
    let candidates = selected
        .iter()
        .flat_map(|dir| {
            [
                dir.join("Toolchains/XcodeDefault.xctoolchain/usr/lib"),
                dir.join("usr/lib"),
            ]
        })
        .chain([
            PathBuf::from("/Library/Developer/CommandLineTools/usr/lib"),
            PathBuf::from(
                "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib",
            ),
        ]);
    candidates
        .into_iter()
        .find(|dir| dir.join("libclang.dylib").is_file())
}

fn configured_native_lib_dirs(stdlib: &Path) -> Vec<PathBuf> {
    if let Some(dirs) = jacket_config::env::NATIVE_LIBS.os() {
        return std::env::split_paths(&dirs).collect();
    }
    let host = Target::host();
    if host.is_wasm() {
        return Vec::new();
    }
    let platform = format!(
        "{}-{}",
        jaic::target::os_name(host.os),
        jaic::target::cpu_name(host.cpu)
    );
    // `JACKET_OUTPUT_DIR` (tools/repo_output.py) relocates the build output directory; a
    // release package carries `native-libs/` beside `stdlib/`, a checkout builds them under
    // `target/`.
    let mut candidates = Vec::new();
    if let Some(output) = jacket_config::env::OUTPUT_DIR.os() {
        // A relative one is below the checkout, as the repository's tools read it.
        candidates.push(
            stdlib
                .join("..")
                .join(output)
                .join("native-libs")
                .join(&platform),
        );
    }
    candidates.push(stdlib.join("../native-libs").join(&platform));
    candidates.push(stdlib.join("../target/native-libs").join(&platform));
    // A linked git worktree shares the main checkout's builds, where `build_native_libs.py` puts
    // them (`tools/repo_output.py`).
    candidates.extend(
        main_checkout(&stdlib.join(".."))
            .map(|main| main.join("target/native-libs").join(&platform)),
    );
    candidates
        .iter()
        .find_map(|dir| jaic::canonicalize(dir).ok())
        .map(|d| vec![d])
        .unwrap_or_default()
}

/// The main checkout of the repository whose linked worktree is `root`: `.git` there is a file
/// naming the worktree's git folder, whose `commondir` leads to the main checkout's `.git`.
fn main_checkout(root: &Path) -> Option<PathBuf> {
    let link = std::fs::read_to_string(root.join(".git")).ok()?;
    let git_dir = PathBuf::from(link.trim().strip_prefix("gitdir:")?.trim());
    let git_dir = if git_dir.is_absolute() {
        git_dir
    } else {
        root.join(git_dir)
    };
    let common = std::fs::read_to_string(git_dir.join("commondir")).ok()?;
    let common = jaic::canonicalize(git_dir.join(common.trim())).ok()?;
    common.parent().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_linked_worktree_finds_the_main_checkout() {
        let base = std::env::temp_dir().join(format!("jaic-worktree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let git = base.join("main/.git");
        let admin = git.join("worktrees/w");
        std::fs::create_dir_all(&admin).unwrap();
        std::fs::create_dir_all(base.join("main/.claude/worktrees/w")).unwrap();
        std::fs::write(admin.join("commondir"), "../..\n").unwrap();
        let worktree = base.join("main/.claude/worktrees/w");
        std::fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", admin.display()),
        )
        .unwrap();
        let main = main_checkout(&worktree).unwrap();
        assert_eq!(main, jaic::canonicalize(base.join("main")).unwrap());
        // The main checkout's `.git` is a folder, and a plain folder has none.
        assert_eq!(main_checkout(&base.join("main")), None);
        assert_eq!(main_checkout(&base), None);
        let _ = std::fs::remove_dir_all(&base);
    }
}
