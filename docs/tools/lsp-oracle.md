# Language server oracle checks

Checks what the language server answers against what the compiler knows, on every file `jaic -check` accepts: stdlib tests, `tests/compiler`, `examples/`, the reviewed upstream corpus and `jaigen` programs. No expected answers are needed; accepted code is the oracle.

```sh
cargo build --release -p jaic-cli -p jai-language-server --example lsp_oracle
PYTHONPATH=tools python3 tools/lsp/oracle.py --sets examples,compiler --jobs 2
PYTHONPATH=tools python3 tools/lsp/oracle.py --sets upstream      # sandboxed
PYTHONPATH=tools python3 tools/lsp/oracle.py path/to/file.jai
target/release/examples/lsp_oracle --probe 12:8 path/to/file.jai  # one position by hand
```

CI runs it nightly in four shards (`.github/workflows/lsp-oracle.yml`). Upstream runs are local only.

## How it works

`tools/lsp/oracle.py` picks files, keeps those `jaic -check` accepts, and passes them in chunks to the `lsp_oracle` example (`crates/jai-language-server/examples/lsp_oracle.rs` and `lsp_oracle/`). It opens each file in a `Session` and prints one JSON report per file.

| Check | Fails when |
| --- | --- |
| a | accepted code has an error diagnostic, a request errors or hangs, or a region marked unreachable actually runs |
| b | an identifier the compiler resolved has no hover, no definition, or a definition elsewhere than its declaration |
| c | hover, definition or diagnostics change beyond shifted positions after inserting blank lines or running `jaifmt`, or editing a session into that text differs from opening it fresh |
| d | completion after `value.` misses a member the code uses, or a struct literal misses a field |

- A crashing chunk is rerun file by file. Upstream files run inside the corpus sandbox; cases needing special setup or still pending review are skipped.
- Findings are keyed `path|kind|name` and compared with `tools/lsp/oracle-known.json`. A new key fails the run (exit 1); a known key no longer found is printed so it can be removed.

## How to change it

- **New check:** add a finding kind in `checks.rs` (or `metamorphic.rs` for edit-based ones) and gate it on a letter in `--checks`.
- **False positive:** narrow the check. A real server gap that won't be fixed now: run with `--update-known` and commit the new `oracle-known.json` lines. It only rewrites keys for the files checked in that run.
- Each oracle process compiles each file several times. Keep `--jobs` low on a 16 GiB machine and don't run it next to `cargo test`.

## Configuration

| Flag | Default | Meaning |
| --- | --- | --- |
| `--sets` | `stdlib,compiler,examples,jaigen` | file sets |
| `--jaic`, `--oracle`, `--jaifmt` | release builds | binaries used |
| `--jaigen N` | 20 | jaigen seeds `0..N` |
| `--shard I/N` | `1/1` | every Nth file from the Ith |
| `--jobs`, `--chunk` | 2, 8 | parallel processes, files per process |
| `--probes` | 300 | identifiers sampled per file |
| `--checks` | `abcd` | which checks run |
| `--timeout`, `--check-timeout` | 120, 60 | seconds per file |
| `--out DIR` | none | write `reports.jsonl` and `summary.json` |
| `--update-known` | off | rewrite `oracle-known.json` |
| `--no-sandbox` | off | run upstream unconfined (CI only) |

## Dependencies

`jaic`, `jai-language-server`'s `Session`, `target/jaifmt`, `tools/differential/jaigen.py`, `tools/corpus/upstream_sandbox.py`. See also [typing sessions](lsp-fuzzing.md). `tools/dev/lsp_partial_edits.py <paths>` is a hand-run companion that runs the `partial_edit_coverage` example once per file with a timeout.
