# Coverage

## What it is

`jaic -coverage` builds a native program that records which basic blocks ran. `tools/coverage/report.py` maps them back to source lines and functions and prints per-module and per-file coverage for `stdlib/`, optionally as an lcov file for editor gutters.

```
$ tools/differential/jaic-sweep.py --coverage target/cov modules
$ tools/coverage/report.py report target/cov --lcov target/cov.info --least 10
```

## How it works

Compile (`Sanitize::coverage` in `crates/jaic-llvm/src/lib.rs`): after the optimization pipeline the `sancov-module` pass numbers every basic block (SanitizerCoverage `trace-pc-guard` plus `pc-table`, level 2). Its options come from LLVM's command line (`set_coverage_options`), not the pass name. `-coverage` travels as the `coverage` entry of the sanitizer list, so it shares the single-codegen-unit rule and the `.dSYM` that sanitized builds get.

Link: the Clang driver also compiles `crates/jaic-llvm/runtime/coverage.c`, which assigns guard ids, sets a byte per block when its guard callback fires, and at exit writes `<dir>/<exe>.<pid>.cov`: a header with the executable path, then one `address entry hit` line per block, with addresses un-slid (link-time) so ASLR does not matter. The directory is `JAIC_COVERAGE_DIR` (default: the working directory at start-up); `JAIC_COVERAGE_FILE` names the file outright.

Ingest (`report.py ingest`): blocks are sorted by address; each block's range is the lines of every DWARF line-table row (`llvm-dwarfdump --debug-line`) inside it. A line is hit if any block containing it ran. Function entries (the pc-table's entry flag) are named by `llvm-symbolizer`; compiler-generated functions without a source line are dropped. Output is a JSON of line and function hits per source file. `jaic-sweep.py --coverage DIR` does this after each native run, before the executable and its debug info are deleted.

Report (`report.py report DIR`): merges every JSON in `DIR` (a line or function is hit if any run hit it) and prints a table sorted from least to most covered. A module is a top-level `stdlib/` entry (`internal/<x>` and `Extensions/<x>` count as their own).

## First measurement

`tools/differential/jaic-sweep.py --coverage DIR --headless modules` on macOS arm64: 439 module test programs ran; 32 fail to build for environment reasons (missing libclang, fonts or audio libraries, as without `-coverage`) and 53 are Windows/Android-only. The sixteen least covered modules, then the total:

```
module                               lines         functions
prelude                        0/95       0.0%      0/42      0.0%
SDL                            0/2        0.0%      1/2      50.0%
internal/bindings              0/95       0.0%      0/11      0.0%
internal/window                0/15       0.0%      0/17      0.0%
Objective_C                    0/72       0.0%      0/40      0.0%
internal/clipboard             0/99       0.0%      0/12      0.0%
internal/basic-interface       0/4        0.0%      0/4       0.0%
GL                             4/112      3.6%      4/77      5.2%
internal/program-print        63/987      6.4%      8/26     30.8%
internal/compiler-api         42/625      6.7%     10/139     7.2%
Android                       16/127     12.6%     55/63     87.3%
internal/rendering           334/1296    25.8%     52/101    51.5%
internal/code-visit           51/156     32.7%      6/6     100.0%
internal/system               33/89      37.1%     13/14     92.9%
internal/process             197/518     38.0%     21/33     63.6%
Program_Print                  8/21      38.1%      6/9      66.7%
total                        19162/35135   54.5%   8484/10557  80.4%
```

The lowest entries are platform code a macOS run cannot reach (`Objective_C`, `internal/clipboard`, `internal/window`, `Android`) or modules whose tests do not build here. `internal/compiler-api`, `internal/program-print`, `internal/rendering` and `GL` are the stdlib code that needs tests.

## How to change it

- Granularity: `-sanitizer-coverage-level=` in `set_coverage_options` (1 function, 2 block, 3 edge). The runtime and tool assume one guard per pc-table row.
- Another instrumentation on top: the pass is one more entry in `Sanitize::passes`; fuzzer instrumentation uses the same pass.
- Use `-O0`: optimized code merges and moves blocks, so lines and functions go missing.
- Only code compiled into an instrumented program is counted. Functions no program references do not appear, so the percentages are an upper bound; generic procedures count once per instantiation.
- Limits: 64 Mi blocks and 64 modules per process (`coverage.c`). Windows, wasm and cross builds are rejected like other sanitizers (`check_sanitizer_target`).

## Configuration

- `jaic -coverage` (also valid next to `-sanitize`), `JAIC_COVERAGE_DIR`, `JAIC_COVERAGE_FILE`.
- `jaic-sweep.py --coverage DIR`; `report.py report --root`, `--files`, `--least N`, `--lcov FILE`, `--json`. `LLVM_SYS_231_PREFIX` selects the LLVM whose `llvm-dwarfdump` and `llvm-symbolizer` are used (default: Homebrew LLVM, then `PATH`).

## Dependencies

LLVM's `sancov-module` pass, a Clang driver of the same LLVM to link, `llvm-dwarfdump` and `llvm-symbolizer`, Python 3. Debug info must be on (the default).
