# Performance gate

`tools/bench/gate.py` fails a change that makes the compiler allocate or execute noticeably more for a fixed set of compiles. `.github/workflows/perf-gate.yml` runs it on pull requests and pushes to main that touch the compiler, the stdlib or the gate. [`compile-bench.yml`](benchmarks.md) measures wall-clock time but can't block, since shared runners are noisy.

## How it works

Six fixed single-threaded compiles (`-serial_parse -threads 1`): hello world and a 2,000-line `tools/benchgen.py` program (each `-check` and native), a 10,000-line program with `-check`, and `gen-2k/meta`, a metaprogram that reads every `Message_Typechecked` (guards the export of code trees to metaprograms). Per compile:

| Metric | Source | Threshold |
| --- | --- | --- |
| `alloc_blocks` | `jaic-alloc: total` line of `jaic -timings` | +2% |
| `alloc_bytes` | same line | +3% |
| `instructions` | `valgrind --tool=cachegrind` "I refs" of the `jaic` process | +4% |

Allocation counts are exact from run to run; instructions vary a hair. A metric more than 5% below baseline prints a note to update it, so improvements are locked in. The baseline, `tools/bench/gate-baseline.json`, has one entry per platform (`linux-x86_64`). A platform without an entry measures but passes; without valgrind only allocations are compared.

## Updating the baseline

When growth is intended, regenerate it on the CI runner type:

1. Run the "performance gate" workflow on your branch with `update` ticked.
2. Download the `gate-baseline` artifact and replace `tools/bench/gate-baseline.json`.
3. Commit it with the change that moved the numbers, and say why.

Locally, `PYTHONPATH=tools python3 tools/bench/gate.py --update --baseline /tmp/mine.json` records your own platform without touching the committed file.

## How to change it

- Workloads: `workloads()` in `gate.py`. Changing one makes the baseline stale; update it in the same commit.
- Thresholds: `THRESHOLDS`; `--threshold 2` doubles them for one run.
- Another platform: add a job on its runner and its baseline entry.

Needs a release `jaic`, valgrind (Linux only) and `tools/benchgen.py`.
