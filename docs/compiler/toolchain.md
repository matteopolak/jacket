# Toolchain

`jaic::toolchain::Toolchain` describes a target's C environment as data: which runtime libraries a link needs and how each linker spells them, which Jai library names mean "the C runtime", which symbols the runtime exports under another name, how its variables are imported, and how much main-thread stack to reserve. `Target::toolchain()` returns the static for the target. The interpreter, code generation, linking and tests all read the same description.

## How it works

`Toolchain::for_target` picks by OS and ABI:

- `WINDOWS_MSVC` (`toolchain/windows_msvc.rs`): DLL runtime, `long double` math renames, `dllimport` for DLL runtime data.
- `WINDOWS_GNU`: MinGW links its runtime statically.
- `LINUX`, `MACOS`: the usual libc and libm.
- `WASM`: foreign functions become imports from `env` or the library's file name; `wasm-ld` links.

Fields worth knowing:

- `runtime.libs` and `runtime_args(driver)`: the runtime libraries in link order, spelled for `link.exe`/`lld-link`/`clang-cl` or for Clang's `cc` driver.
- `stack_arg(driver)`: the 8 MiB main stack on Windows.
- `is_runtime(name)`: names like `libc`, `m`, `pthread`, `dl` link nothing by name.
- `imports_data(name, system)`: whether a foreign variable is `dllimport`.
- `rename(symbol)`: the symbol the runtime really exports. On MSVC `long double` is `double`, so `sqrtl` maps to `sqrt`. Code generation and the interpreter both ask.
- `runtime_names`: the library names that mean the C runtime, with the DLL the interpreter loads on Windows. `ucrt`, `ucrtbase` and `vcruntime` load `ucrtbase.dll`; every other name loads `msvcrt.dll`. They are never merged: different heaps, `FILE*` and `errno`.

`toolchain/discover.rs` is a separate job: finding `link.exe`, `lib.exe`, `LIB` and Clang on a Windows host. It returns paths, never which libraries to link; choosing among programs stays in `jaic-llvm/src/linker.rs`.

## How to change it

- New runtime library or linker spelling: edit the target's file and update the pinned strings in `toolchain/tests.rs` on purpose.
- New renamed symbol: add it to that toolchain's `renames`, not to lowering or the interpreter.
- New target kind: a file with a `Toolchain` static, an arm in `for_target`, and a row in `tests::every_target_has_a_toolchain`.
- Any new C-runtime fact belongs here. `crates/jaic/tests/toolchain_policy.rs` fails if another file under `crates/*/src` names a runtime library.

Gotchas:

- Name matching folds case on Windows only.
- Windows system libraries like `kernel32` go through the interpreter's normal DLL lookup.
- `interp/native/windows.rs` only compiles on Windows; check with `cargo check -p jaic --target x86_64-pc-windows-msvc`.

`JAIC_LINKER` and `JAIC_AR` choose programs ([Windows](../native/windows.md)); `-os`, `-cpu` and `-target` choose the target ([targets](targets.md)).
