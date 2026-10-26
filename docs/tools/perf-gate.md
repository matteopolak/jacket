# Performance gate

## What it is

`tools/bench/gate.py` fails a change that makes the compiler allocate or execute noticeably more for a fixed set of compiles. `.github/workflows/perf-gate.yml` runs it on pull requests and pushes to main that touch the compiler, the stdlib or the gate itself. [`compile-bench.yml`](benchmarks.md) still measures wall-clock time but cannot block: shared runners are too noisy.

## How it works

Six fixed compiles run single-threaded (`-serial_parse -threads 1`): hello world and a `tools/benchgen.py` program of 2,000 lines (seed 1), each with `-check` and as a native build, a 10,000-line program with `-check`, and `gen-2k/meta`: a build metaprogram that creates a workspace for the 2,000-line program and reads every `Message_Typechecked` (declarations and the nodes of `all`). The last one guards the export of code trees to metaprograms (`docs/metaprogramming/compiler-module.md`); a metaprogram that never names the message skips that work, so a regression there would not show in a phase-only build. Per compile:

| Metric | Source | Threshold |
| --- | --- | --- |
| `alloc_blocks` | `jaic-alloc: total` line of `jaic -timings` (blocks Rust code allocated) | +2% |
| `alloc_bytes` | the same line (bytes) | +3% |
| `instructions` | `valgrind --tool=cachegrind` "I refs" of the `jaic` process (not the linker) | +4% |

The allocation counts are exactly the same from run to run on one machine; instructions vary by a hair with the environment. The thresholds leave room for a runner image update and for ordinary stdlib edits (hello world parses part of the stdlib) while catching a regression of a few percent. A metric more than 5% below the baseline prints a note to update it, so improvements are locked in.

The baseline, `tools/bench/gate-baseline.json`, holds one entry per platform (`linux-x86_64`); counts differ between operating systems and CPUs. A platform without an entry measures and prints but passes, and without valgrind only the allocation metrics are compared.

## Updating the baseline

When growth is intended (a feature that allocates more) or after a deliberate improvement, regenerate it on the CI runner type, since that is where it is compared:

1. Run the "performance gate" workflow by hand on your branch with `update` ticked.
2. Download the `gate-baseline` artifact and replace `tools/bench/gate-baseline.json` with it (or paste the printed JSON into the `linux-x86_64` entry).
3. Commit it with the change that moved the numbers, and say why in the commit.

Locally, `PYTHONPATH=tools python3 tools/bench/gate.py --update --baseline /tmp/mine.json` records your own platform for investigating a change (`--baseline` keeps the committed file untouched).

## How to change it

- Workloads: `workloads()` in `gate.py`; changing one makes the baseline stale, so update it in the same commit.
- Thresholds: `THRESHOLDS`; `--threshold 2` doubles them for one run.
- To gate another platform, add a job on its runner and its entry to the baseline.

## Dependencies

A release `jaic` built with `-timings` support, valgrind (`apt`, Linux only), `tools/benchgen.py`.
