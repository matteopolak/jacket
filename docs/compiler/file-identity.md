# File identity

Which file a path names is decided by one key, `FileSystem::canonical` (`crates/jaic/src/sema/modules.rs`), never by spelling. `./src/main.jai`, `src//main.jai`, `src/../src/main.jai`, a symlinked path, and a differently cased spelling on a case-insensitive disk are one file.

## How it works

- `canonical` normalises lexically, then resolves on disk when the file exists (symlinks, on-disk case, no `\\?\` prefix). Files not on disk (unsaved buffers, the browser file system) keep their lexical form.
- The language server's `OverlayFs` also folds Windows drive letters and separators, so open documents meet disk files in the same key.
- The key deduplicates loaded files, maps open documents to compiled files, and decides which files record editor facts: everything under the root file's folder plus all open documents. A build file adding `../src/main.jai` still gives that file hover and diagnostics.
- Display keeps the user's spelling (diagnostics, `#filepath`).

## How to change it

Path helpers live in `crates/jaic-syntax/src/paths.rs`, shared by `jaic`, `jailsp` and `jailint`: `file_identity`, `normalize`, `canonicalize`, `CanonCache`, `default_stdlib_dir()`. Add new ones there.

Compare paths through `canonical`, never `==` or `starts_with` on strings, and fix `canonical` rather than patching a comparison site. Virtual sources (`<input>`) compare as written.

Tests: `crates/jai-language-server/tests/build_file_entries.rs`.
