# Code formatting

## What it is

The repository uses the pinned nightly rustfmt and shared `rustfmt.toml` for Rust layout. `tools/check_rust_format.py` checks every registered authored package, including the standalone fuzz workspace when its real manifest is present, while authors choose meaningful blank lines between definitions, methods and logical steps.

## How it works

Run the shared check from the repository root:

```sh
python3 tools/check_rust_format.py
```

It first compares authored package manifests under `crates/` against the declared root workspace members and the explicit inventory of standalone manifests that are actually present. An undeclared package fails before formatting checks, so a new or held package cannot silently escape CI. Cargo also formats local path dependencies; the guard deliberately requires an explicit inventory instead of relying on implicit dependency membership. It does not parse Rust or implement another style checker.

The actual checks are standard Cargo/rustfmt commands, run from the repository root. The first always runs; the second runs only when `fuzz/Cargo.toml` is a file:

```sh
cargo fmt --all -- --check
cargo fmt --manifest-path fuzz/Cargo.toml -- --check
```

When both scopes exist, both commands run even if the first finds formatting differences. Public checkpoint `123f1c3` has the main workspace only; fuzz integration is held separately. This check does not require a missing manifest or fabricate one, and adding the real fuzz package automatically enables its formatting scope. A passing format check does not establish fuzz compilation or runtime acceptance. They cover Cargo-managed Rust targets and their modules, including source, tests, examples, benchmarks, build scripts and the fuzz entrypoints whose build feature is disabled. They do not compile crates, link LLVM or format generated target outputs, saved artifact snapshots, private candidates, Jai sources or original reference inputs. Formatting checks leave Rust source unchanged.

Rustfmt expands the configured compact constructs and preserves deliberate blank-line groups, up to two consecutive blank lines. It does not decide where an `impl` changes methods or where a method changes logical stages. For example, these two blank lines are authored choices preserved by the formatter:

```rust
impl Counter {
    fn new(value: usize, limit: usize) -> Self {
        Self {
            value,
            limit,
        }
    }

    fn advance(&mut self, delta: usize) -> usize {
        let next = (self.value + delta).min(self.limit);

        self.value = next;
        next
    }
}
```

A passing mechanical check does not prove that every appropriate logical group has been separated. Review those choices explicitly; the formatter has no forced lower blank-line bound.

## How to change it

Change `rustfmt.toml`, inspect copied representative code, and coordinate any formatting pass after active integration is frozen. Keep formatting changes separate from behavior changes. Use `cargo fmt --all` to format the main workspace deliberately. When the real fuzz manifest is present, also use `cargo fmt --manifest-path fuzz/Cargo.toml`, then rerun the shared check.

Register each new authored crate in the root workspace before claiming complete format coverage. If another standalone workspace is introduced, add its manifest inventory and standard Cargo fmt command to `tools/check_rust_format.py`; do not introduce a custom Rust style parser. A package outside `crates/` also needs an explicit authored-scope inventory entry.

For temporary copied files outside the repository, run the pinned formatter with an explicit copied configuration, for example `rustup run nightly-2026-08-29 rustfmt --config-path /path/to/copied/rustfmt.toml /path/to/copied/example.rs`. This checks candidates without rewriting active main sources. The copied miniature workspace probe verified the main-only and main-plus-fuzz scopes, disabled-feature fuzz target coverage, unchanged check inputs, preserved authored method/logical gaps, and refusal of an unregistered package. It is not a full-tree formatting pass; that remains deferred until integration and package registration are stable.

## Configuration

`rust-toolchain.toml` pins the nightly formatter. `rustfmt.toml` selects edition/style edition 2024, a 100-column width and Unix line endings. `blank_lines_upper_bound = 2` preserves existing groups within that ceiling; there is no `blank_lines_lower_bound` override. Compact empty items, struct literals and functions are expanded, and the single-line `if/else` and `let/else` width allowances are zero. Some options require nightly rustfmt.

The scope guard uses root `Cargo.toml` workspace members/exclusions, package manifests below `crates/`, and the explicit `fuzz/Cargo.toml` standalone inventory entry when that file exists. CI's existing Format check delegates to the same script. CI records the original formatting outcome and continues to compiler tests; its final style enforcement fails if formatting or Clippy failed. This collects later evidence without making a failed formatting scope optional. The fuzz workflow may retain its own direct standalone formatting check as well.

## Dependencies

The check uses Python 3.11 or newer for standard-library TOML reading, Cargo and the pinned Rust toolchain's rustfmt component. It adds no registry dependencies. `.github/workflows/ci.yml` supplies the toolchain and Python; a present `fuzz/Cargo.toml` describes the independently checked fuzz targets.
