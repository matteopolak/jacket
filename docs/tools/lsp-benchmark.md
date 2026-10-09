# Language server benchmark

## What it is

`tools/lsp_bench.py` measures the latency, CPU time and memory of a Jai language server (`jailsp` by default)
over stdio, on real projects from the [upstream corpus](upstream-corpus.md) and on generated programs. For each
workload it starts a fresh server, opens a document the way an editor does, and times a fixed script of
requests: first diagnostics, hover, hover after an edit, completion while typing, references, symbols and
semantic tokens. It writes JSON and Markdown and can compare against an earlier run. It exists to show where
the time goes while the server is made incremental (see [semantic analysis](../compiler/language-server.md#semantic-analysis)).
`tools/compile_bench.py` ([compile-time benchmark](compile-time-benchmark.md))
is the model for the options and the results format.

## How it works

- **Client.** Python standard library only. `Client` spawns the server, a reader thread decodes
  `Content-Length` frames (`FrameDecoder`) and timestamps each message the moment it is complete, so waiting in
  the main thread adds no polling delay. Server requests (`workspace/configuration`, `client/registerCapability`,
  refreshes) are answered with `null`. `initialize` sends a workspace folder and capabilities of a typical
  editor: Markdown hover, snippet completion, hierarchical symbols, relative semantic tokens, and either
  `textDocument.diagnostic` (**pull**, which makes jailsp stop pushing) or only `publishDiagnostics` (**push**).
  `--diagnostics pull,push` (the default) runs both styles; their keys are `server/workload/pull` and `/push`.
- **Session.** One session is `initialize`, `initialized`, `didOpen` of the workload's other documents, `didOpen`
  of the measured document, then the metrics below in this order. The first session of a workload is **cold**
  (OS file cache not primed), the median of the others **warm** (`--repeat`, default 5). The server restarts every
  session, so its own caches are always empty at the start.
- **Metrics** (`METRICS`):
  `startup` (launch to `initialize` response), `first_diagnostics` (`didOpen` to the pull answer or first
  `publishDiagnostics`), `hover_first`, `hover_warm` (median of 5 repeats, no edit), `completion_member`
  (after a `.` already in the text), `references` (of the enclosing procedure's name), `document_symbols`,
  `semantic_tokens` (full), `edit_hover` (incremental `didChange` inserting a comment line after the hover line,
  then hover; median of 3 different edits, each a new text), `typing_first` / `typing_median` (a new line in the
  same body typed one character per `didChange`, completion after each; the first and the median; the line is
  deleted afterwards) and `completion_member_typed` (one edit adding `name.` and completion after the dot).
- **Positions** (`pick_positions`) are deterministic: comments and strings are masked, procedure bodies are
  top-level `name :: (...) ... {` up to a `}` at column 0, and the hover is the identifier 60% of the way
  through all identifiers of all bodies (not a keyword, member or declaration). The same body gives the insertion
  line (after the hover line, same indentation), the procedure name for references, and a `base.field` use for
  member completion. Columns are UTF-16. Printing `info.picks` in the JSON shows what was used.
- **CPU and memory.** Between steps (outside the timed window) the script samples the server's user and system
  CPU seconds and RSS (`/proc` or `ps`); each metric records the CPU seconds spent during its step and the RSS
  after it. Totals per session, and `peak_rss_mib`, come from `os.wait4` after shutdown, so they include everything.
  On macOS the sample comes from `proc_pidinfo(PROC_PIDTASKINFO)` through `ctypes`, which has microsecond
  resolution (scaled by `mach_timebase_info`); Linux reads `/proc/<pid>/stat` (10 ms ticks); anywhere else `ps`
  (10 ms). The `typing_median` metric has no CPU of its own (it shares `typing_first`'s step, which includes
  the first character only).
- **Failures are data.** Every request has `--timeout` (60 s). A wait also stops when the server's RSS passes
  `--max-rss` (6144 MiB), because a runaway request can eat a 16 GiB machine. Cells show a status where there
  is no number:
  - `timeout`, `memory`: no answer in time, or the RSS cap; the session ends and its later metrics are `skipped`;
  - `limit`: the server refused the document or died because of a size limit (`didOpen` over `Limits`:
    32 MiB per document, 64 MiB per message; "document is not open in this session" answers also count);
  - `crash`, `error`: the server exited, or answered with an error;
  - `none`: a push-mode server published nothing within `--push-wait` (15 s) for the document;
  - `unsupported`: the server did not advertise the capability in `initialize`, or answered "method not found".
    This is not a failure and does not make `--compare` fail.
- **Workloads** (`WORKLOADS`):
  - `focus`: workspace = the Focus checkout, `src/editors.jai` (200 KiB) with `first.jai` open. `focus-main` has
    `src/main.jai` open instead, so the program is checked from there and not from the build metaprogram.
  - `jails`: `server/program.jai` with `server/main.jai` open; `chess-jai`: `movegen.jai` with `build.jai` open.
  - `gen-60k`, `gen-240k`: [`tools/benchgen.py`](benchmark-generator.md) programs (seed 1) split into 12 and 48
    files; the middle `part_N.jai` is measured, `main.jai` is open.
  - `large-25k`, `large-100k`: one-file programs (`benchgen` with `files=1`) of about 0.8 MB and 3.3 MB, the single
    large documents (the old caps of 256 KiB per document and 1 MiB per message made them `limit` rows).
  The corpus is only read; generated programs are written to a temporary directory.
- **Output.** The JSON (format 2) holds the date, machine, per-server version and commit, settings, and per
  `server/workload/mode` key: each metric's cold, warm, all runs, CPU, RSS and status, `info` (document size,
  diagnostic counts by code, whether hover answered, item and result counts) and totals. The Markdown has four
  tables (warm time, cold time, warm CPU, warm RSS after the step), a line per workload saying what the first
  diagnostics contained, and the failures.

```sh
cargo build --release -p jai-language-server
python3 tools/lsp_bench.py --repeat 5 --out new.json --markdown new.md       # everything, both diagnostic styles
python3 tools/lsp_bench.py --only jails --diagnostics pull --repeat 1        # smoke run
python3 tools/lsp_bench.py --only large --diagnostics pull --compare old.json
```

`--only` takes comma-separated terms, any of which may match. A term with a `/` is a substring of the whole
`server/workload/mode` key (`jailsp/focus`, `/jails/pull`); the name of a server (`jailsp`) selects all its workloads;
any other term is a substring of the workload name only, so `--only jails` runs the `jails` workload and not the
server `jailsp`, and `--only large` runs both large files.

`--compare` prints each regression and exits 1 when warm time, warm CPU time or step RSS of a metric, or total
CPU seconds or peak RSS of a workload, grew by more than `--threshold` (20%) and also by more than
`--min-seconds` (5 ms; CPU uses at least 20 ms) or `--min-mib` (16), when a metric that had a number is now a
failure, or when a workload failed to start. Compare only runs from the same machine; run one benchmark at a
time, since the numbers are wall time.

### Reading the numbers

- The hover position is the identifier 60% of the way through the procedure bodies. When it answers null, up to
  seven other identifiers are tried (untimed) and the first that answers is used for the other hover metrics and
  for edit-then-hover; the Markdown says how many were tried (`hover_retries` in the JSON). A workload that still
  says `hover null` has no hoverable fact at any of them.
- A numeric hover on a workload whose first diagnostics list `jai-check` errors may be an answer from
  incomplete facts, and `hover null` in the Markdown means the server had no fact there. Corpus projects whose
  build is a metaprogram (Jails, chess-jai, Focus) are only partly checkable by the server, which is a property of
  the project and server, not of this script.
- `jai-limit` diagnostics mean the syntax layer hit one of its safety caps (`Limits::tokens`, `Limits::rows`,
  the nesting budget): symbols and syntax navigation are then empty or partial for that version. The caps are far
  above the benchmark's files, so a `jai-limit` here is a regression.

## How to change it

- **New workload**: add an entry to `WORKLOADS`: a corpus directory with `document` and `also` (opened first),
  or `generated: (lines, seed, files)`. Changing the generator changes these programs, so results from before
  and after are not comparable. Do not choose a project whose files the server's build would rewrite, and check
  the corpus with `python3 tools/verify_upstreams.py` after a run.
- **New metric**: add it to `METRICS`, to `NEEDS` (the capability it requires) and a `step(...)` in
  `run_session`. Anything inside the step is timed; sampling is done outside it.
- **Another server**: `--server NAME=COMMAND` (repeatable; the command is split like a shell would and must speak
  LSP over stdio), for example `--server jails=/path/to/jails`. Results are keyed `NAME/workload/mode` and
  `--only` matches those keys. The script sends only standard requests, so a server without them gets
  `unsupported` cells. Jails answers pushes only, so run it with `--diagnostics push`.
- **More client behavior**: the script does not pull diagnostics again after a change; an editor that does adds
  that work to every edit. Add it in `edit_hover` and `typing` if that is the case to measure.
- Update the position picker's heuristics together with `test_lsp_bench.py`.
- Tests: `python3 -m unittest tools/test_lsp_bench.py` (framing, position picking, medians, CPU parsing,
  comparison, Markdown).

## Configuration

- `--server NAME=CMD` (default `jailsp=$CARGO_TARGET_DIR/release/jailsp`, else `target/release/jailsp`),
  `--repeat`, `--only`, `--diagnostics`, `--timeout`, `--push-wait`, `--max-rss`, `--out`, `--markdown`,
  `--compare`, `--threshold`, `--min-seconds`, `--min-mib`, `--list`.
- The server's own configuration is not touched (`JAIC_STDLIB`, `jai.toml` of a workload apply as for an editor).

## Dependencies

- Python 3.11+ (standard library), a release `jailsp`, `ps` (or `/proc`), the corpus from
  `tools/fetch_upstreams.py` for the corpus workloads and `tools/benchgen.py` for the generated ones.
