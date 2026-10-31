# Determinism checks

`tools/differential/determinism.py` compiles the repo's test programs under different thread, parse and codegen-unit settings, several times each, and fails when anything observable differs. It catches output that follows hash-map order, thread scheduling, symbol or node numbering, or the split into code modules.

Only what a user sees has to agree: diagnostics and their order, `-check` output, program behaviour, language-server answers. IR and machine code of a parallel compile may differ between runs; byte-identical artifacts are promised only for `-threads 1`.

```sh
PYTHONPATH=tools python3 tools/differential/determinism.py --jaic target/release/jaic positive negative regressions semantics
PYTHONPATH=tools python3 tools/differential/determinism.py --jaic target/release/jaic --scramble 1,2 regressions
```

A full run with `--scramble 1,2` takes about an hour. CI runs it nightly (`determinism.yml`), split into `--shard`s.

## How it works

| Check | Compared |
| --- | --- |
| `check` | `jaic -check` under several parse/thread settings and repeats: exit code, stdout, stderr |
| `build` | thread settings x `-codegen_units` 1 and 3 x debug and `-O2 -no_debug_info`: exit code, stdout, stderr |
| `single-threaded bytes` | `-threads 1` repeats: LLVM IR and machine code |
| `run` | the built program's output and exit code under each configuration |
| `lsp` | two `jailsp` sessions: diagnostics, symbols, semantic tokens, inlay hints, hovers |

- **Scramble seeds.** Maps keyed on symbols or node ids hash the number, not the text. `JAIC_SYM_SCRAMBLE=<seed>` and `JAIC_ID_SCRAMBLE=<seed>` shift those numbers (`crates/jaic-syntax/src/knobs.rs`); `--scramble` runs one extra pass per seed. Output that changes with the seed sorts by number where it must go by spelling; fix it with an ordered container or a sort on the text.
- **Machine code.** Compared byte for byte, except that unoptimized macOS links hash the kept objects (the debug map stores object times).
- **Host addresses.** Compile-time values placed in the program hold compiler memory addresses, so the check blanks `@compiler.global_data.*` initializers (`strip_global_data`). `HOST_ADDRESS_CASES` lists programs that do this on purpose and compare only diagnostics.
- A timeout is repeated once; only a second timeout counts.

## How to change it

- New configuration: add to `PARSE_CONFIGS`, `UNIT_COUNTS` or `LEVELS` (the first entry is the reference).
- New set: add to `SETS` (`BUILDABLE` sets are also built).
- More language-server facts: add requests to `lsp_session`.
- Reproduce a difference with the printed flags; hash-order bugs need several repeats.

## Configuration

`--jaic`, `--jailsp`, `--repeat` (2), `--filter`, `--jobs`, `--shard K/N`, `--no-lsp`, `--no-build`, `--scramble`.
