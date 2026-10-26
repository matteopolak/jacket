# Rust code coverage

## What it is

Line and function coverage of the Rust crates (`jaic`, `jaic-cli`, `jaic-llvm`, `jai-language-server`, `jailint`), measured with [`cargo llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) over the test suite and the regression sweep. It points new tests at cold sema, interpreter and code generation paths. It is separate from [Jai stdlib coverage](coverage.md) (`jaic -coverage`, `tools/coverage/report.py`), which measures Jai code that a program ran; nothing there overlaps.

## How it works

`.github/workflows/rust-coverage.yml` runs weekly and on demand (Actions, "rust coverage", optional `sweep` input):

1. `cargo llvm-cov nextest --profile ci --no-report` runs the same tests as the `test` job with instrumented binaries.
2. With the sweep on, `cargo llvm-cov show-env` points a build at `target/llvm-cov-target` and sets `LLVM_PROFILE_FILE`; `jaic_sweep.py` runs the corpus, negative and regression sets with that `jaic`, so the compiler processes it spawns add their profiles to the tests'.
3. `cargo llvm-cov report` writes `lcov.info`, an HTML tree and a summary, test files excluded. `tools/coverage/rust_cold.py` turns the lcov into the step summary.

The `rust-coverage` artifact holds `lcov.info`, `html/` (open `html/index.html`), `summary.txt` and `cold.md`. Download it from the run page.

## Reading it

`cold.md` (also the run summary) has coverage per source area (`sema`, `interp`, `parser`, `lexer`, ...), the files with the most uncovered lines, and the functions that never ran, longest first. A long function that never ran is the best target: open it in `html/` (red lines did not execute), then write the smallest program that reaches it:

- a rejected program: a `tests/corpus/negative` case ([testing](testing.md)); the [diagnostic gate](diagnostic-coverage.md) lists messages with no test;
- accepted code: a self-checking `tests/compiler/regressions/*.jai`;
- interpreter or backend behaviour: add the program to the differential set ([testing](testing.md#differential-testing)).

Run it locally with `cargo llvm-cov nextest --profile ci --html` (needs `cargo install --locked cargo-llvm-cov` and `rustup component add llvm-tools`), then `python3 tools/coverage/rust_cold.py` on `cargo llvm-cov report --lcov --output-path lcov.info`.

Generic and macro-expanded code counts once per instantiation, and code reached only by the sweep's child processes needs the sweep step. Percentages are a guide, not a gate.

## How to change it

- Crates measured: the `-p` list in the workflow and the default `--root` list in `rust_cold.py`.
- Exclusions: `--ignore-filename-regex` in the Reports step.
- Version: `llvm_cov` and its checksums in `.github/pins.json`; `tools/ci/install_pinned.py llvm-cov` verifies the prebuilt archive (the release must be at least 14 days old).

## Dependencies

cargo-llvm-cov (pinned), the toolchain's `llvm-tools` component, cargo-nextest, and the jaic sweep for the second step.
