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
- **Where to get declaration order.** A scope's `names` map iterates in symbol-number order. Anything a user can see (which error comes first, the order of suggestions, the type table) takes its order from `Scope::declared()`, which lists `(entity, name)` in declaration order. A pass whose result does not depend on the order (an `any`, a sum, a set that is sorted afterwards) can keep iterating the map.
- **Fast guard.** `cargo test -p jaic-cli --test symbol_order` compiles a few programs from `tests/fixtures/symbol-order/` (and the type-table and metaprogramming cases) under three seeds with `-serial_parse` and requires identical output (`test_support::determinism::assert_stable`); it takes about a second. Add a program there when a new visible symptom is fixed.
- **Machine code.** Compared byte for byte, except that unoptimized macOS links hash the kept objects (the debug map stores object times).
- **Host addresses.** Compile-time values placed in the program hold compiler memory addresses, so the check normalizes byte-string payloads of `@compiler.global_data.g<id>` globals (`strip_global_data`). LLVM renders bytes as printable characters, hex escapes or escaped backslashes; the matcher accepts all three. It still compares symbol identity, storage layout, alignment and all other globals. `HOST_ADDRESS_CASES` lists programs that do this on purpose and compare only diagnostics.
- A timeout is repeated once; only a second timeout counts.

## How to change it

- New configuration: add to `PARSE_CONFIGS`, `UNIT_COUNTS` or `LEVELS` (the first entry is the reference).
- New set: add to `SETS` (`BUILDABLE` sets are also built).
- More language-server facts: add requests to `lsp_session`.
- Reproduce a difference with the printed flags; hash-order bugs need several repeats.

## Configuration

`--jaic`, `--jailsp`, `--repeat` (2), `--filter`, `--jobs`, `--shard K/N`, `--no-lsp`, `--no-build`, `--scramble`.

`tools/differential/test_determinism.py` pins the emitter's global-name format and
the normalization boundary. Run it with
`PYTHONPATH=tools python3 -m unittest differential.test_determinism`. When changing
LLVM symbol naming, update this matcher and the artifact tests together; do not
add affected programs to `HOST_ADDRESS_CASES` just to bypass comparison.
