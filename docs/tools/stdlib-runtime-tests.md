# Stdlib tests

The standard library is tested against the public contracts in [docs/specs](../specs/), module tests next to the modules, and a set of regression suites.

## Where the tests live

| Family | Location |
|---|---|
| Public contracts | `tests/stdlib-contract/manifest.json` |
| Compiler semantics and safety | `tests/compiler-regressions` |
| Extensions | `tests/extensions` and `stdlib/Extensions/*/tests` |
| Native ABI and SDK | `tests/native-regressions`, `tests/native` |
| Graphics and events | `tests/graphics-regressions`, `tests/window-render` |
| Module suites | `stdlib/*/tests`; `tests/portable-stdlib`, `foundations`, `platform-io`, `numerics`, `bindings`, `audio`, `developer-tools` |

Export counts and source reachability are not a measure of completed functionality; `tools/stdlib_coverage.py` reports interpreter reachability only, from a source parser and macro-name heuristic.

## Running them

```sh
cargo build --locked -p jaic-cli -p jailint -p jai-language-server
cargo build --locked --release -p jaic-cli
export JAIC_STDLIB="$PWD/stdlib"
target/debug/jaic jaifmt/main.jai -O2 -exe target/jaifmt
python3 tools/build_native_libs.py
python3 tools/stdlib_runtime.py --jaic target/release/jaic --modes interp,native,wasm-interp,wasm-native --coverage target/stdlib-reachability.txt
python3 tests/stdlib-contract/run.py --jaic "$PWD/target/release/jaic" --stdlib-root "$PWD/stdlib" --mode native --opt O2
PYTHONPATH=tools python3 -m unittest tools/test_stdlib_runtime.py tools/test_stdlib_contract_runner.py
```

The module suites with native setup, child processes, fault tests or target matrices have their own drivers. `tools/check_stdlib_suites.py` runs them all against the production library, continues after a failing suite and exits nonzero if any failed:

```sh
python3 tools/check_stdlib_suites.py --jaic target/release/jaic --stdlib stdlib --jailint target/debug/jailint --jaifmt target/jaifmt --native-libs artifacts/native-libs/macos-arm64 --font /path/to/font.ttf
```

## Options and results

- `tools/stdlib_runtime.py --filter TEXT` selects catalogue ids (an empty selection is an error). `--modes` picks the interpreter, native, target-WASM interpreter or WASI execution; `--opt O0|O2` controls generated executables. `--report FILE` writes each case's result and wall time as it finishes.
- Source checks and compile-time-only cases are counted apart from execution. A native build that has no `main` or executable must pass a strict source check before it counts as compile-time-only; that never counts as runtime execution.
- Positive cases compare exact stdout and, with a stdout contract, exact stderr (empty unless given). Check-fail cases need a nonzero status and a diagnostic substring; privacy cases also compile a positive import baseline.
- Native SDK and graphics families are unavailable under WASM and reported separately, never as passed. Missing SDKs, fonts, display servers and failures on an applicable host are failures.
- Tests may read the mode (`stdlib_runtime.py` sets it) to avoid what a mode cannot do.

## Changing tests

Add public cases to the contract manifest with their spec provenance. Keep compiler, ABI and Extensions regressions in their own family, and do not invent a stdlib export for a test helper. A negative fixture needs a nonempty `diagnostic_contains` and, for privacy, a `positive_baseline`. Specialised cases belong in their module driver so native preparation, O0/O2 and target coverage stay required.

Native dependencies are built from source at the repository pins. Linux needs native headers and libraries, Xvfb/Mesa, fonts and the audio backends; macOS uses the official SDKs and the host's graphics and clipboard; Windows needs MSVC/LLVM, correct-architecture libraries and Mesa.

## Dependencies

`tools/stdlib_runtime.py`, `tests/stdlib-contract/run.py`, `tools/stdlib_coverage.py`, the pinned Rust/LLVM and Node toolchains, and the native libraries from `tools/build_native_libs.py`.
