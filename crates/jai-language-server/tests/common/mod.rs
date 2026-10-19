//! Helpers shared by the language-server integration tests. Each test binary pulls in the
//! subset it uses, so unused items are expected.
#![allow(dead_code, reason = "each test binary uses a subset of these helpers")]

use jai_language_server::{DocumentUri, Environment, Limits, Position, Range, Session};
use jaic::sema::{Options, TargetOs, VirtualFs};
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn stdlib_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
}

/// The repository's stdlib directory, canonical.
pub fn stdlib() -> PathBuf {
    stdlib_dir().canonicalize().unwrap()
}

/// An environment that type-checks against the repository's stdlib.
pub fn environment() -> Environment {
    let stdlib = stdlib_dir();
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

/// [`environment`] plus a `modules` directory beside the main file, optionally for another OS.
/// `JAIC_STDLIB` overrides the stdlib location.
pub fn modules_environment(os: Option<TargetOs>) -> Environment {
    let stdlib = std::env::var_os("JAIC_STDLIB").map_or_else(stdlib_dir, PathBuf::from);
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |main| {
            let mut options = jaic::sema::Options::host();
            if let Some(os) = os {
                options.os = os;
            }
            let dir = main.parent().map(PathBuf::from).unwrap_or_default();
            options.import_paths = vec![dir.join("modules"), stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

/// An environment without the runtime support or stack traces, so no stdlib is needed.
pub fn lean_environment() -> Environment {
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(|_| {
            let mut options = jaic::sema::Options::host();
            options.runtime_support = false;
            options.stack_trace = false;
            options
        }),
    }
}

/// A session over an in-memory file system holding `files` (absolute path, text), without the
/// runtime support or stack traces. `import_paths` are the module search directories and
/// `preload` the file every program starts from, if any.
pub fn virtual_session(
    files: &[(&str, &str)],
    import_paths: &[&str],
    preload: Option<&str>,
) -> Session {
    let mut fs = VirtualFs::default();
    for (path, text) in files {
        fs.insert(path, text.as_bytes());
    }
    let import_paths: Vec<PathBuf> = import_paths.iter().map(PathBuf::from).collect();
    let preload = preload.map(PathBuf::from);
    Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(fs),
            options: Box::new(move |_| {
                let mut options = Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options.import_paths.clone_from(&import_paths);
                options.preload.clone_from(&preload);
                options
            }),
        },
    )
}

/// A session over [`environment`].
pub fn session() -> Session {
    Session::with_environment(Limits::default(), environment())
}

/// A session over [`lean_environment`].
pub fn lean_session() -> Session {
    Session::with_environment(Limits::default(), lean_environment())
}

/// The document URI single-file tests open their text under.
pub fn uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-test/main.jai").unwrap()
}

/// The `file://` URI of absolute `path` (`/project/main.jai`) that need not exist on disk.
pub fn file_uri(path: &str) -> DocumentUri {
    DocumentUri::parse(&format!("file://{path}")).unwrap()
}

/// `source` without its first `|`, and the position where the `|` was: the cursor of a
/// completion or hover test.
pub fn cursor(source: &str) -> (String, Position) {
    let byte = source.find('|').expect("no `|` cursor marker");
    (
        source.replacen('|', "", 1),
        position(&source.replacen('|', "", 1), byte),
    )
}

/// The text of `range`, whose columns are UTF-16 units.
pub fn text_in_range(text: &str, range: Range) -> String {
    let byte = |at: Position| -> usize {
        let start: usize = text
            .split_inclusive('\n')
            .take(at.line as usize)
            .map(str::len)
            .sum();
        let line = &text[start..];
        let mut units = 0;
        for (index, ch) in line.char_indices() {
            if units >= at.character {
                return start + index;
            }
            units += ch.len_utf16() as u32;
        }
        start + line.len()
    };
    text[byte(range.start)..byte(range.end)].to_string()
}

/// A `file://` URI for a path on disk.
pub fn uri_of(path: &Path) -> DocumentUri {
    DocumentUri::from_path(path).unwrap()
}

/// A session over [`environment`] with `text` open at [`uri`].
pub fn open(text: &str) -> Session {
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    s
}

/// Writes `files` under a fresh temporary directory named after `name` and returns its
/// canonical path.
pub fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jailsp-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
}

/// A session over [`environment`] rooted at `folder`.
pub fn workspace_session(folder: &Path) -> Session {
    let mut s = session();
    s.set_workspace_folders(vec![folder.to_path_buf()]);
    s
}

/// Opens `file` of `folder` in `s`, from disk.
pub fn open_from_disk(s: &mut Session, folder: &Path, file: &str) -> (DocumentUri, String) {
    let path = folder.join(file);
    let uri = uri_of(&path);
    let text = std::fs::read_to_string(&path).unwrap();
    s.open(uri.clone(), 1, text.clone()).unwrap();
    (uri, text)
}

/// Opens `file` of `folder` in a session over [`environment`] rooted at `folder`.
pub fn open_file(folder: &Path, file: &str) -> (Session, DocumentUri, String) {
    let mut s = workspace_session(folder);
    let (uri, text) = open_from_disk(&mut s, folder, file);
    (s, uri, text)
}

/// The LSP position (UTF-16 columns) of byte offset `byte` in `text`.
pub fn position(text: &str, byte: usize) -> Position {
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: utf16_column(prefix),
    }
}

/// The UTF-16 length of the last line of `prefix`, i.e. the LSP column at its end.
pub fn utf16_column(prefix: &str) -> u32 {
    jaic::source::utf16_len(prefix.rsplit('\n').next().unwrap())
}

/// The UTF-16 length of `text`.
pub fn utf16_len(text: &str) -> u32 {
    jaic::source::utf16_len(text)
}

/// Position of the first `marker` in `text`.
pub fn at(text: &str, marker: &str) -> Position {
    at_offset(text, marker, 0)
}

/// Position just after the first `marker` in `text`.
pub fn end_of(text: &str, marker: &str) -> Position {
    let byte = text.find(marker).unwrap_or_else(|| panic!("no {marker:?}")) + marker.len();
    position(text, byte)
}

/// Position `offset` bytes into the first `marker` in `text`.
pub fn at_offset(text: &str, marker: &str, offset: usize) -> Position {
    let byte = text.find(marker).unwrap_or_else(|| panic!("no {marker:?}")) + offset;
    position(text, byte)
}

/// Position `offset` bytes into the `n`th (0-based) `marker` in `text`.
pub fn at_nth(text: &str, marker: &str, n: usize, offset: usize) -> Position {
    let byte = text
        .match_indices(marker)
        .nth(n)
        .unwrap_or_else(|| panic!("no occurrence {n} of {marker:?}"))
        .0
        + offset;
    position(text, byte)
}

/// Position of the last `marker` in `text`.
pub fn at_last(text: &str, marker: &str) -> Position {
    position(
        text,
        text.rfind(marker)
            .unwrap_or_else(|| panic!("no {marker:?}")),
    )
}
