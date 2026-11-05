# LSP verification suites

LSP verification combines protocol fixtures, compiler differential checks, full repository oracle runs, and real client exercises. These checks cover different contracts; none is a claim of complete external protocol certification.

## Required existing suites

Run the full `jai-language-server` Rust suite and `ide_reference_index` compiler differential test. The protocol tests include malformed framing, request IDs, lifecycle, cancellation, UTF-16 positions, local URI rules, and serialization against pinned `lsp-types` schemas. The reference index test compares all queried positions with the live compiler, including overloaded, imported, local, member, enum, generic, Unicode, and partial-code cases.

The existing [repository oracle](lsp-oracle.md) checks accepted programs across all four default sets: `stdlib`, `compiler`, `examples`, and `jaigen`. Its manual `lsp-oracle` workflow runs four shards. A short local fixture run does not replace those sets. Report checked files, skipped inputs, known findings, and new failures separately; do not refresh the known-findings file to make a changed implementation pass.

The [typing-session suite](lsp-fuzzing.md) checks incremental answers, undo, fresh-session equivalence, and request budgets. Run every required case without reducing probes or relaxing timeouts. Browser bridge checks and native editor smoke tests exercise their actual adapters; unavailable GPU checks remain explicit skips.

Run the existing VS Code extension E2E suite against the final server and real formatter. It exercises the pinned `vscode-languageclient` through the actual extension host; the compiler CI `vscode-extension` job already runs the entire suite. The Neovim plugin's existing headless integration tests are another actual-client gate. Preserve full case identities and outcomes; these integration suites supplement protocol fixtures rather than establish protocol-wide certification. Record any launcher recovery separately from a clean first launch.

## External suite inventory

Microsoft's protocol repository defines the specification. The `vscode-languageserver-node` repository supplies SDK component and client/server integration tests; those tests are tied to its implementations and fixtures. The inspected repositories do not provide a complete ready-made certification suite for arbitrary servers.

`pytest-lsp` and `lsp-test` provide test-client frameworks rather than a complete portable suite of expected language-server answers. Using a framework does not itself establish conformance. Standard-schema decoding is an independent wire oracle, while this repository's compiler oracle establishes Jai semantics. Both remain necessary.

Primary references: [protocol specification](https://github.com/microsoft/language-server-protocol), [Microsoft Node SDK](https://github.com/microsoft/vscode-languageserver-node), [pytest-lsp](https://pypi.org/project/pytest-lsp/), and [lsp-test](https://github.com/lukel97/lsp-test).

## Configuration and changes

Before release, use the same final source revision for local receipts and all required CI jobs, including a manual oracle dispatch when necessary. Preserve failure evidence and exact tool/input identities alongside the report. See the oracle and typing docs for flags, process limits, fixtures, and extension points.

Reviewed upstream corpus programs may only run through the existing `tools/bench` sandbox. Their scale tests are separate from the four owned-source oracle sets. Do not execute upstream programs directly merely because a test binary can locate them.

The suites depend on the server, compiler, formatter, test-support filesystem fixtures, and the repository's Python oracle tools. Standard wire tests also depend on the pinned `lsp-types` crate.

## Upstream scale confinement

The registered Focus regression delegates its private Rust worker to `tools/bench/sandbox.py` through `tools/lsp/scale_sandbox.py`. It preserves the existing assertions and optional 120-second scale budget; a separate 180-second supervisor terminates a stuck child. There is one attempt, no unconfined fallback, and no network access. Reads cover the worker build, stdlib, pinned upstream project, native libraries and enclosing config files; writes stay in a disposable scratch directory. Generated scale cases use the ordinary harness.

CI installs bubblewrap and enables its unprivileged namespaces before the scale step. Local runs without fetched sources retain the existing explicit skip; enforced runs require the corpus. An unavailable sandbox is a failure whenever the corpus case runs. The registered case always delegates to the bench launcher, which starts the private `lsp_scale_worker` Cargo example with canonical source/stdlib arguments. Build it with `cargo build -p jai-language-server --example lsp_scale_worker` before a focused scale run. Legacy selected-input markers are rejected and never authorize an in-process child. The worker retains the original measured interval and 120-second assertion. This is the existing trusted-launcher boundary: invoke this worker only through `tools/lsp/scale_sandbox.py`; direct manual execution is not cryptographically prevented or claimed safe. Only the registered parent retains the optional absent-corpus skip. To change the wrapper, run `PYTHONPATH=tools python3 -m unittest tools/lsp/test_scale_sandbox.py` and the existing Rust scale case; do not substitute a smaller fixture for it.

The manually dispatched `native LSP transport` workflow runs the focused Rust library and protocol/framing/stdio suites on Windows x64 and ARM64 in debug mode. The regular compiler test matrix covers Unix hosts. This Windows job reuses the pinned toolchain setup with LLVM installation disabled: the frontend-only dependency graph does not link the LLVM backend. It uses Cargo's standard runner because the pinned nextest downloader has no Windows artifact, with one test thread because interruption regressions share process-global test state. Dispatch this workflow on the same final commit as the relevant compiler, browser and actual-client checks; a local macOS result does not establish Windows behavior.

Before the standalone workflow is available on the default branch, dispatch the existing `compiler checks` workflow with `jobs=lsp-transport` and `tier=fast` against the final feature ref. Its manually gated job calls the same local reusable Windows workflow from that ref. The selector excludes this job from empty/default selections, and no full compiler matrix runs for that explicit selection. Cancellation belongs to the caller so the reusable workflow cannot cancel its own parent. Verify the dispatched run's commit SHA before accepting its results.

Native fixtures use owned temporary directories and absolute document URIs. Construct expected paths and counting-filesystem watcher keys with `test_support::lsp::native_identity` before the measured callback; Windows compiler identities use drive-qualified, slash-separated paths, while `std::fs::canonicalize` may return a verbatim path. Keep the actual filesystem operation delegated unchanged so path spelling cannot silently disable interruption or freshness controls.
