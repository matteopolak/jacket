//! Replacing an install folder without ever leaving it half-written.
//!
//! The new tree is fully unpacked and checked beside the old one first. Swapping is then a few
//! renames, each undone if a later one fails, so the old install is either all there or all
//! replaced.
//!
//! Unix renames the folder itself: the running `jacket` keeps its open file, so nothing in the
//! way. Windows cannot rename a folder that holds a running executable, but it can rename the
//! executable (and every other file) inside it, so there each top-level entry is moved aside
//! into `.jacket-old` and the new entries are moved in. Whatever could not be deleted at once
//! (the old `jacket.exe` is still running) is removed by the next `upgrade` ([`clean_stale`]).
use std::path::{Path, PathBuf};

/// Suffix of the sibling folder a new tree is unpacked into.
pub const NEW_SUFFIX: &str = ".jacket-new";

/// Suffix of the sibling folder the old tree is moved to; also the name, inside the install, of
/// the folder the old entries go to on Windows.
pub const OLD_SUFFIX: &str = ".jacket-old";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Rename the whole folder.
    Folder,
    /// Move the entries one by one, for Windows.
    Entries,
}

impl Mode {
    pub const fn native() -> Mode {
        if cfg!(windows) {
            Mode::Entries
        } else {
            Mode::Folder
        }
    }
}

/// `<dir><suffix>` next to `dir`.
pub fn sibling(dir: &Path, suffix: &str) -> PathBuf {
    let mut name = dir.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    dir.with_file_name(name)
}

fn aside_dir(install: &Path) -> PathBuf {
    install.join(OLD_SUFFIX)
}

/// Removes what an earlier upgrade left behind. Best effort: a file that is still in use stays
/// until the next run.
pub fn clean_stale(install: &Path) {
    for path in [
        sibling(install, NEW_SUFFIX),
        sibling(install, OLD_SUFFIX),
        aside_dir(install),
    ] {
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}

fn rename(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::rename(from, to)
        .map_err(|e| format!("could not move {} to {}: {e}", from.display(), to.display()))
}

/// Undoes `done` (renames, oldest first), newest first; errors here are ignored, since there is
/// nothing better left to do.
fn roll_back(done: &[(PathBuf, PathBuf)]) {
    for (from, to) in done.iter().rev() {
        let _ = std::fs::rename(to, from);
    }
}

/// Makes `fresh` the contents of `install`. `fresh` is consumed (moved into place); on error the
/// old install is as it was.
pub fn swap(install: &Path, fresh: &Path, mode: Mode) -> Result<(), String> {
    match mode {
        Mode::Folder => swap_folder(install, fresh),
        Mode::Entries => swap_entries(install, fresh),
    }
}

fn swap_folder(install: &Path, fresh: &Path) -> Result<(), String> {
    let old = sibling(install, OLD_SUFFIX);
    if old.exists() {
        std::fs::remove_dir_all(&old)
            .map_err(|e| format!("could not remove the leftover {}: {e}", old.display()))?;
    }
    rename(install, &old)?;
    if let Err(error) = rename(fresh, install) {
        let _ = std::fs::rename(&old, install);
        return Err(error);
    }
    // Deleting the old tree is cleanup, not part of the swap: the new one is already in place.
    let _ = std::fs::remove_dir_all(&old);
    Ok(())
}

fn swap_entries(install: &Path, fresh: &Path) -> Result<(), String> {
    let aside = aside_dir(install);
    let old_entries = names(install)?
        .into_iter()
        .filter(|name| name != OLD_SUFFIX)
        .collect::<Vec<_>>();
    let new_entries = names(fresh)?;
    if aside.exists() {
        std::fs::remove_dir_all(&aside)
            .map_err(|e| format!("could not remove the leftover {}: {e}", aside.display()))?;
    }
    std::fs::create_dir_all(&aside)
        .map_err(|e| format!("could not create {}: {e}", aside.display()))?;
    let mut done: Vec<(PathBuf, PathBuf)> = Vec::new();
    let moves = old_entries
        .iter()
        .map(|n| (install.join(n), aside.join(n)))
        .chain(new_entries.iter().map(|n| (fresh.join(n), install.join(n))));
    for (from, to) in moves {
        if let Err(error) = rename(&from, &to) {
            roll_back(&done);
            let _ = std::fs::remove_dir_all(&aside);
            return Err(error);
        }
        done.push((from, to));
    }
    let _ = std::fs::remove_dir_all(&aside);
    Ok(())
}

fn names(dir: &Path) -> Result<Vec<std::ffi::OsString>, String> {
    std::fs::read_dir(dir)
        .and_then(|entries| entries.map(|e| e.map(|e| e.file_name())).collect())
        .map_err(|e| format!("could not read {}: {e}", dir.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Scratch {
            let dir =
                std::env::temp_dir().join(format!("jacket-swap-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }

        fn tree(&self, name: &str, files: &[(&str, &str)]) -> PathBuf {
            let root = self.0.join(name);
            for (path, text) in files {
                let file = root.join(path);
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                std::fs::write(file, text).unwrap();
            }
            root
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn read(dir: &Path, file: &str) -> Option<String> {
        std::fs::read_to_string(dir.join(file)).ok()
    }

    const BOTH: [Mode; 2] = [Mode::Folder, Mode::Entries];

    #[test]
    fn the_new_tree_replaces_the_old_one_completely() {
        for mode in BOTH {
            let s = Scratch::new(&format!("replace-{mode:?}"));
            let install = s.tree(
                "0.21.0",
                &[("jacket", "old"), ("stdlib/a.jai", "old"), ("gone", "x")],
            );
            let fresh = s.tree(
                "fresh",
                &[
                    ("jacket", "new"),
                    ("stdlib/a.jai", "new"),
                    ("native-libs/l", "new"),
                ],
            );
            swap(&install, &fresh, mode).unwrap();
            assert_eq!(read(&install, "jacket").as_deref(), Some("new"), "{mode:?}");
            assert_eq!(read(&install, "stdlib/a.jai").as_deref(), Some("new"));
            assert_eq!(read(&install, "native-libs/l").as_deref(), Some("new"));
            assert_eq!(
                read(&install, "gone"),
                None,
                "old files do not survive ({mode:?})"
            );
            assert!(!install.join(OLD_SUFFIX).exists());
            assert!(!sibling(&install, OLD_SUFFIX).exists());
        }
    }

    #[test]
    fn a_failed_swap_leaves_the_old_install_untouched() {
        for mode in BOTH {
            let s = Scratch::new(&format!("fail-{mode:?}"));
            let install = s.tree("0.21.0", &[("jacket", "old"), ("stdlib/a.jai", "old")]);
            let missing = s.0.join("no-such-fresh-tree");
            assert!(swap(&install, &missing, mode).is_err(), "{mode:?}");
            assert_eq!(read(&install, "jacket").as_deref(), Some("old"), "{mode:?}");
            assert_eq!(read(&install, "stdlib/a.jai").as_deref(), Some("old"));
            assert!(!install.join(OLD_SUFFIX).exists());
            assert!(!sibling(&install, OLD_SUFFIX).exists());
        }
    }

    #[test]
    fn rolling_back_undoes_moves_newest_first() {
        let s = Scratch::new("rollback");
        let install = s.tree("0.21.0", &[("a", "old-a"), ("b", "old-b")]);
        let aside = s.0.join("aside");
        std::fs::create_dir(&aside).unwrap();
        let mut done = Vec::new();
        for name in ["a", "b"] {
            let (from, to) = (install.join(name), aside.join(name));
            std::fs::rename(&from, &to).unwrap();
            done.push((from, to));
        }
        roll_back(&done);
        assert_eq!(read(&install, "a").as_deref(), Some("old-a"));
        assert_eq!(read(&install, "b").as_deref(), Some("old-b"));
    }

    #[test]
    fn leftovers_are_cleaned_and_replaced_by_the_next_swap() {
        for mode in BOTH {
            let s = Scratch::new(&format!("stale-{mode:?}"));
            let install = s.tree("0.21.0", &[("jacket", "old")]);
            s.tree(&format!("0.21.0{NEW_SUFFIX}"), &[("x", "stale")]);
            s.tree(&format!("0.21.0{OLD_SUFFIX}"), &[("x", "stale")]);
            s.tree(&format!("0.21.0/{OLD_SUFFIX}"), &[("x", "stale")]);
            clean_stale(&install);
            assert!(!sibling(&install, NEW_SUFFIX).exists());
            assert!(!sibling(&install, OLD_SUFFIX).exists());
            assert!(!install.join(OLD_SUFFIX).exists());
            assert_eq!(read(&install, "jacket").as_deref(), Some("old"));

            // A leftover that cleaning could not remove does not block the swap either.
            s.tree(&format!("0.21.0{OLD_SUFFIX}"), &[("x", "stale")]);
            s.tree(&format!("0.21.0/{OLD_SUFFIX}"), &[("x", "stale")]);
            let fresh = s.tree("fresh", &[("jacket", "new")]);
            swap(&install, &fresh, mode).unwrap();
            assert_eq!(read(&install, "jacket").as_deref(), Some("new"), "{mode:?}");
        }
    }

    #[test]
    fn siblings_keep_the_folder_name() {
        assert_eq!(
            sibling(Path::new("/a/b/0.21.0"), NEW_SUFFIX),
            Path::new("/a/b/0.21.0.jacket-new")
        );
    }
}
