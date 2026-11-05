# Measuring language-server requests

`tools/bench/request_latency.py` measures native LSP responses for real source files. It reports request samples, selected positions, process CPU time, peak RSS and first-diagnostic/initialize times; it does not infer semantic correctness from a fast response or a small payload.

## Workloads

The default `comment-edit` scenario inserts a trailing comment, immediately requests one answer, restores the source and waits before the next sample. Requests stay in the registry's fixed order. This tests edit-to-answer behavior; deferred reference and linked-editing requests may be unavailable until a fresh check finishes. An empty result is an availability outcome and cannot establish useful-answer latency.

`warm-unchanged` performs the same initialization and diagnostic warm-up, then queries one unchanged version. Supply one `--binding` declaration snippet per input. The script finds its real source position, converts it to UTF-16 and requires hover, reference and linked ranges to select that identifier in the same document; hover contents must also be nonempty. For example:

```sh
PYTHONPATH=tools JAIC_STDLIB=$PWD/stdlib python3 tools/bench/request_latency.py \
  --server /absolute/path/to/jailsp --formatter /absolute/path/to/jaifmt \
  --scenario warm-unchanged --binding 'result := value' \
  --methods textDocument/references,textDocument/linkedEditingRange \
  --rounds 7 --output /tmp/lsp-warm.json tools/bench/fixtures/lsp-small/main.jai
```

`tools/bench/fixtures/lsp-small/main.jai` is a complete import-free small workload with known function and local references. Keep `examples/basics/structs.jai` as a separately labeled historical/error fixture when its standalone compile is incomplete.

`edit-recheck` inserts a trailing comment, requests a fresh full diagnostic response, then asks the selected method. Its report separates request-only milliseconds from total edit-to-answer time. This tests fresh checked answers; it does not represent immediate request availability while the compile is deferred.

Supply one `--declaration` snippet per input to pin the rename and prepare-rename target instead of using the heuristic enclosing procedure.

Use `--include-payloads` in a separate correctness capture to retain request parameters, document versions and full results. Validate expected names, edit targets, declaration inclusion and completeness against independent tests; the warm range check proves spelling, not complete symbol resolution. Empty diagnostics can be correct. Rename edits are inspected without applying them.

## Comparable runs

Build baseline and candidate with the same profile, features and toolchain. Keep source inputs, stdlib, formatter, working directories and settings identical. Put the matched compiler beside each `jailsp` as `jaic`, since compile-time services may start it. Pin executable hashes and source revisions, including any uncommitted patch.

Run a one-round pilot first, then choose repetitions from its runtime and observed variation. Alternate independent baseline/candidate process runs under an exclusive quiet-machine lease. Retain all samples and record load and OS-cache conditions. Compare medians and maximums along with CPU/RSS and correctness; a few runs do not establish p99 latency. First-diagnostic measurements use fresh processes but do not imply a cold filesystem cache.

`--limits tools/bench/request-latency-limits.json` applies existing historical ceilings to the original workload. Do not rewrite those limits to match a slower result. Shared-runner CI deliberately does not gate wall-clock request latency.

For CPU attribution, use `--profile /absolute/output.json` with one input file; this requires `samply`. Profile separately from uninstrumented timing. Sampling can locate compiler/linter/query stacks but does not directly measure protocol queue residence or snapshot extraction duration.

Untimed operation counters can distinguish repeated filesystem interface work from compilation: assert useful responses and unchanged compile counts, then report calls and distinct paths by method. These are not kernel syscall counts or latency attribution. Preserve the observer's actual method results, including native identity and stamp behavior. A diagnostic CPU sample under shared desktop load must be marked unscored and run without overlapping builds; it cannot substitute for the exclusive quiet timing comparison.

## Index extraction and retention

`reference_bench` checks the selected binding against the live compiler oracle, then measures repeated owned-index exports. For example:

```sh
cargo run --release -p jai-language-server --example reference_bench -- \
  "$PWD/stdlib" tools/bench/fixtures/lsp-small/main.jai 'result := value' 7
```

It reports compilation outcome, diagnostics, source/fact counts, extraction duration and index bytes. A partial compile is labeled and cannot support complete-program claims. The first export follows the oracle query, so it is not cold. One exported index is live per sample; query equality checks and printing occur outside the measured interval. Index accounting excludes source/dependency records and process RSS. `edit_bench` separately reports session record counts, latest discovery-proof bytes and bounded build-probe/advisory storage after each measured edit/check interval. These counts are collected outside that interval and are not process RSS.

Native on-demand freshness replays captured dependencies. On a successful direct compile, `reference_bench` also measures complete-session references. Its second session scenario opens an unrelated overlay before the interval to force an input-memo miss, then asserts identical nonempty answers and unchanged compile count. Setup, correctness checks, accounting and printing are outside each measured interval. Partial direct compiles skip those input-cost measurements. These in-process component samples exclude JSON transport and queueing.

The private cache owner has a manual `native_cache_validation_benchmark` component test so no cache implementation API is exported for measurements. Build it separately before acquiring the exclusive timing lease, then run the built release test binary with `--ignored --nocapture --test-threads=1 native_cache_validation_benchmark`. It checks complete coverage and a nonempty compiler target for the same three primary files before measuring seven exact cache hits. It reports retained input-proof bytes; setup, oracle checks and printing are untimed. This measures compiler input validation without session discovery, reference export or JSON transport. The test is intentionally excluded from ordinary correctness runs and has no wall-clock threshold.

Measure warm request validation and discovery misses separately from reference export: an unchanged request, a fresh input-context miss, and an edit followed by a completed check exercise different work. Keep exact proof enabled on both correctness captures and the candidate measurements; do not infer validation cost by comparing useful answers with unavailable results.

## Extending the harness

Add registered dispatch variants and their independent protocol method fixtures to `SCENARIOS`, and give them real preparatory items or result IDs. Coverage compares dispatch cases rather than parsing wire-name literals, so library method constants remain authoritative in the server. Preserve default notification order and settings when adding scenarios so historical comparisons remain meaningful. New scenarios need explicit freshness and useful-result checks rather than fabricated hierarchy items or unconditional empty-result acceptance.

Configuration is through the script's flags, `JAIC_STDLIB` and `PYTHONPATH=tools`. The harness depends on the Python standard library, `bench.lsp_client`, position helpers and the native server/compiler/formatter; profiling additionally depends on `samply`. Retained reference ownership and pressure behavior are documented in [checked reference snapshots](../compiler/lsp-snapshots.md).

`wire_bench` measures output construction and writing separately from the server. Give it a private JSON array of captured `{ "method": ..., "result": ... }` records for full semantic tokens, document symbols, completion or pull diagnostics. Validate those results independently first. For example:

```sh
cargo run --release -p jai-language-server --example wire_bench -- /absolute/private/capture.json 7
```

For each item it reconstructs the typed result before measurement, then measures typed-to-`Value`, one and two serialization-count walks, and the library's `Message::write` into a counting sink. It checks the writer's bytes against the versioned count encoding outside the interval. It classifies results above the default host output cap rather than timing them as accepted output. The raw capture, typed result, expected frame and response value remain live outside the intervals. Allocation requests, requested bytes and extra peak live bytes use an isolated diagnostic allocator and are baseline-relative logical layout accounting, not process RSS or production peak. Reallocation may temporarily hold both blocks physically without that transient appearing in the logical count. The measurements include instrumentation overhead; compare the same harness and workload, and measure actual server round trips separately. Inputs are capped and processed one item at a time; no response contents or file paths are printed.

Native queue and compile attribution is opt-in: set `JAIC_LSP_TRACE=1` and set `JAIC_LSP_TRACE_FILE` to a new file on a local filesystem. The server refuses an existing destination. Events contain internal ordinals, fixed categories, byte counts and relative monotonic times, not JSON-RPC IDs, file names, paths, parameters or payloads. The bounded recorder keeps the first events and reports later drops; it also marks output incomplete when producers have not quiesced. Drop and late counts are snapshots at export, so incomplete traces cannot prove absence of queueing or discarded work. The trace file is exported after transport completion, and a slow filesystem can delay shutdown; the benchmark supervisor should bound it.

Reader send duration covers both the payload and arrival-token channel sends, including bounded-channel backpressure. Owner-dequeued is recorded when the core consumes the payload; time spent holding a lookahead token is included. Decode-to-dispatch includes that backpressure and queueing; concurrent send-return and dequeue timestamps do not necessarily order as a pure queue interval. Producer quiescence does not imply every decoded frame was dispatched or every output delivered: exit can abandon later input, and write failures are separate events. Compile records identify accepted Analysis objects (including partial/error-bearing contexts), rejected/abandoned transactions or unknown outcomes and calling-thread CPU where available; they exclude worker threads and child processes. Rejected-transaction CPU is not automatically proven wasted or discarded work. WebAssembly and unavailable clock readings are reported as unavailable rather than zero. Compare trace-on and trace-off runs to measure instrumentation overhead.

For a single-process trace, use one input file and a fresh destination:

```sh
JAIC_LSP_TRACE=1 JAIC_LSP_TRACE_FILE=/tmp/lsp-trace.json \
  PYTHONPATH=tools JAIC_STDLIB=$PWD/stdlib python3 tools/bench/request_latency.py \
  --server /absolute/path/to/jailsp --formatter /absolute/path/to/jaifmt \
  --scenario warm-unchanged --binding 'result := value' \
  --methods textDocument/hover --rounds 1 --output /tmp/lsp-trace-result.json \
  tools/bench/fixtures/lsp-small/main.jai
```

Use separate fresh trace files for separate server processes. Change the fixed event categories and bounded record format in `trace.rs`, transport association in `transport.rs`/`main.rs`, and compile outcomes at the accepting cache transaction. The recorder uses `cpu-time` on supported native targets; native transport uses `lsp-server` and bounded crossbeam channels. Keep trace export outside request intervals, and preserve disabled-path checks when adding hooks.

The ignored `native_protocol_fs_work` library test counts filesystem work without timing it:

```sh
cargo test --locked -p jai-language-server --lib native_protocol_fs_work \
  -- --ignored --nocapture --test-threads=1
```

It uses the native program-options callback and a fixed-options control with fresh sessions, the checked-in client capabilities, and the three primary files. It requires repeated full results to agree, validates nonempty reference/rename counts and checks that warm requests do not compile. Output separates discovery retirement, ordered native history replays and other operations, with parent pass ordinals so nested work is not counted twice. Raw paths stay in ephemeral memory; reports contain only categories, distinct-path counts, multiplicities and returned read bytes. These are filesystem API calls on the calling thread, not kernel syscalls. The controlled `JsonSession` sequence does not reproduce native idle work or every preparatory completion/hierarchy/token request in the latency harness. Do not treat its counts as native end-to-end timing. The optional collector and replay scopes compile only in tests and remain disabled outside this explicit tool.
