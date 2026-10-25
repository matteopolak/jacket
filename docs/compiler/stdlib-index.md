# Standard-library index

**What it is.** A process-wide index of what the standard library's modules declare, so the "import it" help of an unknown name or operator is a lookup instead of a read-and-parse of the whole library. It lives in `crates/jaic/src/sema/stdlib_index.rs`; `sema/suggestions.rs` is its only user.

## How it works

- `StdlibIndex::shared(fs, stdlib_root)` returns an index kept from an earlier compile when it is still current, else builds one. Up to four are kept (one per stdlib folder, plus a changed one).
- **Build.** Lists the modules (`Name.jai`, `Name/module.jai`, and `Extensions/...`; `prelude`, `Preload` and `Runtime_Support` are left out) and reads every `.jai` file of each, skipping `tests`, `examples` and `modules` folders. One line scan per file records:
  - the names that start a line outside `#scope_file`, indented at most four spaces, as `name ::`, `name :=` or `name: T` (`declaring`);
  - the `operator <op> ::` declarations with the words of each parameter list (`operators`);
  - the modules each module `#import`s (for ranking).
- **Parse on demand.** A name found by the line scan is confirmed by parsing its file once (`Parsed`, an `OnceLock` per file): the parse decides which top-level names are really exported (`#scope_export` / `#scope_module` / top-level `#if`). A file that does not parse counts every declaration-looking line.
- **Per compile.** `StdlibView` is the index minus the module the compile preloads (`Options::preload`), cached in `NameCaches::stdlib_view`. `Compiler::imports_declaring*` filter, rank (`ModuleRanking`, project imports first, then stdlib importers, `Extensions/` last, ties by name), and cap the answers at `MAX_IMPORTS`. The module named like the wanted name still comes first.

## Invalidation

Every path the build asked the file system about (folders listed, files read, `module.jai` probes) is recorded with its `FileSystem::stamp`. An index is reused while all stamps are unchanged; that is checked once per compile, on the first suggestion that needs the index.

- `NativeFs`: size and modification time.
- `VirtualFs`: a hash of the contents (memoised until the next `insert`) and of a folder's names.
- The language server's `OverlayFs`: a hash of the open buffer, else the disk file's stamp. It also records the path as read, so editing a stdlib file in the editor re-checks the documents.
- Any other `FileSystem` returns `None` (the default): the index then serves the compile that built it and is dropped.

## How to change it

- A new fact about modules (say, exported types) belongs in `add_file`; keep the line scan cheap and parse lazily like `file_exports` does.
- Anything the old per-compile scan decided per file is now per module (a union over its files); the answers must stay equal. `modules_exporting_name` and `modules_exporting_operator` keep the same rules: only modules with a matching declaration outside `#scope_file`, a name written with letters, digits and `_` only (any other text declares nothing in the library), an operator without blanks or `::`.
- A new `FileSystem` implementation that wraps another should forward `stamp` (and include whatever it overlays), or it opts out of sharing.

## Configuration

None. The stdlib folder comes from `Options::stdlib_root()`.

## Dependencies

`FileSystem` (`sema/modules/filesystem.rs`), `ModuleRanking` / `count_importers` (`sema/module_rank.rs`), the parser and lexer, and `NameCaches` for the per-compile memo.
