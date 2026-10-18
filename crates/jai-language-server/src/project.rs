//! The project a document belongs to: its program's entry file (what `jaic` is given),
//! the files that entry `#load`s, and the folders modules are imported from. Auto-import
//! completion (`auto_import.rs`) uses it to tell a name the program already sees from one a
//! `#load` or `#import` would bring in.
//!
//! Where it comes from, first match wins:
//! 1. `jai.toml`, the nearest one at or above the document (an open one before the one on
//!    disk): `build_files` are the entry files, `import_path` adds module folders; paths are
//!    relative to its folder.
//! 2. Otherwise the workspace folder holding the document, with the entry files inferred: the
//!    first of `build.jai`, `first.jai`, `main.jai` and `src/main.jai` that exist (all of them
//!    are tried as entries).
//! 3. A document no entry reaches belongs to the program of the file that `#load`s it, walking
//!    up the loads among the files under the enclosing stdlib tree or the project folder
//!    (`auto_import.rs` `loader_chain`); a file nothing loads is its own entry.
use std::path::{Component, Path};

pub use jaic::project::*;

pub const CONFIG_FILE: &str = jaic::project::JAI_TOML;

pub use jaic::paths::normalize;

/// `target` relative to the folder `from`, with `/` separators (`../util/strings.jai`).
pub fn relative(from: &Path, target: &Path) -> String {
    let from: Vec<Component> = from.components().collect();
    let to: Vec<Component> = target.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".into(); from.len() - common];
    parts.extend(
        to[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths() {
        assert_eq!(
            relative(Path::new("/p/src"), Path::new("/p/src/ui/draw.jai")),
            "ui/draw.jai"
        );
        assert_eq!(
            relative(Path::new("/p/src/ui"), Path::new("/p/util.jai")),
            "../../util.jai"
        );
        assert_eq!(
            normalize(Path::new("/p/src/../a/./b.jai")),
            Path::new("/p/a/b.jai")
        );
    }

    /// The language server and the linter resolve the same root for the same tree where they are
    /// meant to agree (their tie-breaks are separate options of `jaic::project::RootOptions`).
    #[test]
    fn server_and_linter_agree_on_a_fixture_tree() {
        use crate::{Environment, Limits, Session};
        use jaic::project::{DiskFs, LoadCache, ProjectFs, RootOptions, resolve_root};
        let dir = std::env::temp_dir().join(format!("jailsp-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in [
            ("main.jai", "#load \"a.jai\";"),
            ("a.jai", "#load \"deep/b.jai\";"),
            ("deep/b.jai", ""),
            ("lone.jai", ""),
            ("Thing/module.jai", "#load \"part.jai\";"),
            ("Thing/part.jai", ""),
        ] {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let dir = DiskFs.key(&dir);
        let mut session = Session::with_environment(
            Limits::default(),
            Environment {
                fs: std::rc::Rc::new(jaic::sema::NativeFs),
                options: Box::new(|_| jaic::sema::Options::host()),
            },
        );
        session.set_workspace_folders(vec![dir.clone()]);
        let linter = RootOptions::linter(Vec::new());
        for name in [
            "deep/b.jai",
            "a.jai",
            "main.jai",
            "lone.jai",
            "Thing/part.jai",
        ] {
            let file = dir.join(name);
            let uri = crate::document::DocumentUri::from_path(&file).unwrap();
            let by_server = session.project_root(&uri).unwrap();
            let by_linter = resolve_root(&DiskFs, &file, &linter, &mut LoadCache::default());
            assert_eq!(by_server, by_linter.path(), "{name}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
