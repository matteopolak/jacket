# Toolchain

## What it is

`jaic::toolchain::Toolchain` describes a target's C environment as data: which runtime libraries a link needs and how each linker spells them, which Jai library names mean "the C runtime", which symbols that runtime exports under another name, how its variables are imported, and how much main-thread stack to reserve. There is one static per kind of target, and `Target::toolchain()` returns it.

The interpreter, code generation, linking and the tests all read the same description. Finding programs and SDK directories on disk is a different job (`toolchain::discover`) and is not part of it.

## How it works

`Toolchain::for_target` picks by OS and ABI:

| Static | File | Targets |
|---|---|---|
| `WINDOWS_MSVC` | `toolchain/windows_msvc.rs` | `*-windows-msvc`: DLL runtime, `long double` math renames, `dllimport` for DLL runtime data |
| `WINDOWS_GNU` | `toolchain/windows_gnu.rs` | `*-windows-gnu`: MinGW links its runtime statically |
| `LINUX` | `toolchain/linux.rs` | every Linux ABI |
| `MACOS` | `toolchain/macos.rs` | `*-apple-darwin` |
| `WASM` | `toolchain/wasm.rs` | WASI and bare modules; foreign functions become imports from `env` or the library's file name |

The fields, and who reads them:

- `linker` (`Unix`, `MinGw`, `Msvc`): what `jaic-llvm` runs. WebAssembly says `Unix` but never reaches the `cc` code; `wasm-ld` links it.
- `runtime.libs`: the runtime libraries in link order. `default` says how a link names one on its own; the others are only listed when a custom link command replaces the defaults (`link_inputs`).
- `runtime_args(driver)`: that one table written for `link.exe`/`lld-link`/`clang-cl` (`/DEFAULTLIB:x`, plus `/MD` for `clang-cl`) or for Clang's `cc`-style driver (`-fms-runtime-lib=dll`, `-Wl,/NODEFAULTLIB:libcmt`, `-Wl,/DEFAULTLIB:x`, `-lx`). Clang's own link step adds the static runtime, hence the exclusion.
- `stack_arg(driver)`: the 8 MiB main stack on Windows, spelled `/STACK:n`, `-Wl,/STACK:n` or `-Wl,--stack,n`.
- `is_runtime(name)`: Jai library names that link nothing by name (`libc`, and on Windows also `m`, `pthread`, `dl`, ...). Used by `library_args`.
- `imports_data(name, system)`: whether a foreign variable is `dllimport`. A Windows system library is a DLL; the C runtime follows `runtime_data`, and on MSVC only the DLL runtime's own names (`ucrt`, `ucrtbase`) promise that, because the generic names also reach the static import library, which defines some data itself.
- `rename(symbol)`: the symbol the runtime really exports. On MSVC `long double` is `double`, and the runtime has no `sqrtl`; the name maps to `sqrt`. Both code generation and the interpreter ask this.
- `import_module(library)`: the wasm import module.
- `runtime.dlls` and `interpreter_names`: the DLLs the interpreter loads on a Windows host and the library names it resolves to the first of them. The interpreter always uses `Toolchain::interpreter()` (the MSVC description), because it loads those DLLs whichever ABI `jaic` itself was built for.
- `implicit_libraries`: libraries every Unix link adds (`m`).

The rendered link arguments are pinned to exact strings in `toolchain/tests.rs`, so a refactor cannot change what the linker is given. `crates/jaic/tests/toolchain_policy.rs` fails if any other file under `crates/*/src` names a runtime library or a renamed math stem.

## How to change it

- A new runtime library or linker spelling: edit the target's file and extend the pinned strings in `toolchain/tests.rs` deliberately.
- A new symbol the runtime renames: change that toolchain's `renames`. Do not add a second check in lowering or in the interpreter.
- A new target kind: add a file with a `Toolchain` static, a match arm in `Toolchain::for_target`, and a row in `tests::every_target_has_a_toolchain`.
- A new place that needs a C-runtime fact: add a field or method here and read it; the policy test will reject a string match elsewhere.

Gotchas:

- `names_fold_case` is true on Windows only; a Linux library called `C` is not the runtime.
- The interpreter's name list (`interpreter_names`) is shorter than `runtime_names`. A system library like `pthread` is dropped from native Windows links but goes through the interpreter's ordinary DLL and import-library lookup. It was this way before the toolchain existed and is left alone.
- `runtime_args` for `MsvcLink` on a MinGW target (`JAIC_LINKER=link` with a `*-windows-gnu` target) is empty; MinGW has no import-library runtime to name.
- `windows.rs` in `interp/native` is Windows-only, so a plain `cargo build` elsewhere does not compile it. `cargo check -p jaic --target x86_64-pc-windows-msvc` does.

## Configuration

None of its own. `JAIC_LINKER` and `JAIC_AR` choose programs ([Windows](../native/windows.md)); `-os`, `-cpu` and `-target` choose the target ([targets](targets.md)).

## Dependencies

`jaic::target` (`Target`, `TargetOs`, `TargetAbi`). Used by `jaic-llvm` (`link.rs`, `linker.rs`, `link_options.rs`, `lower/symbols.rs`) and the interpreter (`interp/native/windows.rs`).

## Discovery

`toolchain/discover.rs` finds a Microsoft `link.exe`, `lib.exe`, the `LIB` value and a Clang outside `PATH` on a Windows host, through the `find-msvc-tools` crate, and tells Microsoft's `link` apart from the coreutils one. It returns paths and environment; it never says which libraries to link. Choosing among the programs it finds stays in `crates/jaic-llvm/src/linker.rs` (`linker_command`).
