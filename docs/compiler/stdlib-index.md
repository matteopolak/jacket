# Standard-library index

A process-wide index of what the standard library's modules declare, so the "import it" help for an unknown name or operator is a lookup instead of parsing the whole library. It lives in `crates/jaic/src/sema/stdlib_index.rs`; `sema/suggestions.rs` is its only user.

## How it works

- `StdlibIndex::shared(fs, stdlib_root)` reuses an index from an earlier compile while it is still current, else builds one. Up to four are kept.
- Build: lists modules (`prelude`, `Preload`, `Runtime_Support` excluded), then token-scans each `.jai` file (skipping `tests`, `examples`, `modules`) for declaration candidates and `#import`s used for ranking. Candidates include declarations sharing a physical line. Only operator-bearing files are parsed during this scan; their exported operator headers come from the existing parser, including multiline parameters and inline scope directives.
- A name found by the scan is confirmed by parsing its file once, which decides what is really exported (`#scope_export`, `#scope_module`, top-level `#if`). Comments, strings and nested declarations cannot become exports of a valid file. Malformed files retain the older line-based advisory fallback; extra token candidates are not accepted without parse validation.
- Per compile, `StdlibView` drops the preloaded module. `Compiler::imports_declaring*` rank results (project imports first, then stdlib importers, `Extensions/` last) and cap them at `MAX_IMPORTS`.
- Project popularity counts already-resolved import sites in the main module and its file scopes, including files added by `#load`. Anonymous imports, named imports and named `using` imports count once per source span; repeated sites in different files count separately. Transitive imports, alias constants and imports nested in procedures do not count. Ranking neither loads nor resolves an unused named import.
- Invalidation: every path the build asked the file system about is recorded with its `FileSystem::stamp` (size and mtime on disk, content hash for virtual and overlay files). The index is reused while all stamps match. A `FileSystem` returning `None` for stamps gets no sharing.

## How to change it

- A new fact about modules goes in `add_file`: keep ordinary name files lazy like `file_exports`, and use the existing parser for operator declarations rather than inventing statement boundaries. Keep the malformed-file fallback separate from validated exports.
- Answers must stay per module (a union over its files), with the same rules as before: declarations outside `#scope_file`, names of letters, digits and `_`, operators without blanks or `::`.
- A `FileSystem` wrapper should forward `stamp` and include whatever it overlays, or it opts out of sharing.

The stdlib folder comes from `Options::stdlib_root()`.

## Caller-owned diagnostic hints

`Compiler::with_name_suggestion_in` accepts a separate filesystem and `HintContext`. It keeps render-time directory scans, neighboring builder reads, and module-name suggestions separate from the compiler's resolution inputs. For a prepared compiler and diagnostic:

```rust
let context = jaic::sema::HintContext::default();
let enriched = compiler.with_name_suggestion_in(&diagnostic, &context, observed_fs);
```

This context builds a fresh index instead of borrowing the process-wide stamp-based index or its lazy parsed cells. Batch errors and warnings through one context, capture every actual response through the supplied filesystem, then discard the context. Any retained enriched output must own and validate that captured proof; the context does not establish freshness by itself. Context reuse additionally requires the same compiler, options and unchanged compiled facts, including import-ranking facts. The LSP uses an ephemeral context per batch.

The default `with_name_suggestion` and compiler rendering still use the existing compiler filesystem and caches. Module-not-found hints embedded during checking keep their original input authority. Missing-operator diagnostics retain only owned operator and operand/home-module names until rendering. Discarded overload or dead-branch diagnostics therefore never search the external import index. When extending suggestions, route any new external reads through the explicit hint inputs and test both filesystem separation and unchanged default rendering.

`Options::hint_index_mode` selects the default rendering/checking index policy before compiler construction. `Shared` preserves the CLI's existing process-wide fingerprint cache. Exact observed callers use `PerCompiler`: the first index build records actual source responses through that compiler's filesystem, and its view remains local to that compiler. The LSP forces this mode before `LintCompile::start`, including the options cloned by workspace children and host twins. This also covers legacy compiler/build rendering. Deferred operator help uses the same current render-context import ranking as unknown-name help, rather than preserving a cache-history-dependent ranking from when an error was created. `Diagnostic::render` renders prepared details; use `Compiler::render` when compiler-derived help is required. Clearing suggestions removes pending operator fixes too; enriching an already prepared diagnostic cannot duplicate them.
