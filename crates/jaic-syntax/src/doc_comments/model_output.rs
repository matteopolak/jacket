//! Public compiler documentation command output and cross-module doc-link destinations.
use super::{module, to_json};
use std::collections::BTreeSet;
use std::path::Path;

/// What `jaic -doc_model` prints for the library in `dir`. Links in docs to a name of the
/// library (or `Module.name` of another library of `stdlib`) become `jaidoc:Module/name` and
/// `jaidoc:Module/name#member`, for the renderer to turn into pages.
pub fn print(dir: &Path, stdlib: &Path) -> String {
    let (dir, stdlib) = (
        &super::super::canonical(dir),
        &super::super::canonical(stdlib),
    );
    let model = module(dir, stdlib);
    let key = {
        let canonical = super::super::canonical(dir);
        let root = super::super::canonical(stdlib);
        canonical.strip_prefix(&root).map_or_else(
            |_| {
                canonical
                    .file_name()
                    .map_or(String::new(), |n| n.to_string_lossy().into_owned())
            },
            crate::paths::module_key,
        )
    };
    let own: BTreeSet<&str> = model
        .items
        .iter()
        .filter(|i| !i.private)
        .map(|i| i.name.as_str())
        .collect();
    let resolve = |names: &[String]| -> Option<String> {
        let first = names.first()?;
        if own.contains(first.as_str()) {
            return Some(match names.get(1) {
                Some(member) => format!("jaidoc:{key}/{first}#{member}"),
                None => format!("jaidoc:{key}/{first}"),
            });
        }
        let other = names.get(1)?;
        let module_dir = stdlib.join(first);
        let ext_dir = stdlib.join("Extensions").join(first);
        if crate::paths::module_entry(&module_dir).is_file() {
            return Some(format!("jaidoc:{first}/{other}"));
        }
        if crate::paths::module_entry(&ext_dir).is_file() {
            return Some(format!("jaidoc:Extensions/{first}/{other}"));
        }
        None
    };
    to_json(&model, &resolve)
}
