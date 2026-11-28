//! Where `tools/native/build_native_libs.py` (or a release package) put the third-party libraries
//! a program links and the interpreter opens: `native-libs/<os>-<cpu>` beside the stdlib, under
//! the checkout's `target/`, or the main checkout's when this is a linked git worktree. See
//! `docs/tools/native-libs.md`.
use std::path::{Path, PathBuf};

use jaic::target::Target;

use crate::host::Host;

/// The folder name `tools/native/build_native_libs.py` gives the libraries built for `target`:
/// `<os>-<cpu>`, with `-mingw` for a Windows target linked by MinGW.
fn platform_name(target: Target) -> String {
    let mingw = if target.os == jaic::target::TargetOs::Windows
        && target.abi == jaic::target::TargetAbi::Gnu
    {
        "-mingw"
    } else {
        ""
    };

    format!(
        "{}-{}{mingw}",
        jaic::target::os_name(target.os),
        jaic::target::cpu_name(target.cpu)
    )
}

/// Where the libraries built for a cross `target` are: the target's own folder beside the host's.
/// Empty when none was built, so a cross link finds none of the host's archives.
pub fn cross_lib_dirs(host: &dyn Host, stdlib: &Path, target: Target) -> Vec<PathBuf> {
    if target.is_wasm() || target.is_freestanding() {
        return Vec::new();
    }

    built_lib_dirs(host, stdlib, &platform_name(target))
}

/// The library folders for the target this `jaic` was built for: the `JAIC_NATIVE_LIBS` path
/// list when set (an error when one of its folders is not there), else the first built folder
/// that exists.
pub fn host_lib_dirs(host: &dyn Host, stdlib: &Path) -> Result<Vec<PathBuf>, String> {
    if let Some(list) = host
        .var(&jacket_config::env::NATIVE_LIBS)
        .filter(|v| !v.is_empty())
    {
        let dirs: Vec<PathBuf> = std::env::split_paths(&list).collect();

        if let Some(gone) = dirs.iter().find(|dir| !host.is_dir(dir)) {
            return Err(format!(
                "JAIC_NATIVE_LIBS names {}, which is not a folder\n\
                 note: build the libraries with `tools/native/build_native_libs.py`, or correct \
                 or unset JAIC_NATIVE_LIBS",
                gone.display()
            ));
        }

        return Ok(dirs);
    }

    let target = Target::host();

    if target.is_wasm() {
        return Ok(Vec::new());
    }

    Ok(built_lib_dirs(host, stdlib, &platform_name(target)))
}

/// The first of the places a build puts `platform`'s libraries that exists.
fn built_lib_dirs(host: &dyn Host, stdlib: &Path, platform: &str) -> Vec<PathBuf> {
    // `JACKET_OUTPUT_DIR` (tools/repo_output.py) relocates the build output directory; a
    // release package carries `native-libs/` beside `stdlib/`, a checkout builds them under
    // `target/`.
    let mut candidates = Vec::new();

    if let Some(output) = host.var(&jacket_config::env::OUTPUT_DIR) {
        // A relative one is below the checkout, as the repository's tools read it.
        candidates.push(stdlib.join("..").join(output).join("native-libs"));
    }

    candidates.push(stdlib.join("../native-libs"));
    candidates.push(stdlib.join("../target/native-libs"));
    // A linked git worktree shares the main checkout's builds, where `build_native_libs.py` puts
    // them (`tools/repo_output.py`).
    candidates.extend(
        host.canonicalize(&stdlib.join(".."))
            .and_then(|root| main_checkout(host, &root))
            .map(|main| main.join("target/native-libs")),
    );
    candidates
        .iter()
        .find_map(|dir| host.canonicalize(&dir.join(platform)))
        .into_iter()
        .collect()
}

/// The main checkout of the repository whose linked worktree is `root`: `.git` there is a file
/// naming the worktree's git folder, whose `commondir` leads to the main checkout's `.git`.
fn main_checkout(host: &dyn Host, root: &Path) -> Option<PathBuf> {
    let link = host.read_to_string(&root.join(".git"))?;
    let git_dir = PathBuf::from(link.trim().strip_prefix("gitdir:")?.trim());
    let git_dir = if git_dir.is_absolute() {
        git_dir
    } else {
        root.join(git_dir)
    };
    let common = host.read_to_string(&git_dir.join("commondir"))?;
    let common = host.canonicalize(&git_dir.join(common.trim()))?;

    common.parent().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use jacket_config::env;

    use super::*;
    use crate::host::Os;
    use crate::testing::FakeHost;

    const STDLIB: &str = "/jacket/stdlib";

    fn host_platform() -> String {
        platform_name(Target::host())
    }

    #[test]
    fn a_linked_worktree_finds_the_main_checkout() {
        let mut host = FakeHost::mac();

        host.dir("/main/.git/worktrees/w");
        host.text("/main/.git/worktrees/w/commondir", "../..\n");
        host.text(
            "/main/.claude/worktrees/w/.git",
            "gitdir: /main/.git/worktrees/w\n",
        );
        let worktree = Path::new("/main/.claude/worktrees/w");

        assert_eq!(main_checkout(&host, worktree), Some("/main".into()));
        // The main checkout's `.git` is a folder, and a plain folder has none.
        assert_eq!(main_checkout(&host, Path::new("/main")), None);
        assert_eq!(main_checkout(&host, Path::new("/")), None);
    }

    #[test]
    fn the_configured_list_wins_over_every_built_folder() {
        let mut host = FakeHost::mac();

        host.dir(&format!("/jacket/native-libs/{}", host_platform()));
        let built = host_lib_dirs(&host, Path::new(STDLIB)).unwrap();

        assert_eq!(
            built,
            [PathBuf::from(format!(
                "/jacket/native-libs/{}",
                host_platform()
            ))]
        );
        host.dir("/a").dir("/b");
        let list = std::env::join_paths(["/a", "/b"]).unwrap();

        host.set(&env::NATIVE_LIBS, list.to_str().unwrap());
        assert_eq!(
            host_lib_dirs(&host, Path::new(STDLIB)).unwrap(),
            [PathBuf::from("/a"), PathBuf::from("/b")]
        );
    }

    #[test]
    fn a_configured_folder_that_is_not_there_is_an_error() {
        let mut host = FakeHost::mac();

        host.dir("/a");
        host.dir(&format!("/jacket/native-libs/{}", host_platform()));
        let list = std::env::join_paths(["/a", "/gone"]).unwrap();

        host.set(&env::NATIVE_LIBS, list.to_str().unwrap());
        let error = host_lib_dirs(&host, Path::new(STDLIB)).unwrap_err();

        assert!(
            error.starts_with("JAIC_NATIVE_LIBS names /gone, which is not a folder"),
            "{error}"
        );
        assert!(error.contains("\nnote: "), "{error}");
    }

    #[test]
    fn built_folders_are_tried_output_dir_first_then_the_package_then_target() {
        let platform = host_platform();
        let mut host = FakeHost::mac();
        let worktree_stdlib = "/main/.claude/worktrees/w/stdlib";

        host.text(
            "/main/.claude/worktrees/w/.git",
            "gitdir: /main/.git/worktrees/w\n",
        );
        host.text("/main/.git/worktrees/w/commondir", "../..\n");
        let found = |host: &FakeHost| host_lib_dirs(host, Path::new(worktree_stdlib)).unwrap();

        host.dir(&format!("/main/target/native-libs/{platform}"));
        assert_eq!(
            found(&host),
            [PathBuf::from(format!(
                "/main/target/native-libs/{platform}"
            ))]
        );
        host.dir(&format!(
            "/main/.claude/worktrees/w/target/native-libs/{platform}"
        ));
        assert_eq!(
            found(&host),
            [PathBuf::from(format!(
                "/main/.claude/worktrees/w/target/native-libs/{platform}"
            ))]
        );
        host.dir(&format!("/main/.claude/worktrees/w/native-libs/{platform}"));
        assert_eq!(
            found(&host),
            [PathBuf::from(format!(
                "/main/.claude/worktrees/w/native-libs/{platform}"
            ))]
        );
        host.set(&env::OUTPUT_DIR, "out");
        host.dir(&format!(
            "/main/.claude/worktrees/w/out/native-libs/{platform}"
        ));
        assert_eq!(
            found(&host),
            [PathBuf::from(format!(
                "/main/.claude/worktrees/w/out/native-libs/{platform}"
            ))]
        );
        host.set(&env::OUTPUT_DIR, "/elsewhere");
        assert_eq!(
            found(&host),
            [PathBuf::from(format!(
                "/main/.claude/worktrees/w/native-libs/{platform}"
            ))],
            "an absolute output folder with no libraries is skipped"
        );
    }

    #[test]
    fn a_cross_target_has_its_own_folder_and_mingw_its_own_name() {
        let mut host = FakeHost::new(Os::Linux, "x86_64");

        host.dir("/jacket/native-libs/windows-x64-mingw");
        host.dir("/jacket/native-libs/windows-x64");
        let dirs =
            |triple: &str| cross_lib_dirs(&host, Path::new(STDLIB), Target::parse(triple).unwrap());

        assert_eq!(
            dirs("x86_64-pc-windows-gnu"),
            [PathBuf::from("/jacket/native-libs/windows-x64-mingw")]
        );
        assert_eq!(
            dirs("x86_64-pc-windows-msvc"),
            [PathBuf::from("/jacket/native-libs/windows-x64")]
        );
        assert!(dirs("aarch64-unknown-linux-gnu").is_empty(), "none built");
        assert!(dirs("wasm32-unknown-unknown").is_empty());
    }
}
