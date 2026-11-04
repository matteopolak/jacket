# Benchmarks and profiling

`tools/bench/` is the maintainer harness. It times the compiler (and, with `compare`, two compiler builds head to head), the language server, the linter and the formatter. Each subcommand writes JSON and a Markdown summary to `target/bench/` (gitignored), takes the median of N runs, and can compare against an earlier run. To benchmark Jai code, use [`jacket bench`](jacket-bench.md).

```sh
PYTHONPATH=tools python3 tools/bench/main.py compile --compiler jaic=target/release/jaic
PYTHONPATH=tools python3 tools/bench/main.py lsp
PYTHONPATH=tools python3 tools/bench/main.py lint
PYTHONPATH=tools python3 tools/bench/main.py fmt
PYTHONPATH=tools python3 tools/bench/main.py compile --compare target/bench/old.json   # exit 1 on regressions
PYTHONPATH=tools python3 tools/bench/main.py compare --baseline OLD/jaic --candidate NEW/jaic
```

Results are never committed. Keep a baseline per machine with `--out` and pass it to `--compare`.

## How it works

Every result has three numbers, each as median, min, max, MAD and p95: `wall` seconds, `cpu` seconds (process plus waited-for children), and `rss_mib` (peak of the largest single process). The harness warms the file cache (`--warmup`, default 1) and then takes `--repeat` timed runs (default 5).

`bench.measure.run_measured` owns the command's process tree: a private process group on POSIX, a Job Object on Windows. Timeout, interruption and normal exit clean up descendants. Windows setup failures also close the new Job and terminate a suspended process if assignment failed. For long diagnostic runs, `run_measured(command, output=log_file)` streams stdout and stderr to an open binary file; the result keeps status and resource metrics with empty output strings. The caller owns the file. PGO training uses this path so a timed-out sweep cannot leave compiler jobs running.

`--compare old.json` flags a result when wall, CPU or RSS grew by more than `--threshold`, more than `--min-seconds`/`--min-mib`, and more than three MADs. A workload that worked before and fails now is a regression too. Compare runs from one machine only.

Common options: `--only A,B` (substrings of result keys), `--list`, `--timeout`, `--out`, `--markdown`.

- **`compile`**: result keys are `<workload>/<mode>/<compiler>`; modes are `check`, `build`, `optimized`, `interp`. Workloads (`tools/bench/workloads.py`) cover startup (`hello`), generated programs (`gen-2k` to `gen-60k` from `tools/benchgen.py`), real projects from the [upstream corpus](upstream-corpus.md), compile-time-heavy programs in `benchmarks/compile/`, and generated-code speed in `benchmarks/runtime/`. A stage table (front end, compile-time execution, codegen, link) comes from `jaic -timings`; `-profile` adds the interpreted instruction count, which is nearly deterministic.
- **`lsp`**: starts a fresh server over stdio per session and times a fixed script of requests (`lsp_session.py` lists the probes). A null or empty answer is counted as `empty_answers`. `--server NAME=COMMAND` times another server.
- **`lint` and `fmt`**: `jailint` and `jaifmt` on generated and repository inputs. `lint --only repo` runs the whole repository (about five minutes of ten cores).
- **`probe`**: runs files under `jaic` and a reference compiler (`--reference` or `$JAI_REFERENCE`) in a sandbox and diffs the outcome. Output is for reading; do not commit it.
- **`compare`**: the fair A/B of two binaries. Generated programs of several sizes plus the corpus projects are built in check, build and optimized modes. Both binaries must succeed with the same output before anything is timed. Warm rounds alternate which binary goes first; cold rounds use fresh output folders, an empty `JAIC_CACHE_DIR` and (on macOS, if `sudo -n /usr/sbin/purge` is allowed) a purged file cache. The ratio is candidate over baseline per round, with a bootstrap 95% interval, and the verdict is only given when the interval excludes 1. Build modes also record the executable's size, instruction count and hardware counters. Results go to `playground/bench/<timestamp>/` (`summary.md`, `results.json`, `raw.csv`, `inputs.json`, `command.txt`).
- **Quiet machine** (`quiet.py`): refuses to start when the load is above `--max-load` or the CPU is throttled, unless `--force`.
- **Sandbox**: binaries and the programs they build run through `tools/bench/sandbox.py` (no network, limited reads, writes only to scratch). `--no-sandbox` turns it off; `--sandbox-read PATH` adds a readable folder.
- **Stages across binaries**: every binary builds through one metaprogram, `tools/bench/phase_timing.jai`, which times compiler messages, so different compilers give comparable buckets. It cannot split parsing from type checking.

### Windows

`compile`, `compare`, `lsp`, `lint` and `fmt` run on Windows 10 or 11 (x64 or arm64). You need Python 3.11+, the MSVC Build Tools, jaic, and any other compiler you compare. Plug in, use the best-performance power mode, and exclude the benchmark folders from Defender, which is the largest noise source and cannot be detected.

```powershell
$env:PYTHONPATH = "tools"
python tools\bench\main.py compile --compiler "ref=C:\jai\bin\jai.exe" --compiler "jaic=C:\jacket\jaic.exe" --only hello/build --repeat 1
python tools\bench\main.py compare --baseline C:\jai\bin\jai.exe --candidate C:\jacket\jaic.exe --cold-rounds 0
```

- Runs use an AppContainer sandbox (no network, reads limited to system folders and granted paths) and a Job Object for process accounting. A binary linked without relocations cannot start in it; use `--no-sandbox`.
- Windows cannot purge its file cache, so use `--cold-rounds 0`.
- For a stronger wall, `main.py wsb` writes a Windows Sandbox configuration that runs the harness in an offline VM.
- Windows numbers are not comparable with macOS or Linux numbers. Code: `winapi.py`, `winjob.py`, `winsandbox.py`, `cmd_wsb.py`.

## How to change it

- **New compile workload:** add it to `all_workloads()` in `workloads.py` (source, category, a `Plan` per mode). Set `compare_output` when output is deterministic and `runtime` to time the executable.
- **New jaic phase:** wrap the code in `timings::time("name", ...)` in `crates/jaic-cli/src/main.rs`, add it to `JaicTimings` in `stages.py`, and update `timings_report_phases` in `crates/jaic-cli/tests/cli.rs`.
- **New LSP probe:** add a function and a `Probe` in `lsp_session.py`.
- **New lint or fmt scenario:** add a `Scenario` in `cmd_lint.py` or `cmd_fmt.py`.
- **A compiler with another command line:** subclass `Dialect` in `dialects.py`.
- **`compare` internals:** workloads in `cmd_compare.py`, noise thresholds in `quiet.py`, statistics in `pairing.py`.
- **Process ownership:** `measure.py` handles POSIX cleanup and file destinations; `winjob.py` owns Windows job lifetime. Preserve exception cleanup and caller-owned streams when extending either path; regressions are in `test_compile_cmd.py` and `test_windows.py`.
- Gotchas: `jaic -timings` goes before a lone `-`. Seatbelt matches real paths (`/private/var/folders`).

## Profiling

- **Interpreter:** `-profile` counts calls, blocks and instructions per procedure and prints the top 40 to stderr. Counts are comparable across interpreter changes because they count IR.
- **Native:** `cargo build --profile profiling -p jaic-cli`, then `samply record --save-only --unstable-presymbolicate -o prof.json.gz -- target/profiling/jaic first.jai -check`, then `just profile-report prof.json.gz --top 30` (`--callers NAME`, `--within NAME`, `--without NAME`).
- **Comparing two builds:** keep the old binary, alternate both on the same workload at least five times, and compare medians and minimums (`bench compile --compiler before=... --compiler after=...`).
- Hot spots found before: weak hashes turning a `Table` into one chain, linear scans in lists on every call, filling `Code_Node`/`Type_Info` member by member, and per-element loops where one `memset` would do.

## Configuration

`CARGO_TARGET_DIR` locates the default binaries; `--compiler`, `--server`, `--jailint`, `--jaifmt` override. `JACKET_OUTPUT_DIR` moves `target/bench/`. `compare` also takes `--max-load`, `--force`, `--no-purge`, `--program-runs`, `--baseline-env`/`--candidate-env`, `--out-dir`. The `compile-bench.yml` workflow is a manual dispatch on `macos-15` and never fails a PR.

## Dependencies

Python 3.11+ (standard library only), release builds of the tools being measured, the corpus (`tools/fetch_upstreams.py`) and native libraries for the real-project workloads, node for the wasm formatter, samply for native profiles.
