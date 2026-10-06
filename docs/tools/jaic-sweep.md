# jaic regression sweep

## What it is

`tools/jaic-sweep.py` runs the `jaic` binary over sets of Jai programs and reports pass/fail. It is the main regression gate for compiler and stdlib changes.

## How it works

Each set yields `(id, path, mode, expectation, extra args)`; the tool runs `jaic <mode> <path> <args>` from the file's directory and compares the exit code (and stdout, for `corpus` cases with a recorded runtime expectation).

| set | source | mode |
|---|---|---|
| `corpus` | `tests/corpus/manifest.json` cases with a `runtime` record | run, exact stdout + exit code |
| `negative` | `kind: negative` cases of `tests/corpus/manifest.json` (`tests/corpus/negative/*.jai`) | check, non-zero exit and the recorded text in stderr |
| `stdlib` | `tests/stdlib/*.jai` | run, exit code 0 |
| `modules` | the stdlib's own tests: `stdlib/<Module>/tests/*.jai`, `stdlib/tests/**/*.jai` (not under a `modules/` folder, which holds a test directory's mock modules) | run, exit code 0 |
| `examples` | `tests/examples.json` cases (`examples/tour`) | run with the case's `args`, exit code 0, every `stdout_contains` line present and no `stdout_excludes` text |
| `upstream` | `tools/upstream-cases.json` (paths under `corpus/upstream/`; every The_Way_to_Jai example that works, plus project entry points) | per case (`run` or `check`, optional `args`), exit code 0 |
| a path | that file | run |

Expected result of `corpus negative stdlib modules upstream examples`: everything passes. Programs that must fail to compile (including `getrect-rh-negative-control`, which proves the GetRect geometry assertions fire) live in the `negative` set, where a case passes when the compiler reports its expected error.

Each case runs with an exact allocation cap (`--memory-limit`, default 3 GiB), which the sweep passes to `jaic` as `JAIC_MEMORY_LIMIT` (see [memory limit](../compiler/memory-limit.md)). `jaic` itself stops at the first allocation past it with exit status 120, and the case fails with `error: memory limit of 3072 MiB exceeded`; memory outside `jaic`'s allocator (LLVM, child processes) is not capped. The `--timeout` still kills a case that runs too long. The default `--jobs` is the CPU count capped so that jobs × limit fits in physical memory. The sweep refuses to start when the `jaic` binary is older than any source in `crates/jaic`, `crates/jaic-cli` or `crates/jaic-llvm` (`--allow-stale` overrides): a stale build can lack the limits that keep negative cases such as unbounded polymorphic recursion from exhausting memory.

## How to change it

- New negative program (must be rejected): drop it in `tests/corpus/negative/`, add a `kind: negative` entry with `sha256` and a `negative.check` string that appears in the diagnostic to `tests/corpus/manifest.json`.
- New regression program: drop a self-checking `tests/stdlib/<name>.jai` (return non-zero / `assert` on failure).
- An upstream case may list `setup` commands (argv lists, run once in the case's directory before the cases
  start), e.g. compiling the C++ library `ttwj-30-cpp-library-main` loads.
- A case whose program writes into its own tree (generated bindings, `module_api.public.jai`) lists the corpus
  directories to `copy` into a scratch directory, and the read-only siblings it reaches through relative paths to
  `link` there. Path, setup commands and the run then use the copy, which is deleted after the sweep. Example:
  `vk-engine-*-check` copies `ostef--Vk-Engine` and `ostef--Jolt-Jai`, and links `ostef--Linalg` and `ostef--JoltC`.
  Never `link` a directory that a setup command or the program writes to: the link leads back into the corpus.
- An upstream entry point started working: add it to `tools/upstream-cases.json` so it stays working.
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
