# Rust code coverage

Line and function coverage of the Rust crates (`jaic`, `jaic-cli`, `jaic-llvm`, `jai-language-server`, `jailint`), measured with [`cargo llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) over the test suite and the regression sweep. It points new tests at cold sema, interpreter and codegen paths. Coverage of Jai code is separate: [coverage](coverage.md).

## How it works

`.github/workflows/rust-coverage.yml` runs weekly and on demand (optional `sweep` input):

1. `cargo llvm-cov nextest --profile ci --no-report` runs the `test` job's tests with instrumented binaries.
2. With the sweep on, `jaic_sweep.py` runs the corpus, negative and regression sets with an instrumented `jaic`, adding its profiles.
3. `cargo llvm-cov report` writes `lcov.info`, `html/` and a summary; `tools/coverage/rust_cold.py` turns the lcov into `cold.md`.

The `rust-coverage` artifact holds all of these. `cold.md` lists coverage per source area, the files with the most uncovered lines, and the functions that never ran, longest first. For a cold function, open it in `html/` and write the smallest program that reaches it:

- rejected program: a `tests/corpus/negative` case ([testing](testing.md)); the [diagnostic gate](diagnostic-coverage.md) lists messages with no test;
- accepted code: a self-checking `tests/compiler/regressions/*.jai`;
- interpreter or backend behaviour: the [differential set](testing.md#differential-testing).

Locally (needs `cargo install --locked cargo-llvm-cov` and `rustup component add llvm-tools`):

```sh
cargo llvm-cov nextest --profile ci --html
cargo llvm-cov report --lcov --output-path lcov.info
PYTHONPATH=tools python3 tools/coverage/rust_cold.py
```

Generic and macro-expanded code counts once per instantiation. Percentages are a guide, not a gate.

## How to change it

- Crates measured: the `-p` list in the workflow and the default `--root` list in `rust_cold.py`.
- Exclusions: `--ignore-filename-regex` in the Reports step.
- Version: `llvm_cov` and its checksums in `.github/pins.json`; `tools/ci/install_pinned.py llvm-cov` verifies the archive.
