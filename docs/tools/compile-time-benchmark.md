# Compile-time benchmark

## What it is

`tools/compile_bench.py` measures how long jaic takes to compile real projects from the
[upstream corpus](upstream-corpus.md), in three modes: `check`, `build -O0` and `build -O2`. For each workload
it records wall time, peak RSS and the time per compiler phase. It writes JSON and Markdown and can compare
against an earlier run. `tools/bench.py` ([benchmarks](benchmarks.md)) covers interpreter micro-benchmarks
instead.

## How it works

- **Workloads** (`WORKLOADS` in the script): focus, Jails, sgpu examples (check only), jaison's tests, open-jai
  `33.3_getrect_buttons`, chess-jai (`build.jai - ui ai debug|release`) and forbear (check and `-O0`).
  Projects driven by a metaprogram pick their own optimization level, so their `build-O2` row passes the
  project's release switch (Jails' `-release` is O3). Vk-Engine is left out: its `Build.jai` rewrites files in
  the corpus.
- **Generated workloads**: `gen-10k`, `gen-60k` and `gen-240k` (`check` and `build-O0`) write a program from
  [`tools/benchgen.py`](benchmark-generator.md) (seed 1) into a temporary directory first. They measure scaling
  on corpus-shaped code and need no corpus checkout. Changing the generator changes these programs, so
  results from before and after are not comparable.
- **Setup**: a workload can name `tools/upstream-cases.json` ids whose `setup` commands run first (forbear's
  vendored C libraries).
- **Runs**: every workload runs `--repeat` times. The first run is **cold**: binary pages and sources are not
  yet in the OS file cache (jaic has no on-disk cache). The median of the remaining runs is **warm**, and
  `--compare` uses warm numbers. Builds write into a temporary directory or the project's own output path.
- **Peak RSS** comes from `os.wait4` on the jaic process. Children such as the linker are not included.
- **Phases**: the script passes `--timings` (placed before any `-` so the metaprogram does not get it). jaic
  then prints `jaic-timing: <phase> <seconds> <calls>` on stderr. The phases are `total`, `front end`
  (parse, sema, compile-time code, including metaprogram workspaces built inside it), `workspaces`, `run`,
  `prepare output`, `codegen` (LLVM), `link` and `debug info` (dsymutil; only builds that run it, so not `-O0` on macOS, see [debug info](../native/debug-info.md)). Phases nest, so they do not add up
  to `total`. `codegen` is wall time: large `-O0` builds and, after the optimizer, `-O1`+ builds generate
  machine code on several threads ([codegen units](../native/llvm-backend.md#codegen-units)), which also adds a
  copy of each thread's share of the module to peak RSS. Results recorded before that split show `-O2` builds
  slower and smaller, so `--compare` against them flags RSS on those rows.
- **Output**: the JSON holds the date, machine (CPU, cores, memory, OS), jaic commit, dirty flag, rustc and
  LLVM versions, the settings, and per-workload cold, warm, all runs, RSS and phases. The Markdown table has
  one row per workload and mode.

```sh
cargo build --release -p jaic-cli
python3 tools/compile_bench.py --repeat 5 --out new.json --markdown new.md
python3 tools/compile_bench.py --only focus --modes check --compare benchmarks/results/compile-time-apple-m5.json
```

`--compare` prints each workload's change. It exits 1 when warm wall time or peak RSS grew by more than
`--threshold` (10%) and also by more than `--min-seconds` (0.1 s) or `--min-mib` (16 MiB), or when a
workload failed.

The committed baseline is in `benchmarks/results/compile-time-apple-m5.{json,md}`. Compare only against
runs from the same machine; CI runners are a different machine.

### CI

`.github/workflows/compile-bench.yml` is a manual `workflow_dispatch` job on `macos-15`, with inputs
`repeat` and `only`. It builds the release compiler, fetches the corpus, builds the native libraries, runs the
benchmark, puts the Markdown in the job summary, and uploads both files as the `compile-bench` artifact. It
never fails a PR, because shared runners are too noisy. Download two artifacts and use `--compare` to compare
them.

## How to change it

- **New workload**: add an entry to `WORKLOADS`: corpus directory, `{mode: jaic arguments}`, and setup case
  ids. `{tmp}` in arguments becomes a per-run temporary directory (for `-o`). Leave out projects that write
  into `corpus/upstream`; check with `python3 tools/verify_upstreams.py` after a run.
- **New phase**: wrap the code in `timings::time("name", || ...)` in `crates/jaic-cli/src/main.rs`. The
  script reads every reported phase, so nothing else has to change.
- Update the baseline after a deliberate performance change, and record the commit in the JSON (the script
  does this).
- Tests: `python3 -m unittest tools/test_compile_bench.py` (argument insertion, medians, comparison).

## Configuration

- `--jaic` (default `$CARGO_TARGET_DIR/release/jaic`, else `target/release/jaic`), `--repeat`, `--only`,
  `--modes`, `--timeout`, `--out`, `--markdown`, `--compare`, `--threshold`, `--min-seconds`, `--min-mib`,
  `--list`.
- `JAIC_NATIVE_LIBS` when the checkout has no `artifacts/native-libs` (worktrees): chess-jai and focus link
  stb libraries.
- jaic's `--timings` flag works with any command.

## Dependencies

- Python 3.11+, a release jaic, and the corpus from `tools/fetch_upstreams.py`.
- Native libraries from `tools/build_native_libs.py` for the build modes, and clang for forbear's setup.
