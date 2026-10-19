# Determinism checks

## What it is

`tools/differential/determinism.py` compiles the local test programs under different thread, parse and codegen-unit settings, several times each, and fails when anything observable differs. It catches output that follows hash-map iteration order, thread scheduling or the split into code modules.

```sh
python3 tools/differential/determinism.py --jaic target/release/jaic positive negative regressions semantics
python3 tools/differential/determinism.py --jaic target/release/jaic --filter case- --jobs 1 positive   # a few files
```

A full run takes about 20 minutes on an M-series laptop (`--jobs 3`); CI runs it nightly (`.github/workflows/determinism.yml`) and on demand.

## How it works

For every file, in its own directory:

| Check | Configurations | Must be equal |
| --- | --- | --- |
| `check` | `jaic -check` with `-parse_threads 1`, `-serial_parse`, `-parse_threads 8`, `-threads 1` and the defaults, each `--repeat` (2) times | exit code, stdout, stderr: diagnostic text and order |
| `build` | the first three parse settings x `-codegen_units 1` and `3` x default (debug) and `-O2 -no_debug_info`, each repeated | exit code, output, the LLVM IR from `-emit_ir`, and the machine code (below) |
| `run` | the built program under each unit count and level | what the program prints and its exit code (the split changes the objects, never behaviour) |
| `lsp` | two `jailsp` sessions per file | pushed diagnostics, document symbols, semantic tokens, inlay hints and hovers on the first column of each line |

Rust seeds hash maps per process, so two runs of the same binary already differ in iteration order; a repeat that matches is evidence the output does not depend on it.

**Machine code.** The linked executable is compared byte for byte, with one exception: an unoptimized macOS link stores the modification time of each object in its debug map, so the check hashes the kept objects (`.build/*.o`) instead. The optimized variant is built with `-no_debug_info` for the same reason (its objects are deleted, but their times would still be in the debug map). Every build writes to the same output name, because the output path is embedded in the executable.

**Host addresses.** `ctdata-opaque-pointer.jai` and `rules-compiler-module.jai` put an address from the compile-time interpreter into the program by design, so the check compares only their diagnostics (`HOST_ADDRESS_CASES` in `tools/differential/determinism.py`).

## How to change it

- New configuration: add to `PARSE_CONFIGS`, `UNIT_COUNTS` or `LEVELS`. The first entry of each is the reference the others are compared with.
- New set: add it to `SETS`; sets in `BUILDABLE` are also built.
- More language-server facts: add requests to `lsp_session`. It is a small client of its own so the script needs no third-party packages.
- A difference prints the field that differs (`field 4` of a build is the machine code hash). Reproduce with the printed flags; for hash-order bugs repeat the build several times, the order changes per process.

## Configuration

`--jaic`, `--jailsp` (default: beside `jaic`), `--repeat`, `--filter`, `--jobs`, `--no-lsp`, `--no-build`. `JAIC_STDLIB` is set to the checkout's `stdlib/`.

## Dependencies

A release build of `jaic-cli` and `jai-language-server`, LLVM for the native builds. Only the repository's own test programs are used (nothing from the upstream corpus).
