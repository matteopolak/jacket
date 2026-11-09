# LSP inclusion-graph audit

`tools/lsp/project_graph.py` audits all repository files actually included by a program, including metaprogram-created workspaces. It opens those files in canonical and seeded shuffled orders, querying language features both immediately after each open and after all files are open.

```sh
cargo build --release -p jaic-cli -p jai-language-server --bin jaic --example ide_coverage
PYTHONPATH=tools python3 tools/lsp/project_graph.py \
  --project "$HOME/repos/focus" --entry first.jai \
  --jaic target/release/jaic --out /tmp/focus-graph
PYTHONPATH=tools python3 tools/lsp/project_graph.py --upstream \
  --jaic target/release/jaic --out /tmp/corpus-graphs
```

The nightly [LSP oracle](lsp-oracle.md) workflow runs it over the fixtures in `tools/lsp/fixtures/graph/`: `two-entry` (a `jacket.toml` with two entries, `first.jai` and `second.jai`, sharing one loaded file; audit each with `--entry`) and `metaprogram` (an entry that starts a second compilation in a workspace of its own and loads a helper file).

## How it works

`jaic -deps` discovers the actual loaded source files while executing the entry in the repository sandbox. On a CI runner without a sandbox it runs unconfined, as the [oracle](lsp-oracle.md) does; elsewhere a missing sandbox is an error. The audit includes every dependency inside the project root and lists dependencies outside it separately. A failed dependency check or empty graph fails that case; it is not treated as a successful LSP check.

The existing `ide_coverage` example receives the graph as explicit inputs, preserving stable input IDs while changing the open order. Source bytes are decoded with replacement for invalid UTF-8, matching the compiler and editor document text. Its ordinary audit requests hover and definitions at every identifier and checks semantic coverage and compilation health. Compiler-recorded built-in types and members are distinguished from source declarations; missing navigation to a real declaration still fails. A call to a procedure must also go where a plain scope lookup of its name goes (`wrong_definition` otherwise); other names are not cross-checked. Order observations compare diagnostics, inlay hints, symbols, hover, definitions and member completion; up to 64 identifier positions per file are sampled for response comparisons. Zero is the canonical seed; the default shuffled seeds are `1` and `8675309`.

All children run without network access and with writes restricted to scratch. Reports are copied out by the driver. Each case writes `graph.json`, per-order output and progress records, and `summary.json`. Top-level `summary.json` retains every case, including discovery failures, incomplete runs and order-dependent responses. Progress snapshots contain response fingerprints and compile counts, not source payloads. `--details` additionally retains full local feature responses for diagnosing differences. Source diagnostics remain in the explicit stderr artifacts.

## How to change it

Discovery and cross-order comparisons live in `tools/lsp/project_graph.py`. The `--order-seed` handling in `ide_coverage/runner.rs` shuffles stable inputs; `ide_coverage/order.rs` captures observations. Keep missing observations and failed processes as failures. Do not filter files based on whether their LSP check succeeds.

## Configuration

`--seeds 0,1,8675309` chooses reproducible orders; `--check-timeout` bounds dependency discovery (default 60 seconds); `--timeout` bounds each full order (default 600 seconds). `--audit` selects the `ide_coverage` binary. Project roots and entry context are discovered by the language server from the real workspace and `jacket.toml`; the driver does not silently replace that configuration.

## Dependencies

macOS (`sandbox-exec`) or Linux (`bwrap`), Python, `jaic`, `jai-language-server`'s `ide_coverage` example, and the repository's `tools/bench` sandbox. `--upstream` uses the reviewed entrypoint selection from the [LSP oracle](lsp-oracle.md).
