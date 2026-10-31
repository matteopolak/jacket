# Standard-library index

A process-wide index of what the standard library's modules declare, so the "import it" help for an unknown name or operator is a lookup instead of parsing the whole library. It lives in `crates/jaic/src/sema/stdlib_index.rs`; `sema/suggestions.rs` is its only user.

## How it works

- `StdlibIndex::shared(fs, stdlib_root)` reuses an index from an earlier compile while it is still current, else builds one. Up to four are kept.
- Build: lists modules (`prelude`, `Preload`, `Runtime_Support` excluded), then line-scans each `.jai` file (skipping `tests`, `examples`, `modules`) for top-level declarations, `operator` declarations, and `#import`s used for ranking.
- A name found by the scan is confirmed by parsing its file once, which decides what is really exported (`#scope_export`, `#scope_module`, top-level `#if`).
- Per compile, `StdlibView` drops the preloaded module. `Compiler::imports_declaring*` rank results (project imports first, then stdlib importers, `Extensions/` last) and cap them at `MAX_IMPORTS`.
- Invalidation: every path the build asked the file system about is recorded with its `FileSystem::stamp` (size and mtime on disk, content hash for virtual and overlay files). The index is reused while all stamps match. A `FileSystem` returning `None` for stamps gets no sharing.

## How to change it

- A new fact about modules goes in `add_file`: keep the scan cheap and parse lazily like `file_exports`.
- Answers must stay per module (a union over its files), with the same rules as before: declarations outside `#scope_file`, names of letters, digits and `_`, operators without blanks or `::`.
- A `FileSystem` wrapper should forward `stamp` and include whatever it overlays, or it opts out of sharing.

The stdlib folder comes from `Options::stdlib_root()`.
