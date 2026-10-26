# Language server oracle checks

Checks what the language server answers against what the compiler knows, on every file `jaic -check` accepts: stdlib tests, `tests/compiler`, `examples/`, the reviewed upstream corpus and `jaigen` programs. Nothing needs expected answers; accepted code is the oracle.

## How it works

`tools/lsp/oracle.py` picks the files, keeps those `jaic -check` accepts, and passes them in chunks to the `lsp_oracle` example (`crates/jai-language-server/examples/lsp_oracle.rs`, with `lsp_oracle/{checks,metamorphic,oracle}.rs`). The example opens each file in a `Session` and prints one JSON report per file:

| check | finding kinds | what fails |
|---|---|---|
| a | `a-error`, `a-request`, `a-crash` | an error diagnostic on accepted code, a request that errors, a panic or hang |
| a | `a-dead-ran` | a region the server marks unreachable whose marker statement prints when the program runs (the script inserts `print` markers and runs the program) |
| b | `b-hover`, `b-definition`, `b-definition-target` | an identifier the compiler resolved (from the compiler's recorded refs) with no hover, no definition, or a definition somewhere other than its declaration |
| c | `c-blank`, `c-format`, `c-edit` | hover, definition and diagnostics change beyond shifted positions after inserting blank lines or running `jaifmt`; editing an open session into that text differs from opening it fresh |
| d | `d-member`, `d-literal` | completion after `value.` misses a member the code uses there; completion in a struct literal misses a field |

The blank-line comparison ignores `line:column` pairs in hover text, since a constant that prints where a literal is changes with the layout. Member completion (`d-member`) in a polymorphic body offers the members of every instance's receiver. `jaigen` programs have no reversed ranges and no compound assignment that reads its target, so the lints on those stay quiet.

A chunk that crashes or hangs is rerun file by file, so one bad file does not hide the others. Upstream files run inside the corpus sandbox (`tools/corpus/upstream_sandbox.py`), as the upstream sweep does; cases that need special build setup (`args`, `files`, `setup`, ...) or are still `"review": "pending"` are skipped.

Findings are keyed `path|kind|name` and compared with `tools/lsp/oracle-known.json`. A key missing from that file fails the run (exit 1); a known key that is no longer found is printed so it can be removed.

## Running it

```sh
cargo build --release -p jaic-cli -p jai-language-server --example lsp_oracle
PYTHONPATH=tools python3 tools/lsp/oracle.py --sets examples,compiler --jobs 2
PYTHONPATH=tools python3 tools/lsp/oracle.py --sets upstream                 # sandboxed
PYTHONPATH=tools python3 tools/lsp/oracle.py path/to/file.jai                # just these files
```

One file, by hand, with the hover, definition and completion at a position (1-based line, 0-based character):

```sh
target/release/examples/lsp_oracle --probe 12:8 path/to/file.jai
```

CI: `.github/workflows/lsp-oracle.yml` runs nightly (and on dispatch, with a `sets` input) in four shards over stdlib, compiler, examples and jaigen, and uploads the reports. Upstream is local only.

## How to change it

- New check: add a finding kind in `checks.rs` (or `metamorphic.rs` for edit-based ones) and gate it on a letter in `--checks`.
- A false positive that is the check's fault: narrow the check (e.g. `d-member` skips union tag declarations and incomplete lists). A real server gap that will not be fixed now: run with `--update-known` and commit the new `oracle-known.json` lines, then log the gap.
- `--update-known` only rewrites keys for the files checked in that run, so a sharded or set-limited run keeps the others.
- Memory: each oracle process compiles each file several times. Keep `--jobs` low on a 16 GiB machine and do not run it next to `cargo test`.

## Configuration

| flag | default | meaning |
|---|---|---|
| `--sets` | `stdlib,compiler,examples,jaigen` | which file sets |
| `--jaic`, `--oracle`, `--jaifmt` | release builds in `$CARGO_TARGET_DIR` / `target/` | binaries used |
| `--jaigen N` | 20 | jaigen seeds `0..N`, written under the target dir |
| `--shard I/N` | `1/1` | every Nth file from the Ith |
| `--jobs`, `--chunk` | 2, 8 | parallel processes, files per oracle process |
| `--probes` | 300 | identifiers sampled per file |
| `--checks` | `abcd` | which checks run |
| `--timeout`, `--check-timeout` | 120, 60 | seconds per file for the oracle and `jaic -check` |
| `--out DIR` | none | write `reports.jsonl` and `summary.json` |
| `--update-known` | off | rewrite `oracle-known.json` with what was found |
| `--no-sandbox` | off | run upstream unconfined (CI only) |

## Dependencies

`jaic` (the compiler and its IDE facts), `jai-language-server`'s `Session`, `target/jaifmt`, `tools/differential/jaigen.py`, `tools/corpus/upstream_sandbox.py`. Related: [language server typing sessions](lsp-fuzzing.md) checks the same server under random typing.
