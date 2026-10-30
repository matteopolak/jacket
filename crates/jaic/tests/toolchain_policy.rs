//! The C runtime is described once, in `crates/jaic/src/toolchain/`. No other source file names
//! one of its libraries, or a symbol it renames; they ask the
//! `Toolchain` instead (see `docs/compiler/toolchain.md`).
use std::path::{Path, PathBuf};

/// Spellings that belong to the toolchain description only.
const FORBIDDEN: &[&str] = &[
    "msvcrt",
    "ucrt",
    "vcruntime",
    "libcmt",
    "legacy_stdio",
    "oldnames",
    "msvc_double_math",
    // Stems of the `<math.h>` functions whose `long double` forms the MSVC runtime renames.
    "nextafter",
    "tgamma",
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn only_the_toolchain_module_names_the_c_runtime() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let toolchain = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/toolchain");
    let module = toolchain.with_extension("rs");
    let mut files = Vec::new();
    for krate in std::fs::read_dir(crates).unwrap().flatten() {
        let src = krate.path().join("src");
        if src.is_dir() {
            sources(&src, &mut files);
        }
    }
    assert!(files.len() > 100, "found only {} sources", files.len());
    let mut found = Vec::new();
    for file in files {
        if file == module || file.starts_with(&toolchain) {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        for (number, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if let Some(word) = FORBIDDEN.iter().find(|word| line.contains(**word)) {
                found.push(format!(
                    "{}:{}: `{word}`",
                    file.strip_prefix(crates).unwrap().display(),
                    number + 1
                ));
            }
        }
    }
    assert!(
        found.is_empty(),
        "these name the C runtime outside the toolchain module; add it to `Toolchain` and ask that:\n{}",
        found.join("\n")
    );
}
