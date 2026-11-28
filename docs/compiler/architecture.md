# Compiler architecture

How a build flows through the crates under `crates/`, and what each crate owns. [Features](../features.md) tracks what works.

## Crates

From the bottom up. The last column lists workspace crates, then the external crates that shape the boundary.

| Crate | Owns | Depends on |
| --- | --- | --- |
| `jacket-config` | `jacket.toml`, the environment-variable registry (`env.rs`), LLVM install discovery (`llvm.rs`), `locate` for sibling tools, OS-string splitting (`os_text.rs`) | `serde`, `serde_ignored`, `toml` |
| `jaic-syntax` | sources, spans, lexer, parser, AST, diagnostics and their renderer; no type checking | `jacket-config` |
| `jaic` | sema, IR, interpreter, build API, IDE services, the target model (`target.rs`, `toolchain.rs`) and Visual Studio / Windows SDK discovery (`toolchain/discover.rs`); re-exports `jaic-syntax` | `jaic-syntax`, `jacket-config`; `find-msvc-tools` |
| `jaic-toolchain` | finding linkers, archivers, sysroots, builtins, Xcode and native library folders; install notes; the toolchain cache | `jaic`, `jacket-config` |
| `jaic-native` | the link command, linker-argument translation, the direct macOS `ld` link, wasm and freestanding links | `jaic`, `jaic-toolchain`, `jacket-config` |
| `jaic-llvm` | LLVM code generation over the shared IR | `jaic`, `jaic-native`, `jacket-config`; `inkwell` |
| `jaic-backend` | `jaic-llvm` as the shared library (`libjaic_backend`) a release `jaic` opens on its first native build ([lazy backend](lazy-backend.md)) | `jaic-llvm`, `jaic-native` |
| `jaic-cli-args` | `jaic`'s option table and `Cli` ([command-line driver](command-line-driver.md)) | `jaic` |
| `jaic-cli` | the `jaic` binary | `jaic`, `jaic-cli-args`, `jaic-toolchain`, `jacket-config`; `jaic-native`, `jaic-llvm` behind `llvm` |
| `jailint` | lint rules and the `jailint` binary | `jaic`, `jacket-config` |
| `jai-language-server` | `jailsp` ([language server](language-server.md)) | `jaic`, `jailint`, `jacket-config` |
| `jai-wasm` | the browser playground and language server ([browser](../browser/playground.md)) | `jai-language-server`, `jaic`; `jacket-config` at build time |
| `jacket` | the `jacket` binary | `jaic`, `jailint`, `jaic-cli-args`, `jaic-toolchain`, `jacket-config` |
| `test-support` | shared test helpers | `jaic`, `jaic-toolchain`, `jacket-config`; optional `jaic-llvm`, `jai-language-server` |

Only the syntax layer is a separate crate; sema, interp and build call each other both ways.

## How it works

```
lexer -> parser (ast) -> sema (demand-driven checking + lowering) -> ir -> interp | jaic-llvm
```

- Keywords are ordinary identifiers the parser reads by position; directives (`#run`) and notes (`@x`) are their own tokens. The parser stops at the first error ([parser](parser.md)).
- Sema ([sema](sema.md)) checks a declaration when something needs it. Checking a body and lowering it to [IR](ir.md) are one pass.
- The [interpreter](interpreter.md) runs `#run`, metaprograms and `jaic -interp`. Its `Host` decides which `#foreign` calls exist; the browser uses `SandboxHost`.
- `ide/` records what each name refers to while checking, and the editor queries read those facts ([IDE facts](ide-facts.md)).
- `stdlib/` is plain Jai, loaded through `sema::FileSystem` (real or virtual).

### Threading

Sema is single-threaded. Parsing runs ahead on worker threads ([compile speed](compile-speed.md#parallel-lex-and-parse)), codegen runs one thread per unit, and `#run` code can start its own threads. The one `rayon` pool (`pool.rs`, sized by `-threads`) is shared by body scans, codegen units and `jailint`. Checking bodies on several threads was designed and shelved ([parallel type checking](parallel-sema.md)).

The symbol interner and `AstId` counter are process-wide, so their numbers follow thread timing. Nothing visible may depend on them (sort by spelling; `tools/differential/determinism.py --scramble` checks it). Workers parse with a placeholder file id and `ast::relocate` rewrites spans when sema claims the tree, so any new `Span` or `Ident` field in the AST must also be visited in `ast/relocate.rs`.

### Native and browser

`crates/jaic/src/platform.rs` names what differs between the native compiler and the browser (wasm32) build, so ordinary code doesn't carry its own `cfg(target_arch = "wasm32")`.

- `HAS_CLOCK` is false on wasm. `Stopwatch` is a real `Instant` natively and reads zero on wasm.
- Subsystems that exist on one side only are separate modules picked where they are declared: `parse_ahead`, `service`, the interpreter's `threads` versus `threads_inline`, `progress::start_ticker`, and `fiber`'s per-CPU assembly.
- The OS and CPU reported to `#run` code come from `compiler_host` in `sema/mod.rs`.

### The target model sits inside `jaic`

`jaic-toolchain` needs `Target`, `Linker` and `Driver`, which live in `jaic`, so it depends on `jaic` and `jaic` cannot use it. The interpreter opens Windows import libraries from the SDK's `LIB` folders, so `jaic` keeps the Visual Studio and SDK lookup (`toolchain/discover.rs`, on `find-msvc-tools`) itself. Everything else about finding tools is in `jaic-toolchain`.

The interpreter gets its library folders from the driver: `jaic-cli` calls `jaic::interp::set_library_dirs` with what `jaic-toolchain` found, and `set_missing_libraries` with the reason for any library it looked for and did not find (libclang on a Mac), which a failed open then reports.

### Path finding

Every lookup of a tool or SDK reads the machine through `jaic_toolchain::Host`, so a test describes a machine with `FakeHost`. Each one finds the tool or returns a `Missing` with an install note; none switches to another toolchain quietly.

| What | Where | How |
| --- | --- | --- |
| Programs on `PATH` and in install folders | `jaic-toolchain/src/lookup.rs` | `which`, `every_on_path`, `tool`, `llvm_tool` |
| LLVM installs | `jacket-config/src/llvm.rs` | `llvm-config`, `LLVM_PREFIX`, the baked prefix |
| Visual Studio, Windows SDK, `LIB` | `jaic/src/toolchain/discover.rs` | the `find-msvc-tools` crate (`vswhere`, the setup COM API, the registry) |
| Clang outside `PATH`; Microsoft's `link.exe` versus coreutils `link` | `jaic-toolchain/src/msvc.rs` | `Program Files`, then Visual Studio's Clang for the host CPU; the `link /?` banner |
| Xcode: `ld`, SDK, `libclang_rt.osx.a`, `libclang.dylib` | `jaic-toolchain/src/apple.rs` | the folder `xcode-select` selects (`DEVELOPER_DIR`, `/var/db/xcode_select_link`, `/Applications/Xcode.app`, the Command Line Tools), read as one layout |
| Built third-party libraries | `jaic-toolchain/src/native_libs.rs` | `JAIC_NATIVE_LIBS`, else `native-libs/<os>-<cpu>` under `JACKET_OUTPUT_DIR`, beside the stdlib, under `target/`, or in the main checkout of a git worktree |
| Linkers, archivers, sysroots, builtins | `jaic-toolchain/src/linker.rs`, `builtins.rs`, `cache.rs` | the order in [toolchain discovery](../native/toolchain-discovery.md) |

Xcode is read from its folders rather than asked through `xcrun`: no maintained crate finds it without starting `xcrun`, and starting it costs the 8 ms the direct `ld` link saves ([linking](../native/native-linking.md)).

### Arguments that are not text

Process arguments, paths and environment values are OS strings, and Clang and the linkers take any bytes. `jaic` reads its command line with `args_os`. `-link_arg`, `-linker`, `-sysroot` and `-compiler_rt` are `Action::Passthrough` options, stored as given, `-link_arg=VALUE` included. `jaic-native` translates and renders `OsString`s (`ExtraArgs` in `link_options.rs`, `freestanding.rs`, `wasm.rs`, `apple_ld.rs`) and rewrites only the ASCII spellings it knows. `jacket_config::os_text` splits an OS string at ASCII characters without decoding it. Any other command-line argument that is not UTF-8 is an error.

The metaprogram's `additional_linker_arguments` are Jai strings, held as `String` in `jaic::build::BuildSettings` and turned into `OsString`s by `jaic-cli`.

## How to change it

- New syntax: `lexer.rs`, `parser/`, `ast.rs` (tests in `parser/tests.rs`), then sema, then a program under `tests/compiler/regressions/` or `tests/corpus/`.
- Unsupported constructs must produce a diagnostic, never a silent success.
- Closed sets of names (directives, procedure flags, builtin keys) are enums declared with `raw_enum!` (`macros.rs`). Match on the enum, not strings.
- A new lookup of something on the machine goes in `jaic-toolchain`, reads through `Host`, and has a `FakeHost` test. If `jaic` itself needs it, it goes in `jaic`, which cannot depend on `jaic-toolchain`.
- A new passthrough option is an `Action::Passthrough` and stays an `OsString` until it reaches `Command`.
- When a field only means something given another, make the two one enum rather than a `bool` beside an `Option`.
- A new browser-lacks-X difference: add a constant or small wrapper to `platform.rs` and branch on it instead of adding a `cfg`. A native-only subsystem is a module with a wasm stub. A new CPU for fibers is its `.s` file and one arm in `fiber.rs`.

## Configuration

```sh
cargo run -p jaic-cli -- file.jai -check [-I dir] [-os linux|windows|macos]
cargo run -p jaic-cli -- file.jai -interp
```

- `dynamic-llvm` (default), `static-llvm` and `lazy-llvm` (release archives: `jaic` opens `libjaic_backend` beside it on the first native build, see [lazy backend](lazy-backend.md)) need LLVM 23 ([LLVM setup](../tools/llvm-setup.md)).
- `mimalloc` is on with either LLVM feature ([allocator](compile-speed.md#allocator)).
- `--no-default-features` builds a check-and-interpret `jaic` without LLVM.

## Dependencies

`rayon`; Inkwell and LLVM for `jaic-llvm`; `serde` for the language server.
