# jaic regression sweep

## What it is

`tools/jaic-sweep.py` runs the `jaic` binary over sets of Jai programs and reports pass/fail. It is the main regression gate for compiler and stdlib changes.

## How it works

Each set yields `(id, path, mode, expectation, extra args)`; the tool runs `jaic <mode> <path> <args>` from the file's directory and compares the exit code (and stdout, for `corpus` cases with a recorded runtime expectation).

| set | source | mode |
|---|---|---|
| `corpus` | `tests/corpus/manifest.json` cases with a `runtime` record | run, exact stdout + exit code; a record with `error` instead expects a runtime error: any failing status and that text on stderr |
| `negative` | `kind: negative` cases of `tests/corpus/manifest.json` (`tests/corpus/negative/*.jai`) | check, non-zero exit and the recorded text in stderr |
| `stdlib` | `tests/stdlib/*.jai` | run, exit code 0 |
| `modules` | the stdlib's own tests: `stdlib/<Module>/tests/*.jai`, `stdlib/tests/**/*.jai` (not under a `modules/` folder, which holds a test directory's mock modules) | run, exit code 0 |
| `examples` | `tests/examples.json` cases (`examples/tour`) | run with the case's `args`, exit code 0, every `stdout_contains` line present and no `stdout_excludes` text |
| `upstream` | `tools/upstream-cases.json` (paths under `corpus/upstream/`; every The_Way_to_Jai example that works, plus project entry points) | per case (`run` or `check`, optional `args`), exit code 0, and the case's `expect` record if it has one |
| a path | that file | run |

Expected result of `corpus negative stdlib modules upstream examples`: everything passes. Programs that must fail to compile (including `getrect-rh-negative-control`, which proves the GetRect geometry assertions fire) live in the `negative` set, where a case passes when the compiler reports its expected error.

Each case runs with an exact allocation cap (`--memory-limit`, default 3 GiB), which the sweep passes to `jaic` as `JAIC_MEMORY_LIMIT` (see [memory limit](../compiler/memory-limit.md)). `jaic` itself stops at the first allocation past it with exit status 120, and the case fails with `error: memory limit of 3072 MiB exceeded` (the sweep goes by the status, which only `jaic` gives that meaning: a built executable's 120 is its own); memory outside `jaic`'s allocator (LLVM, child processes) is not capped. The `--timeout` still kills a case that runs too long. The default `--jobs` is the CPU count capped so that jobs × limit fits in physical memory. The sweep refuses to start when the `jaic` binary is older than any source in `crates/jaic`, `crates/jaic-cli` or `crates/jaic-llvm` (`--allow-stale` overrides): a stale build can lack the limits that keep negative cases such as unbounded polymorphic recursion from exhausting memory.

`--native` builds each `run` case with `jaic build` into a scratch directory and runs the executable instead of `jaic run`, with the same expectation; `--opt O0..O3` sets the build's optimization level and `--sanitize address,undefined` instruments it (both imply `--native`). A sanitized case also fails on any sanitizer report in stderr. A program whose metaprogram writes no executable is listed separately as compile-time only. See [sanitizers](../native/sanitizers.md). The sweep exits with status 1 when any case fails.

`--headless` is for machines without a display or GPU, such as CI runners. A `run` case whose source imports `Window_Creation` is only checked (`jaic check`), or built when `--native` is on, and its expected output is ignored.

## How to change it

- New negative program (must be rejected): drop it in `tests/corpus/negative/`, add a `kind: negative` entry with `sha256` and a `negative.check` string that appears in the diagnostic to `tests/corpus/manifest.json`.
- New regression program: drop a self-checking `tests/stdlib/<name>.jai` (return non-zero / `assert` on failure).
- New program that must stop with a runtime error on every backend: drop it in `tests/corpus/positive/` and give its manifest entry `"runtime": {"error": "<message>", "stdout": "<output before the error>"}`. The sweep (`--native` too), `crates/jaic-cli/tests/native.rs` and `tools/windows_cross.py` accept any failing status with the message on stderr; [differential testing](differential-testing.md) requires every backend to stop with a runtime error. Example: `null-deref-print-any`.
- An upstream case may list `setup` commands (argv lists, run once in the case's directory before the cases
  start), e.g. compiling the C++ library `ttwj-30-cpp-library-main` loads.
- A case whose program writes into its own tree (generated bindings, `module_api.public.jai`) lists the corpus
  directories to `copy` into a scratch directory, and the read-only siblings it reaches through relative paths to
  `link` there. Path, setup commands and the run then use the copy, which is deleted after the sweep. Example:
  `vk-engine-*-check` copies `ostef--Vk-Engine` and `ostef--Jolt-Jai`, and links `ostef--Linalg` and `ostef--JoltC`.
  Never `link` a directory that a setup command or the program writes to: the link leads back into the corpus.
- An upstream entry point started working: add it to `tools/upstream-cases.json` so it stays working.
- An upstream case may carry an `expect` record taken from the project's own documentation or tests
  (never from jaic's output): `exit_code`, exact `stdout`, `stdout_contains` (each line present) or
  `stdout_ordered` (each line present, in this order), plus a `source` naming where the values come from.
  See [upstream corpus, recorded outputs](upstream-corpus.md#recorded-outputs).
- A case may name `run_after` (argv relative to the case's directory): after a successful build, the sweep runs
  that program, and the `expect` record applies to the program's output. With `copy`, `files` maps destinations in
  the scratch copy to files of this repository, such as `tools/upstream-drivers/reflector-tests.jai`.
- `build` cases (`focus-native-build`, `jails-native-build`, `jaison-native-build`) produce native executables and
  need LLVM plus the third-party libraries, which the sweep builds on first use and shares across worktrees (see [native libs](native-libs.md)).

## Configuration

`--jaic PATH` (default `$CARGO_TARGET_DIR/debug/jaic`, else `target/debug/jaic`), `--filter TEXT`, `--verbose`, `--timeout SECONDS` (default 60; use 900 for the full sweep, which builds Focus and Jails), `--jobs N` (cases run at once, default the CPU count capped so jobs × limit fits in RAM; cases run with stdin closed), `--memory-limit GIB` (default 3).

```sh
python3 tools/jaic-sweep.py --jaic /path/to/target/debug/jaic corpus stdlib --timeout 900
```

The `upstream` set needs `python3 tools/fetch_upstreams.py` first (the corpus is gitignored).

## Dependencies

Python 3 standard library, a built `jaic-cli`.
