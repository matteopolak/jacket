# Browser editor

## What it is

The standalone Jai playground uses a locally bundled CodeMirror 6 editor beside a hierarchical source explorer. It supports syntax coloring, line numbers, bracket matching and pairing, automatic indentation, multiple selections, per-file undo/redo, find, and a diagnostic gutter.

## How it works

The stateful Jai language definition in `tools/browser-editor/jai-language.mjs` tracks nested block comments, quoted strings, escaped characters and `#string` terminators across lines. It colors source directives, notes, procedure declarations and built-in types without inserting source as HTML. Coloring is lexical; it does not claim to infer Jai types.

Each source file retains its own immutable CodeMirror state, selection and history when another file is selected. `workspace.mjs` remains the authority for normalized relative paths, source versions and immutable execution snapshots. The explorer derives folders from these real source identities. File names, hover contents and diagnostics render through text nodes.

A separate language worker exchanges standard JSON-RPC messages with the same bounded `jai-language-server::JsonSession` used by the native server. The browser uses `file:///jai-script/<relative-path>` identities and UTF-16 positions. Open/change/close notifications keep the service synchronized; source versions reject stale diagnostics, completion responses and hover responses. Completion and hover display only results returned by the core. File URI segments use canonical RFC3986 escapes, including punctuation retained by JavaScript’s default URI encoder. A rejected open/change notification arrives as an actual server error log; the client stops the language session, rejects pending requests, and ignores late replies so it cannot display retained old-source results as current. Runtime execution remains available through its independent worker. Its initial capabilities describe authentic lexer/parser analysis and admitted declarations, not full semantic type checking or source execution.

The execution worker owns the real Rust-generated WebAssembly interpreter. Run takes one immutable workspace snapshot. Cancel terminates that execution worker; editing and the independent language worker remain responsive. A small bootstrap module handles editor import failures before reporting them to the embed host. Initialization checks actual Wasm exports. An embedded page reports readiness only after its editor and compiler worker initialize, and language initialization completes when the bundle exposes it.

## Source workspace

`workspace.mjs` owns parsed relative file identities and immutable source records. Both the JavaScript workspace and Rust bundle use explicit POSIX path normalization independent of the host OS. Host `Path::is_absolute()` is unsuitable on `wasm32-unknown-unknown`, so normalization is explicit. aliases resolve to one name, paths reject backslashes, colons and escapes from the workspace, and names must fit the bridge's UTF-8 boundary. A private factory token prevents constructing an unchecked `SourcePath`. The entry is always present and cannot be overwritten by adding an alias. Folder nodes derive from actual file paths; they do not bypass path validation.

Each changed source receives a strictly increasing version for language service synchronization. Adding or removing files sends actual open/close notifications. The editor retains each file's selection and undo history while switching. A run takes one immutable snapshot before sending source, additional files, arguments and fuel to its execution worker. Later edits affect the next run. Cancel terminates that worker; the next run creates a fresh interpreter, while the separate language worker remains available.

For example, put `answer :: () -> int { return 42; }` in `lib/helper.jai` and use `#load "lib/helper.jai";` from the entry. The explorer shows a `lib` folder with the actual helper source. Source names and contents cross pointer-free WebAssembly channels. Rust still enforces its aggregate source quota and closed virtual filesystem.

Workspace behavior lives in `web/scripting-runtime/workspace.mjs`; interactions in `editor.mjs`; layout in `style.css`. Tests: `tools/test_browser_workspace.mjs` (identity, alias collisions, tree, versions, snapshots) and `tools/test_jai_editor.mjs` (lexer, history, language transport). The entry is `main.jai`; names are limited to 4,096 UTF-8 bytes and the Rust bundle limits aggregate source to 4 MiB. Refreshing starts a new workspace.

## How to change it

Change editor extensions and the visual theme in `tools/browser-editor/editor-kit.mjs`; modify lexical states in `jai-language.mjs`. Keep the language service protocol in `web/scripting-runtime/lsp-client.mjs`, scalar Wasm exports in `jai-wasm/src/language_server.rs`, and byte-channel decoding in `engine.mjs` paired with the portable server core. Requests and responses stay bounded; never interpret server strings as HTML or guess malformed ranges.

Run `npm ci --ignore-scripts`, then `npm run build` and `npm test`. The build emits `web/scripting-runtime/editor.bundle.mjs` and `THIRD-PARTY-NOTICES.txt`. Commit the generated bundle with its source and exact lockfile. Regeneration needs Node and the locked dependencies; using the published playground needs no npm install or CDN. `node tools/check_editor_dependencies.mjs` verifies official npm release timestamps against the fixed release-age cutoff and writes a review receipt.

Check actual file switching, selections, undo/redo, find, nested paths, execution and cancellation in a browser. Core and Wasm tests establish different evidence from UI tests. `node tools/check_playground_worker.mjs <staged-directory>` runs the unchanged worker with a Node host adapter and the real compiled Wasm, checking multi-file snapshots, separate language state, actual termination and restart. It does not mock compiler success or claim rendered-browser evidence. 

## Configuration

Tab uses four-space indentation; Ctrl/Command+F opens find, Ctrl/Command+Z undoes, and Ctrl/Command+Enter runs `main.jai`. Run options retain arguments and the unsigned 32-bit interpreter step limit. The bridge admits at most 1 MiB of language request bytes and 2 MiB of response bytes. The core additionally limits each document to 256 KiB, with document-count, workspace and analysis limits. Larger runtime sources may remain inside the 4 MiB VFS budget but lose language service after actual admission refusal; the UI reports this failure instead of treating the edit as synchronized. Reloading creates a fresh language session.

The release producer writes `release.json` with a full compiler commit. All assets resolve relative to their module URL, including workers and Wasm. In `?embed=1` mode, `parent.postMessage({type:'jai-playground',state:'ready'|'error',revision:<full-sha>,message?}, location.origin)` reports actual initialization. A missing or invalid release revision is an embedded initialization error. There are no external runtime URLs or persisted browser sources.

## Dependencies

CodeMirror state, view, language, commands, autocomplete, search and lint packages provide editor behavior; Lezer highlight provides token styles. Exact package and transitive versions are locked, checked to be at least fourteen days old as of 2026-10-03, and bundled with esbuild. Execution depends on the actual `jai_wasm.wasm`; language features require the paired portable `jai-language-server` core. See [playground](playground.md).
