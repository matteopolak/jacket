# Crate layout

What each crate under `crates/` owns and where the boundaries between them are. [Compiler architecture](../compiler/architecture.md) describes how a build flows through the crates; this page is about where code lives.

## The crates

From the bottom up. The last column lists workspace crates, then the external crates that shape the boundary.

| Crate | Owns | Depends on |
| --- | --- | --- |
| `jacket-config` | `jacket.toml`, the environment-variable registry (`env.rs`), LLVM install discovery (`llvm.rs`), `locate` for sibling tools, OS-string splitting (`os_text.rs`) | `serde`, `serde_ignored`, `toml` |
| `jaic-syntax` | lexer, parser, AST, diagnostics and their renderer | `jacket-config` |
| `jaic` | sema, IR, interpreter, build API, IDE services, the target model (`target.rs`, `toolchain.rs`) and Visual Studio / Windows SDK discovery (`toolchain/discover.rs`) | `jaic-syntax`, `jacket-config`; `find-msvc-tools` |
| `jaic-toolchain` | finding linkers, archivers, sysroots, builtins, Xcode and native library folders; install notes; the toolchain cache | `jaic`, `jacket-config` |
| `jaic-native` | the link command, linker-argument translation, the direct macOS `ld` link, wasm and freestanding links | `jaic`, `jaic-toolchain`, `jacket-config` |
| `jaic-llvm` | LLVM code generation | `jaic`, `jaic-native`, `jacket-config`; `inkwell` |
| `jaic-backend` | `jaic-llvm` as the shared library a release `jaic` opens on its first native build ([lazy backend](../compiler/lazy-backend.md)) | `jaic-llvm`, `jaic-native` |
| `jaic-cli-args` | `jaic`'s option table and `Cli` | `jaic` |
| `jaic-cli` | the `jaic` binary | `jaic`, `jaic-cli-args`, `jaic-toolchain`, `jacket-config`; `jaic-native`, `jaic-llvm` behind `llvm` |
| `jailint` | lint rules and the `jailint` binary | `jaic`, `jacket-config` |
| `jai-language-server` | `jailsp` | `jaic`, `jailint`, `jacket-config` |
| `jai-wasm` | the browser playground and language server | `jai-language-server`, `jaic`; `jacket-config` at build time |
| `jacket` | the `jacket` binary | `jaic`, `jailint`, `jaic-cli-args`, `jaic-toolchain`, `jacket-config` |
| `test-support` | shared test helpers | `jaic`, `jaic-toolchain`, `jacket-config`; optional `jaic-llvm`, `jai-language-server` |

## The target model sits inside `jaic`

`jaic-toolchain` needs `Target`, `Linker` and `Driver`, which live in `jaic`, so it depends on `jaic` and `jaic` cannot use it. The interpreter opens Windows import libraries from the SDK's `LIB` folders, so `jaic` keeps the Visual Studio and SDK lookup (`toolchain/discover.rs`, on `find-msvc-tools`) itself. Everything else about finding tools is in `jaic-toolchain`.

The interpreter gets its library folders from the driver: `jaic-cli` calls `jaic::interp::set_library_dirs` with what `jaic-toolchain` found, and `set_missing_libraries` with the reason for any library it looked for and did not find (libclang on a Mac), which a failed open then reports.

## Path finding

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

## Arguments that are not text

Process arguments, paths and environment values are OS strings, and Clang and the linkers take any bytes. `jaic` reads its command line with `args_os`. `-link_arg`, `-linker`, `-sysroot` and `-compiler_rt` are `Action::Passthrough` options, stored as given, `-link_arg=VALUE` included. `jaic-native` translates and renders `OsString`s (`ExtraArgs` in `link_options.rs`, `freestanding.rs`, `wasm.rs`, `apple_ld.rs`) and rewrites only the ASCII spellings it knows. `jacket_config::os_text` splits an OS string at ASCII characters without decoding it. Any other command-line argument that is not UTF-8 is an error.

The metaprogram's `additional_linker_arguments` are Jai strings, held as `String` in `jaic::build::BuildSettings` and turned into `OsString`s by `jaic-cli`.

## How to change it

- A new lookup of something on the machine goes in `jaic-toolchain`, reads through `Host`, and has a `FakeHost` test. If `jaic` itself needs it, it goes in `jaic`, which cannot depend on `jaic-toolchain`.
- A new passthrough option is an `Action::Passthrough` and stays an `OsString` until it reaches `Command`.
- When a field only means something given another, make the two one enum rather than a `bool` beside an `Option`.
