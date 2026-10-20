//! Opening real source files for the typing-session checker (`typing_session.rs`): the file, the
//! sibling files it loads or imports by relative path, and a `jai.toml` that makes it the entry.
use jai_language_server::{DocumentUri, Environment};
use std::path::{Path, PathBuf};

pub struct FileInput {
    pub setup: Vec<(DocumentUri, String)>,
    pub docs: Vec<(DocumentUri, String)>,
}

/// The environment `jailsp` itself runs with: modules next to the main file, then `stdlib`.
pub fn native_environment(stdlib: PathBuf) -> Environment {
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |main| {
            let mut options = jaic::sema::Options::host();
            let dir = main.parent().map(PathBuf::from).unwrap_or_default();
            options.import_paths = vec![dir.join("modules"), stdlib.clone()];
            options.preload = Some(
                jaic::find_preload(&jaic::sema::NativeFs, &stdlib)
                    .unwrap_or_else(|| stdlib.join(jaic::PRELOAD_ENTRY)),
            );
            options
        }),
    }
}

/// Files next to `path` named by a quoted `#load` or `#import` string.
fn sibling_files(path: &Path, text: &str) -> Vec<PathBuf> {
    let dir = path.parent().unwrap_or(Path::new("."));
    // Names as the directory spells them: a case-insensitive file system would otherwise turn
    // `#import "IntroSort"` into the file `introsort.jai` itself.
    let names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    let mut out = Vec::new();
    for directive in ["#load \"", "#import \""] {
        for (at, _) in text.match_indices(directive) {
            let rest = &text[at + directive.len()..];
            let name = &rest[..rest.find('"').unwrap_or(0)];
            let found = [name.to_string(), format!("{name}.jai")]
                .into_iter()
                .filter(|n| n.contains('/') || names.contains(n))
                .map(|n| dir.join(n))
                .find(|p| p.is_file() && p.extension().is_some_and(|e| e == "jai") && p != path);
            if let Some(found) = found
                && !out.contains(&found)
            {
                out.push(found);
            }
        }
    }
    out
}

/// `None` when the file cannot be read as UTF-8.
pub fn read_file_input(path: &Path) -> Option<FileInput> {
    let path = path.canonicalize().ok()?;
    let text = std::fs::read_to_string(&path).ok()?;
    let uri = |p: &Path| DocumentUri::from_path(p).ok();
    let mut docs = vec![(uri(&path)?, text.clone())];
    for sibling in sibling_files(&path, &text).into_iter().take(2) {
        let content = std::fs::read_to_string(&sibling).ok()?;
        docs.push((uri(&sibling)?, content));
    }
    let config = uri(&path.parent()?.join("jai.toml"))?;
    let entry = serde_json::to_string(path.file_name()?.to_str()?).ok()?;
    Some(FileInput {
        setup: vec![(config, format!("build_files = [{entry}]\n"))],
        docs,
    })
}
