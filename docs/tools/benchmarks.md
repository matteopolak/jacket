# Benchmarks and profiling

## What it is

One harness, `tools/bench/`, times the compiler, the language server, the linter and the formatter. Each subcommand writes JSON and a Markdown summary, takes the median of N runs, reports the spread, and can compare against an earlier run.

```sh
python3 tools/bench/main.py compile --compiler jaic=target/release/jaic     # compile time, stages, generated-code speed
python3 tools/bench/main.py lsp                                             # jailsp latency by request kind
python3 tools/bench/main.py lint                                            # jailint
python3 tools/bench/main.py fmt                                             # jaifmt, native and wasm
python3 tools/bench/main.py compile --compare target/bench/old.json         # exit 1 on regressions
```

`tools/regex_bench.py` with `benchmarks/regex/` is separate: `Extensions/Regex` against other engines (results in `benchmarks/results/regex.md`). `benchmarks/results/` holds only the regex numbers: the compile and language-server baselines the old harnesses wrote there were removed. They measured other workloads in another format, so nothing could compare against them, and a committed baseline goes stale with every change; keep a baseline per machine from `--out` and pass it to `--compare` (a report in any other format is refused up front).

## How it works

Results go to `target/bench/<subcommand>.json` and `.md` (gitignored; `--out` and `--markdown` change that). Every result is one benchmarked thing (a workload under one compiler and mode, one request kind, one command), with three numbers, each summarized as median, min, max, median absolute deviation and p95:

- `wall`: wall-clock seconds.
- `cpu`: user + system CPU seconds of the process and the children it waited for. With threads this exceeds wall time.
- `rss_mib`: peak resident set of the process itself (from `wait4`; a child such as a linker is not in it).

Before timing, the harness reads the binary and the workload's files, then does `--warmup` untimed runs (default 1) so the OS file cache is warm. `--repeat` (default 5) timed runs follow.

`--compare old.json` flags a result when its median wall time, CPU time or peak RSS grew by more than `--threshold` (relative), more than `--min-seconds` / `--min-mib` (absolute, so tiny workloads do not trip), and more than three median absolute deviations of either run (so noise does not trip). A result that worked before and fails now is a regression too, and so is growth in `interp_instructions`. Exit status is 1 for a regression or a failed workload. Compare only runs from one machine, one benchmark at a time; shared CI runners are too noisy to gate on.

Common options: `--repeat`, `--warmup`, `--only A,B` (substrings of result keys), `--list`, `--timeout`, `--out`, `--markdown`, `--compare`, `--threshold`, `--min-seconds`, `--min-mib`.

### compile

```sh
cargo build --release -p jaic-cli
python3 tools/bench/main.py compile --repeat 5
python3 tools/bench/main.py compile --categories compile-time,startup --modes check,build
python3 tools/bench/main.py compile --only focus/check --compare target/bench/old.json
```

Result keys are `<workload>/<mode>/<compiler>`. Modes: `check` (`-check`), `build` (default build), `optimized` (`-o`), `interp` (`-interp`). The workloads (`tools/bench/workloads.py`):

| Category | Workloads |
| --- | --- |
| startup | `hello` |
| lots of code | `gen-2k`, `gen-10k`, `gen-60k`: `tools/benchgen.py` programs, seed 1 (changing the generator makes old results incomparable) |
| real projects | focus, Jails, sgpu examples, jaison tests, the open-jai drawing board, forbear from the [upstream corpus](upstream-corpus.md) (`python3 tools/fetch_upstreams.py`) |
| compile-time heavy | `benchmarks/compile/`: `run-loops` (hot `#run` loops), `insert-meta` (`#insert` of generated procedures, structs and enums), `polymorph-explosion`, `constant-tables`; and `benchmarks/compile-time.jai` |
| stdlib heavy | `stdlib-imports`: sixteen modules imported and used |
| interpreter | `benchmarks/interp-*.jai` under `-interp` |
| generated-code runtime | `runtime-compute` (a numeric loop, a sieve, hash table traffic, a sort) and `gen-10k`: the built executable is run and timed as `<workload>/run-<mode>/<compiler>`, its size is in the JSON (`binary_bytes`) |

Sizes come from the same lexer `tools/jaistats.py` uses: `info.source` holds files, code lines, procedures and structs, and the tables show seconds per thousand lines.

**Stages.** The stage table comes from a phase-timing metaprogram that every compiler runs (see [Comparing against another compiler](#comparing-against-another-compiler)). A second table, for jaic only, comes from its own report: `jaic -timings` prints `jaic-timing: <phase> <seconds> <calls>` on stderr: `front end`, `workspaces`, `run`, `prepare output`, `codegen`, `link`, `debug info`, `total`, and `interp`. `interp` is the time compile-time code (`#run`, metaprograms, macros) spent executing in the interpreter; it is measured inside `Interp::call` and overlaps `front end`, which also covers lowering the procedures that code needs. `tools/bench/stages.py` maps those phases onto the same bucket names:

| Bucket | jaic phases |
| --- | --- |
| front end | `front end` minus `interp` |
| compile-time execution | `interp` plus `run` (the program itself under `-interp`) |
| codegen | `prepare output` + `codegen` + `debug info` |
| link | `link` |
| other | wall time not in any bucket (process start and exit included) |
| total | wall time of the process |

For a build metaprogram that compiles workspaces, backend time is also inside its `front end` and `interp`, so buckets can add up to more than the total; `other` never goes negative. With `-interp`, `-profile` adds one untimed run that records `info.interp_instructions`, the interpreted IR instruction count: nearly deterministic, so it shows small interpreter changes that wall time hides.

### lsp

```sh
cargo build --release -p jai-language-server
python3 tools/bench/main.py lsp --repeat 5
python3 tools/bench/main.py lsp --only gen-240k --diagnostics pull
```

Each session starts a fresh server over stdio, opens a workspace as an editor would and runs a fixed script on one document. `--repeat` sessions are pooled, so p50 and p95 are over every sample of every session. Workloads (`lsp_workloads.py`): `gen-60k` (12 files) and `large-25k` (one file) from benchgen, and `repo` (this repository, opened at its root, with `stdlib/Extensions/Jai_Format/format.jai` as the document); `gen-240k` and `large-100k` run when named with `--only`.

Probes (`lsp_session.py`), keyed `<server>/<workload>/<probe>`: `initialize` (launch to the response), `index_ready` (until workspace symbols find the document's procedures), `did_open` (until diagnostics), `did_change_keystroke`, `did_change_paste`, `did_change_large` (each until the diagnostics for the edit arrive), `completion_member`, `completion_global`, `completion_import`, `hover_first`, `hover_warm`, `definition`, `references`, `rename`, `semantic_tokens_full`, `semantic_tokens_delta`, `document_symbols`, `workspace_symbols`, `inlay_hints`, `signature_help`, `formatting`, `range_formatting`, `file_watcher` (a file changed on disk until its new symbol is searchable; generated workspaces only) and `long_session` (many edits; `info.rss_growth_mib`). `session` has the whole session's wall time, CPU and peak RSS.

A request the server does not advertise is `skipped`, not a failure. An answer that is null or empty is counted (`empty_answers`), because a fast answer to nothing is not a fast answer. A timeout, crash or memory blow-up (`--max-rss`) ends a session; the probes it did not reach are skipped. `--server NAME=COMMAND` times another server (repeatable); `--diagnostics pull` makes the client pull instead of receiving pushes.

`crates/jai-language-server/examples/edit_bench.rs` stays as it is: it drives the library in-process (no stdio, one document), which the harness does not replace.

### lint and fmt

`lint` runs `jailint` on a generated 25k-line file, on the densest `tests/lint` case, `--fix` on a fresh copy of it for every run, and over all `tests/lint` programs with `-j 1`, `2` and `4`. The whole-repository scenario (`repo`) takes about five minutes of ten cores, so it runs only when named: `lint --only repo`. `fmt` runs `jaifmt --check` on the repository paths CI formats and on a 100k-line file, `--check` and rewrite of formatted and of unformatted input (a formatted sample with its indentation removed and its spaces collapsed, `unformat` in `cmd_fmt.py`), stdin with both, and the same stdin runs through `target/jaifmt.wasm` under node. Range formatting is reported as skipped while `jaifmt --help` lists no range option. Build the tools first (`cargo build --release -p jailint`; `jaic jaifmt/build.jai`, with `- wasm` for the wasm build).

## Comparing against another compiler

```sh
python3 tools/bench/main.py compile --compiler jaic=target/release/jaic --compiler other=/path/to/other/jai
```

A binary named `jaic*` gets jaic's options; any other is driven with the reference compiler's command line (`dialects.py`). Fairness rules the harness enforces:

- **Same machine, same flags.** Each compiler builds the same files with the equivalent option for each mode. Each uses its own standard library; the harness passes no module paths.
- **Interleaved runs.** Round 1 runs compiler A then B, round 2 again A then B, so thermal drift and background load hit every compiler alike.
- **Warm cache.** Binaries and sources are read first and one untimed run precedes the timed ones.
- **Correctness gate.** A workload is timed only if every compiler builds it and, for workloads whose output is deterministic, prints the same output (the compile-time output, and for builds the program's). Otherwise it is listed under "Not timed" as `skipped: <compiler>: <why>` (for example `exit 1: syntax the other compiler rejects`, or `output differs`). With one compiler a failure is a `failed` result and exit status 1.
- **CPU time next to wall time.** Compilers use threads differently (jaic's code generation is parallel); a wall-time win with a CPU-time loss, or the reverse, is information. Read both.
- **Check and interpreter modes.** The reference has neither. `check` compiles the program from a wrapper file that sets `do_output=false`, and `interp` adds `#run main();` to it. Both therefore time the front end plus compile-time execution of the real program; the ratio compares those, not an identical code path.
- **Stages.** The reference prints no timings, so stages are not read from compiler output. Instead every compiler, jaic included, builds the program through one metaprogram, `tools/bench/phase_timing.jai`. It creates a workspace, intercepts the compiler's messages (`Message_Phase` and `COMPLETE` from the `Compiler` module) and prints `bench-phase: <name> <seconds>` as each arrives; `phases.py` turns those lines into buckets. `--phase-repeat N` (default 3, interleaved like the rest) sets how many such runs each compiler gets per workload; the wall, CPU and RSS columns come from the direct invocations, not these. What the messages can and cannot tell apart:
  - `front end` runs to the last `TYPECHECKED_ALL_WE_CAN`. Parsing, type checking and `#run` execution all happen before it and no message marks them off from each other, so `compile-time execution` is not reported (`-`) rather than guessed. Time spent in `#run` is inside `front end` for both compilers.
  - In build and optimized modes the metaprogram sets `use_custom_link_command`, and on `READY_FOR_CUSTOM_LINK_COMMAND` it links the compiler's object files itself with the system linker (`link_command.jai`: `xcrun ld` with the SDK from `xcrun` on macOS, `cc` on Linux; no custom link on other systems, which fall back to the compiler's own link), times that command and prints it as `bench-phase: link <seconds>`. Both compilers link through the same system linker, so `link` is identical work and comparable. It also means a compiler's bundled linker is never used: the reference's own `lld` is blocked by Gatekeeper on macOS, so this is the only way it can build here at all.
  - `codegen` is the last typecheck message to `READY_FOR_CUSTOM_LINK_COMMAND`, so it includes writing the object files.
  - Comparing compilers, build and optimized runs of the timed columns go through this metaprogram for every compiler (`JobRunner.driver`), not just the stage runs, so the executables are linked the same way and the wall times compare like with like. A single compiler, and workloads with their own build metaprogram (`own_output`), use the compiler's native command line; the reference cannot build those here.
  - The metaprogram's own start-up (loading `Basic` and `Compiler`, creating the workspace) is before the first timestamp and shows in `other` and `total`; `total` is the wall time of the phase run, not of the direct run, so it is larger than the wall column. jaic interprets the metaprogram, which also adds to its front end; compare the ratios as "this method on both", not as the speed of a plain `jaic file.jai`.
  - Builds with their own build metaprogram (`own_output` workloads) are not covered: the wrapper would build their entry with default options. They get no stage rows.
  - jaic's own `-timings` stays as a second table (`detail` in the JSON), jaic only and without ratios: it is finer (it splits `#run` time out) but is not the same measurement.
- **Sandboxed.** `compile` runs every compiler and every program it builds through `tools/bench/sandbox.py`: no network, file contents readable only from the system, the developer tools, Homebrew (projects link its libraries), each compiler's install (the folder above `bin/`, or for jaic its stdlib, `target/native-libs` and the shared libraries it links) and `--sandbox-read` paths, and writes only to the run's scratch folder. Workloads from the repository and the corpus are copied there first; a corpus workload in a project subfolder copies the whole project (it may import `../module.jai`), with links to sibling projects copied as what they point to. macOS uses `sandbox-exec`, Linux `bwrap`; `--no-sandbox` turns it off (Windows has no sandbox). The sandbox does not get past Gatekeeper: a quarantined reference binary still shows its popup until the quarantine flag is cleared by hand.
- **Never commit these results.** They are written under `target/bench/`, which is gitignored. Do not copy another compiler's numbers, paths or output into the repository, its docs, commit messages or issues.

## How to change it

- **A project built by its own metaprogram that a second compiler cannot run:** benchmark what the metaprogram builds instead. `jails` compiles `server/main.jai` with `Plan(..., import_dirs=('modules',))`, which passes the module folder to every compiler (`-I` for jaic, `-import_dir` for the reference) and to the phase-timing workspace (`IMPORT_DIRS` in `phase_timing.jai`). Its own `build.jai` was not usable here: the reference links through a bundled linker that fails on a new SDK. A plan's entry may be in a subfolder (`Request.entry_arg` is relative to the project).
- **New compile workload:** add it to `all_workloads()` in `tools/bench/workloads.py`: a source (`RepoDirectory`, `CorpusProject`, `Generated`), a category, and a `Plan` per mode (entry file, metaprogram arguments after `-`, and whether the harness may redirect the executable: a project whose metaprogram picks its own output uses `own_output`). Set `compare_output` when the output is deterministic and `runtime` to time the executable. Compile-time-heavy programs live in `benchmarks/compile/`.
- **New jaic phase:** wrap the code in `timings::time("name", || ...)` in `crates/jaic-cli/src/main.rs`, add it to `JaicTimings` in `stages.py` (the `-timings` detail table), and update `timings_report_phases` in `crates/jaic-cli/tests/cli.rs`.
- **New LSP probe:** write a function `(Session) -> list of seconds` in `lsp_session.py`, add a `Probe` to `PROBES` with the server capability it needs. Position heuristics are in `lsp_positions.py`.
- **New lint or fmt scenario:** add a `Scenario` in `cmd_lint.py` / `cmd_fmt.py`.
- **Another kind of compiler:** subclass `Dialect` in `dialects.py` (command line per mode, and `source_command` for the phase metaprogram) and choose it in `parse_compiler`. If the compiler's messages differ, change `phase_timing.jai` and `PhaseTiming` in `phases.py`.
- **The link command:** edit the per-OS `LINKER` table in `link_command.jai`; `test_link_command.py` runs `link_arguments` through jaic and checks the arguments.
- **Another phase boundary:** add a `mark("name")` in `phase_timing.jai` and read it in `PhaseTiming`.
- Gotchas: every run happens in its own process group, killed on timeout and again when the command exits, so a stuck linker cannot outlive the benchmark. `jaic -timings` goes before a lone `-`, which starts the metaprogram's arguments. The reference writes `.build` next to the file it compiles, so the harness compiles check and interpreter runs from a wrapper in a scratch directory. Seatbelt matches real paths (`/var/folders` is `/private/var/folders`), and `xcrun` caches tool lookups in the per-user temporary folder whatever `TMPDIR` says, so the profile allows its `xcrun_db` files; without them every link is a second slower. Tests: `python3 -m unittest discover -s tools/bench -p 'test_*.py'`, run by CI's policy step.

## Profiling

- **Interpreter profile.** `-profile` makes every interpreter count calls, blocks and instructions per procedure (self counts, merged in `crates/jaic/src/interp/profile.rs`); the CLI prints the top 40 to stderr plus a breakdown by IR instruction kind and by folded op. Counts stay comparable across interpreter changes because they count IR. Off, it costs one `Option` check per call and block.
- **Native profile.** `cargo build --profile profiling -p jaic-cli`, then `samply record --save-only --unstable-presymbolicate -o prof.json.gz -- target/profiling/jaic first.jai -check` and `python3 tools/profile_report.py prof.json.gz --top 30` (`--callers NAME`, `--within NAME`, `--without NAME`). It reads the busiest thread, which is the 1 GiB compiler worker. For a hot line inside a big inlined function, resolve leaf addresses with `atos -i -o target/profiling/jaic.dSYM -l <load address> <addresses>`.
- **Comparing two builds.** Copy the old binary (`cp target/release/jaic /tmp/jaic-before`), alternate the two on the same workload at least 5 times each, compare medians and minimums (`bench compile --compiler before=/tmp/jaic-before --compiler after=target/release/jaic`, or `--out` and `--compare` across two runs). Record peak memory with `/usr/bin/time -l` (macOS) or `-v` (Linux). For interpreter changes also compare `JAIC_PROFILE` counts, which do not depend on load. For the LLVM backend time an `-O2` build of Jails or jaison; `JAIC_CODEGEN_UNITS` and `JAIC_SPLIT_UNITS` change how time splits across threads ([LLVM backend](../native/llvm-backend.md)).

Patterns that past profiles found, worth checking first: weak hashes turning a `Table` into one chain (pointer keys use `knuth_hash`; Rust side uses `jaic::fxhash`); linear scans in stdlib or compiler lists on every call (the usual cause of a quadratic metaprogram); filling `Code_Node`/`Type_Info` records member by member instead of the native cached plan ([compiler records](../metaprogramming/compiler-module.md)); per-element loops in the interpreter where one `memset` would do; copying a growing list per item or clearing a whole per-procedure table per block (shows as `memcpy`/`memset` under sema or `code::build`); scanning a token list per lexer token. What remains is interpreter dispatch, sema spread thin, workspaces re-lexing the same files, and the serial `-O2` optimizer.

## Configuration

- `CARGO_TARGET_DIR` (or `target/`) locates the default `jaic`, `jailsp` and `jailint`; `--compiler`, `--server`, `--jailint`, `--jaifmt` and `--wasm` override. `JACKET_OUTPUT_DIR` moves `target/bench/`. `--no-sandbox` and `--sandbox-read PATH` (compile).
- `-profile` (interpreter profile), `JAIC_NATIVE_LIBS` (worktrees without `target/native-libs`), `[profile.profiling]` in the root `Cargo.toml`.
- The CI workflow `compile-bench.yml` is a manual `workflow_dispatch` job on `macos-15` (inputs `command`, `repeat`, `only`) that uploads `bench.json` and `bench.md`; it never fails a PR.

## Dependencies

Python 3.11+ (standard library only), a release `jaic`, `jailsp`, `jailint` and `jaifmt` as needed, the corpus from `tools/fetch_upstreams.py` and native libraries for the real-project workloads, node 24 for the wasm formatter, samply for native profiles. Internal: `tools/benchgen.py` (generated programs), `tools/jaistats.py` (workload size), `tools/repo_output.py` (output directory).
