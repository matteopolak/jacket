//! Helpers shared by the language-server integration tests. Each test binary pulls in the
//! subset it uses, so unused items are expected.
#![allow(dead_code, reason = "each test binary uses a subset of these helpers")]

use jai_language_server::{DocumentUri, Environment, Limits, Position, Session};
use jaic::sema::TargetOs;
use std::path::{Path, PathBuf};

fn stdlib_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
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

/// Opens `file` of `folder` in a session over [`environment`] rooted at `folder`.
pub fn open_file(folder: &Path, file: &str) -> (Session, DocumentUri, String) {
    let mut s = session();
    s.set_workspace_folders(vec![folder.to_path_buf()]);
    let path = folder.join(file);
    let uri = uri_of(&path);
    let text = std::fs::read_to_string(&path).unwrap();
    s.open(uri.clone(), 1, text.clone()).unwrap();
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
    prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32
}

/// The UTF-16 length of `text`.
pub fn utf16_len(text: &str) -> u32 {
    text.encode_utf16().count() as u32
}

/// Position of the first `marker` in `text`.
pub fn at(text: &str, marker: &str) -> Position {
    at_offset(text, marker, 0)
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
