# Benchmarks and profiling

## What it is

Four harnesses, each writing JSON and Markdown and able to compare against an earlier run (compare only on the same machine, one benchmark at a time; shared CI runners are too noisy to gate on):

- `tools/bench.py`: interpreter and compile-time micro-benchmarks (`benchmarks/*.jai`) plus a few corpus projects. Median wall time and interpreted instruction count.
- `tools/compile_bench.py`: whole-project compile time (`check`, `build -O0`, `build -O2`), peak RSS and compiler phases.
- `tools/lsp_bench.py`: language server latency, CPU and memory over stdio.
- `tools/regex_bench.py` with `benchmarks/regex/`: `Extensions/Regex` against other engines (results in `benchmarks/results/regex.md`).

Committed baselines are in `benchmarks/results/` (`compile-time-apple-m5.*`, `lsp-apple-m5.*`).

## bench.py and profiling

```sh
python3 tools/bench.py --out before.json
# change something, rebuild
python3 tools/bench.py --compare before.json
```

Each workload runs `--repeat` times (median wall time) and once more with `JAIC_PROFILE=1` to read the total interpreted instruction count. Counts barely vary between runs, so they show small changes that wall time hides. Workloads: `compile-time`, `interp-calls`, `interp-fib`, `interp-loops`, `interp-loops-long`, `interp-foreign`, `interp-strings`, and corpus rows (`focus-check`, `jaison-tests`, `jails-check`, `sgpu-examples-check`). Vk-Engine is left out because its `Build.jai` rewrites files in the corpus.

- **Interpreter profile.** `JAIC_PROFILE=1` makes every interpreter count calls, blocks and instructions per procedure (self counts, merged in `crates/jaic/src/interp/profile.rs`); the CLI prints the top 40 to stderr plus a breakdown by IR instruction kind and by folded op. Counts stay comparable across interpreter changes because they count IR. Off, it costs one `Option` check per call and block.
- **Native profile.** `cargo build --profile profiling -p jaic-cli`, then `samply record --save-only --unstable-presymbolicate -o prof.json.gz -- target/profiling/jaic first.jai -check` and `python3 tools/profile_report.py prof.json.gz --top 30` (`--callers NAME`, `--within NAME`, `--without NAME`). It reads the busiest thread, which is the 1 GiB compiler worker. For a hot line inside a big inlined function, resolve leaf addresses with `atos -i -o target/profiling/jaic.dSYM -l <load address> <addresses>`.
- **Comparing two builds.** Copy the old binary (`cp target/release/jaic /tmp/jaic-before`), alternate the two on the same workload at least 5 times each, compare medians and minimums (`bench.py --jaic /tmp/jaic-before --out before.json`, then `--compare`). Record peak memory with `/usr/bin/time -l` (macOS) or `-v` (Linux). For interpreter changes also compare `JAIC_PROFILE` counts, which do not depend on load. For the LLVM backend time an `-O2` build of Jails or jaison; `JAIC_CODEGEN_UNITS` and `JAIC_SPLIT_UNITS` change how time splits across threads ([LLVM backend](../native/llvm-backend.md)).

Patterns that past profiles found, worth checking first: weak hashes turning a `Table` into one chain (pointer keys use `knuth_hash`; Rust side uses `jaic::fxhash`); linear scans in stdlib or compiler lists on every call (the usual cause of a quadratic metaprogram); filling `Code_Node`/`Type_Info` records member by member instead of the native cached plan ([compiler records](../metaprogramming/compiler-module.md)); per-element loops in the interpreter where one `memset` would do; copying a growing list per item or clearing a whole per-procedure table per block (shows as `memcpy`/`memset` under sema or `code::build`); scanning a token list per lexer token. What remains is interpreter dispatch, sema spread thin, workspaces re-lexing the same files, and the serial `-O2` optimizer.

## compile_bench.py

```sh
cargo build --release -p jaic-cli
python3 tools/compile_bench.py --repeat 5 --out new.json --markdown new.md
python3 tools/compile_bench.py --only focus --modes check --compare benchmarks/results/compile-time-apple-m5.json
```

- **Workloads** (`WORKLOADS`): focus, Jails, sgpu examples (check only), jaison tests, open-jai `33.3_getrect_buttons`, chess-jai, forbear, plus generated `gen-10k`, `gen-60k`, `gen-240k` from `tools/benchgen.py` (seed 1; changing the generator makes old results incomparable). A workload may name `tools/upstream-cases.json` ids whose `setup` runs first.
- The first run is cold (OS cache not primed); the median of the rest is warm and is what `--compare` uses. Peak RSS comes from `os.wait4` on `jaic` (children such as the linker are excluded). `-timings` makes jaic print `jaic-timing: <phase> <seconds> <calls>`: `total`, `front end`, `workspaces`, `run`, `prepare output`, `codegen` (LLVM wall time, multi-threaded for large builds), `link`, `debug info`. Phases nest.
- `--compare` exits 1 when warm time or RSS grew more than `--threshold` (10%) and `--min-seconds` (0.1) or `--min-mib` (16), or a workload failed.
- CI: `compile-bench.yml` is a manual `workflow_dispatch` job on `macos-15` (inputs `repeat`, `only`) that uploads the `compile-bench` artifact and never fails a PR.
- **New workload:** an entry in `WORKLOADS` (corpus directory, `{mode: jaic arguments}`, setup ids; `{tmp}` becomes a per-run temporary directory). Skip projects that write into `corpus/upstream`. **New phase:** wrap code in `timings::time("name", || ...)` in `crates/jaic-cli/src/main.rs`. Tests: `tools/test_compile_bench.py`.

## lsp_bench.py

```sh
cargo build --release -p jai-language-server
python3 tools/lsp_bench.py --repeat 5 --out new.json --markdown new.md
python3 tools/lsp_bench.py --only jails --diagnostics pull --repeat 1      # smoke run
```

- Python-standard-library LSP client; each session starts a fresh server, opens documents like an editor, and times: `startup`, `first_diagnostics`, `hover_first`/`hover_warm`, `completion_member`, `references`, `document_symbols`, `semantic_tokens`, `edit_hover`, `typing_first`/`typing_median`, `completion_member_typed`. `--diagnostics pull,push` runs both styles (pull makes jailsp stop pushing). CPU is sampled outside timed windows; totals and peak RSS come from `wait4`. Positions are picked deterministically (`pick_positions`); `info.picks` in the JSON shows them.
- Failures are data, not aborts: `timeout`, `memory` (`--max-rss`, 6144 MiB), `limit` (document over the server's size caps), `crash`, `error`, `none` (push server published nothing within `--push-wait`), `unsupported` (capability not advertised; not a failure).
- Workloads: `focus`, `focus-main`, `jails`, `chess-jai`, `photon`, `forbear`, `protobuf`, `reflector`, `no-api`, `toml`, `jai-format` (different project shapes, so a speedup tuned to Focus shows as flat elsewhere), generated `gen-60k`/`gen-240k`, and one-file `large-25k`/`large-100k`. A `limit` on any of them is a regression.
- `--only` takes comma-separated terms: a term with `/` matches the whole `server/workload/mode` key; a server name selects its workloads; other terms match the workload name.
- `--server NAME=COMMAND` benchmarks another server (standard requests only; Jails needs `--diagnostics push`). The Jails comparison is in the [jailsp README](../../crates/jai-language-server/README.md#compared-with-jails).
- `--compare` flags warm time, CPU or RSS growth over 20% and 5 ms / 16 MiB. **New metric:** add to `METRICS`, `NEEDS` and a `step(...)` in `run_session`; keep position-picker heuristics in step with `tools/test_lsp_bench.py`.

## Configuration

`JAIC_PROFILE`, `JAIC_NATIVE_LIBS` (worktrees without `target/native-libs`), `[profile.profiling]` in the root `Cargo.toml`. Common flags: `--jaic` (default `$CARGO_TARGET_DIR/release/jaic`), `--repeat`, `--only`, `--out`, `--compare`, `--list`.

## Dependencies

Python 3.11+, a release `jaic` / `jailsp`, the corpus from `tools/fetch_upstreams.py` for project workloads, native libraries for build modes, samply for native profiles.
