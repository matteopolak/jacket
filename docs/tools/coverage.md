# Coverage

`jaic -coverage` builds a native program that records which basic blocks ran. `tools/coverage/report.py` maps them back to source lines and functions and prints per-module and per-file coverage for `stdlib/`, optionally as an lcov file for editor gutters. Coverage of the Rust crates is separate: [Rust code coverage](rust-coverage.md).

```sh
tools/differential/jaic_sweep.py --coverage target/cov modules
tools/coverage/report.py report target/cov --lcov target/cov.info --least 10
```

## How it works

- **Compile.** After optimization, LLVM's `sancov-module` pass numbers every basic block (`trace-pc-guard` plus `pc-table`, level 2; `set_coverage_options` in `crates/jaic-llvm/src/lib.rs`). `-coverage` travels in the sanitizer list, so it shares the single-codegen-unit rule and `.dSYM` of sanitized builds.
- **Run.** `crates/jaic-native/runtime/coverage.c` counts each block and, at exit, writes `<dir>/<exe>.<pid>.cov` with un-slid addresses. Counts are saturating but not atomic, so threads can lose some.
- **Ingest.** `report.py ingest` maps blocks to lines through the DWARF line table (`llvm-dwarfdump`) and names function entries with `llvm-symbolizer`, producing a JSON per source file. `jaic_sweep.py --coverage DIR` does this after each native run.
- **Report.** `report.py report DIR` merges every JSON (hit if any run hit it) and prints a table from least to most covered. A module is a top-level `stdlib/` entry (`internal/<x>` and `Extensions/<x>` count separately).

### Interpreted programs

`jaic -interp -coverage` records lines without LLVM, on any `-os`/`-cpu` (`crates/jaic/src/interp/line_cover.rs`). It writes lines instead of blocks:

```
jaic-coverage 1
interp
file /abs/path/a.jai
12 3
13 0
```

Only procedures that were entered are listed; `jacket test --coverage` adds the rest from source ([jacket test](jacket-test.md)). Compile-time code in the same process (`#run`, plugins) is counted too.

## Gotchas

- Use `-O0`: optimized code merges and moves blocks.
- Functions no program references don't appear, so percentages are an upper bound. Generic procedures count once per instantiation.
- Limits: 32 Mi blocks and 64 modules per process. Windows, wasm and cross builds are rejected like other sanitizers.
- To change granularity, edit `-sanitizer-coverage-level=` in `set_coverage_options`. The runtime and tool assume one guard per pc-table row.

## Configuration

- `jaic -coverage` (also with `-sanitize` and `-interp`), `JAIC_COVERAGE_DIR`, `JAIC_COVERAGE_FILE`.
- `report.py report --root`, `--files`, `--least N`, `--lcov FILE`, `--json`.
- `LLVM_PREFIX` (or the `llvm-sys` variable) selects the LLVM whose `llvm-dwarfdump` and `llvm-symbolizer` are used ([LLVM setup](llvm-setup.md)).
