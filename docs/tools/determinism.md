# Determinism checks

## What it is

`tools/differential/determinism.py` compiles the local test programs under different thread, parse and codegen-unit settings, several times each, and fails when anything observable differs. It catches output that follows hash-map iteration order, thread scheduling, symbol or syntax-node numbering, or the split into code modules.

Only what a user sees has to agree everywhere: diagnostics and their order, `-check` output, what the program does, language-server answers. Symbols and syntax-node ids are numbered in whatever order threads reach them, so the IR and machine code of a parallel compile may differ between runs; byte-identical artifacts are promised only for single-threaded builds (`-threads 1`).

```sh
PYTHONPATH=tools python3 tools/differential/determinism.py --jaic target/release/jaic positive negative regressions semantics
PYTHONPATH=tools python3 tools/differential/determinism.py --jaic target/release/jaic --filter case- --jobs 1 positive   # a few files
PYTHONPATH=tools python3 tools/differential/determinism.py --jaic target/release/jaic --scramble 1,2 regressions        # number-order bugs
```

A full run with `--scramble 1,2` takes about an hour on an M-series laptop (`--jobs 4`). CI runs it nightly (`.github/workflows/determinism.yml`) and on demand, split into three `--shard`s per OS so each stays well inside the job's 120 minutes. Each finished file prints a progress line with its time.

## How it works

For every file, in its own directory:

| Check | Configurations | Must be equal |
| --- | --- | --- |
| `check` | `jaic -check` with `-parse_threads 1`, `-serial_parse`, `-parse_threads 8`, `-threads 1` and the defaults, each `--repeat` (2) times plus once per `--scramble` seed | exit code, stdout, stderr: diagnostic text and order |
| `build` | `-threads 1`, `-parse_threads 8` and the defaults x `-codegen_units 1` and `3` x default (debug) and `-O2 -no_debug_info`, each repeated; the seeds run only `-threads 1` | exit code, stdout, stderr |
| `single-threaded bytes` | the `-threads 1` builds' plain repeats | the LLVM IR from `-emit_ir` and the machine code (below) |
| `run` | the built program under each unit count, level and seed | what the program prints and its exit code (the split changes the objects, never behaviour) |
| `lsp` | two `jailsp` sessions per file, plus one per seed | pushed diagnostics, document symbols, semantic tokens, inlay hints and hovers on the first column of each line |

Rust seeds hash maps per process, so two runs of the same binary already differ in iteration order; a repeat that matches is evidence the output does not depend on it.

**Scramble seeds.** Hash maps keyed on symbols (`Sym`) or syntax-node ids (`AstId`) hash the number, not the text, and `Sym`'s `Ord` is number order too. Under `JAIC_SYM_SCRAMBLE=<seed>` the interner skips a few seeded numbers before each new spelling, and under `JAIC_ID_SCRAMBLE=<seed>` `AstId::fresh` skips some at the start of every block of 256 (`crates/jaic/src/knobs.rs`). `--scramble` sets both for one extra run per configuration; output that changes with the seed sorts or iterates by number where it must go by spelling. Fix such a site with an ordered container or an explicit sort on the text.

**Machine code.** The linked executable is compared byte for byte, with one exception: an unoptimized macOS link stores the modification time of each object in its debug map, so the check hashes the kept objects (`.build/*.o`) instead. The optimized variant is built with `-no_debug_info` for the same reason (its objects are deleted, but their times would still be in the debug map). Every build writes to the same output name, because the output path is embedded in the executable.

**Host addresses.** A compile-time value placed in the program (a `Code` value or node list a metaprogram built, a pointer a `#run` returned) holds an address of the compiler's own memory, so `@compiler.global_data.*` initializers differ from process to process. The check compares the IR with those initializers blanked (`strip_global_data`); when the raw IR differs between repeats only there, the machine code is not compared for that program either, since it embeds the same addresses. The run output is still compared. `ctdata-opaque-pointer.jai` and `rules-compiler-module.jai` put such an address into the program on purpose, so only their diagnostics are compared (`HOST_ADDRESS_CASES` in `tools/differential/determinism.py`, which also drops the language server's inlay hints for them: they show the pointer).

**Timeouts.** A run that exceeds the time limit is repeated once; only a second timeout counts as the result.

## How to change it

- New configuration: add to `PARSE_CONFIGS`, `UNIT_COUNTS` or `LEVELS`. The first entry of each is the reference the others are compared with.
- New set: add it to `SETS`; sets in `BUILDABLE` are also built.
- More language-server facts: add requests to `lsp_session`. It is a small client of its own so the script needs no third-party packages.
- A difference prints the field that differs (`field 1` of `single-threaded bytes` is the machine code hash). Reproduce with the printed flags; for hash-order bugs repeat the build several times, the order changes per process.

## Configuration

`--jaic`, `--jailsp` (default: beside `jaic`), `--repeat`, `--filter`, `--jobs`, `--shard K/N` (every N-th file from the K-th, as in `stdlib_runtime.py`), `--no-lsp`, `--no-build`, `--scramble`. `JAIC_STDLIB` is set to the checkout's `stdlib/`.

## Dependencies

A release build of `jaic-cli` and `jai-language-server`, LLVM for the native builds. Only the repository's own test programs are used (nothing from the upstream corpus).
