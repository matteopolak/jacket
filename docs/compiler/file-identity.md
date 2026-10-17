# File identity

## What it is

Which file a path names is decided by one key, `FileSystem::canonical` (`crates/jaic/src/sema/modules.rs`), never by how the path is spelled. `./src/main.jai`, `src//main.jai`, `src/../src/main.jai`, an absolute path through a symlink (or `/var` for `/private/var`), and the editor's other-case spelling on a case-insensitive disk are one file.

## How it works

`canonical` normalises lexically (`.`, `..`, duplicate separators), then, when the file exists, resolves it on disk (`jaic::canonicalize`: symlinks, on-disk case, Windows `\\?\` prefixes stripped). A file that is not on disk (an unsaved editor buffer, the browser's virtual file system) keeps its lexical form. The language server's overlay file system (`OverlayFs`) also folds Windows drive letters and separators (`document::path_key`), so open documents and disk files meet in the same key.

The key is used for:

- loaded-file deduplication (`Compiler::load_file`, which also records the canonical path as the source's path),
- mapping an open document to the compiled `FileId` (`find_file` in `semantic.rs`),
- deciding which files record editor facts (`Compiler::ide_wants`): files under the root file's folder, plus every open document wherever it is (`IdeFacts::with_files`). A build file in `tools/build.jai` that adds `../src/main.jai` therefore still gives `src/main.jai` hover, goto and diagnostics.

Display keeps the spelling the user wrote (diagnostic text, `#filepath`).

## How to change it

Compare paths through `canonical`, not with `==` or `starts_with` on strings. Do not add separator or prefix fixes at a comparison site; fix `canonical`. Virtual sources (`<input>`) have no path on disk and compare as written.

## Configuration

None.

## Dependencies

`std::fs::canonicalize` through `jaic::canonicalize`; the language server's `OverlayFs`. Tests: `crates/jai-language-server/tests/build_file_entries.rs` (each spelling above, a symlinked project folder, a case variant, a build file in a subfolder).
