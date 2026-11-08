# Project indexing in the language server

The language server checks source files in their enclosing program, including files loaded by other sources and programs created by a build metaprogram. Opening a fragment first must give it the same context as opening the entry first.

## How it works

Project entries come from `[project] entry` in `jacket.toml`, or the existing inferred entry names (`build.jai`, `first.jai`, `main.jai`, `src/main.jai`, `module.jai`). The server indexes entries while idle, recording the actual sources, generated strings and import directories supplied by their metaprograms. Each entry's compiler checks its inclusion graph. The recorded graph includes imports, `#import,file`, and child workspaces; lexical load closures remain separate because imported names have different visibility. Inferred entries already included by an earlier entry are not checked again as standalone programs. Explicit configured entries remain independent.

An included file uses that recorded program context. Unrecorded inferred entries are probed too, so a file can find its enclosing metaprogram before any other source is open. New build edges invalidate cached loader graphs. Changed loader files are rescanned before a cached graph is used, including when a removed load is restored. Entry selection starts from the requested file, so open loaders cannot change its project context. Source stamps and filesystem dependencies validate recorded graphs after changes. Opening or closing unchanged disk text preserves analyses and discovery caches, including when a preview tab closes one file while opening another. Closing a compiled buffer reuses its analysis only when its former bytes match disk; discarding unsaved edits still requires checking. Buffer edits retain older analyses for responsive requests, while disk notifications invalidate affected analyses. Directory enumeration records dependencies and includes virtual open files. Interrupted indexing leaves the entry pending. Opening a buffer whose exact bytes already have recorded editor facts reuses the indexed analysis, including facts in child workspaces. Newly opened sources without those facts still require checking. Reading an unopened source uses the compiler's replacement decoding for invalid UTF-8, so opening the file cannot change whether a load is considered present. For files that were only tested for existence, reuse also requires proof that they were already present and the new buffer exactly matches disk.

After discovering the graph, the server checks diagnostics for every included workspace file, including closed files. Lints run in batches per program and configuration so interprocedural summaries are shared. Completed snapshots serve workspace diagnostic pulls or push publications, keep explorer warnings when files close, and let a newly opened matching buffer reuse diagnostics immediately. Edits invalidate affected snapshots; interrupted batches never publish partial results. Indexing remains active until this diagnostics pass finishes.

The native transport uses standard `window/workDoneProgress/create` and `$/progress` messages. It waits for the client's token acknowledgement, then sends a begin notification before checking. Its label is `jailsp: indexing 128 files`, using the discovered workspace file count, and it disappears when project checks finish. Publishing cached diagnostics or summarizing code lenses does not start the indexing indicator. File totals become known after build metaprograms discover their inclusion graphs. VS Code and Neovim can display these through their LSP progress integration. Clients that do not support work-done progress keep receiving normal language features.

## How to change it

`session/indexing.rs` schedules entries, and `session/workspace_diagnostics.rs` caches whole-program diagnostic batches. `auto_import/programs.rs` selects their owning program, and `session/compile.rs` records build facts. Keep compiler-recorded edges authoritative and invalidate derived loader graphs when those edges change.

Lifetime lint queries memoize shared temporary-value paths by value and depth; write-summary propagation checks for interruption so edits can stop the diagnostics batch.

`protocol/progress.rs` owns the progress lifecycle; `main.rs` flushes its begin notification before starting idle work. Progress must not block requests while indexing is interrupted by new input. Feature refreshes are sent when indexing completes, rather than after each attempted or interrupted step: a refresh response is itself input and must not restart the same unfinished work. Checks already completed for code lenses also satisfy indexing. Background workspace diagnostic polls and server-request acknowledgements wait for ongoing checks; interactive messages interrupt them. Framing happens on the input thread so partial frames cannot cause repeated restarts. The native transport registers cancellations for the queued batch, then flushes each reply as it finishes; an expensive diagnostic request cannot hold back hints and tokens already answered earlier in that batch.

Member completion first uses receiver types from the indexed program. A placeholder compile is needed only when that context is missing. Known built-in types and members have no source definition and do not trigger cross-target searches for one. Public type-alias signatures are resolved before module completion so their kind is stable.

Use the [graph audit](lsp-graph-audit.md) to check every included repository file in several open orders.

## Configuration

```toml
[project]
entry = ["first.jai"]
import_path = ["modules"]
```

Explicit entries override inference. Existing compile-time budgets and workspace trust settings still apply. The checker has no executable output backend; indexing follows build metaprograms without linking their output.

## Dependencies

`jaic::project`, the shared lint compiler and sandbox, the semantic analysis cache, workspace build observers, and the client's standard LSP work-done progress support.
