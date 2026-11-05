# Language server typing sessions

A seeded checker that types into documents the way an editor does and asks the language server the same questions before, during and after, to find answers that get lost, change or contradict a fresh start. Run it after touching `jai-language-server`, the IDE parts of `jaic` (`parser/ide.rs`, `ide/`, `sema/ide_sketch.rs`) or incremental analysis.

```sh
cargo test -p jai-language-server --test typing_session        # fast slice, fixed seeds
cargo run --release -p jai-language-server --example typing_fuzz -- \
    --seeds 0..20 --scenarios 6 --diff-every 7 stdlib/Hash/tests/scalars.jai examples/*.jai
fuzz/run.sh lsp_typing 600                                      # libFuzzer
```

Run one process at a time; a session compiles the document many times. A failure prints the file, seed, oracle, the last edits and the text, and exit status is 1. Oracle d failures are minimized by delta debugging.

## How it works

`crates/jai-language-server/examples/support/typing_session.rs` opens real files in a `Session`, records a baseline (diagnostics, semantic tokens, definition and hover at sampled identifiers), then sends incremental edits with rising versions and queries between them. It needs no expected answers, only oracles:

| Oracle | Checks |
| --- | --- |
| a | no request panics or runs past the time budget |
| b | outside the edited region, answers equal the baseline mapped through the edits |
| c | back at the original text, everything equals the baseline |
| d | every Nth version equals a fresh `Session` on the same text |
| e | diagnostic ranges are ordered, in bounds and without duplicates |
| f | the server's text equals the checker's copy |
| g | text that lexes has no literal left out of the analysis |
| h | asking the baseline's requests twice gives the same answers |

Traffic includes typing one character at a time (with auto-closed brackets), backspacing, retyping, pastes, unclosed brackets closed later, undo to the original, and the same edits in a second open document. The libFuzzer target runs oracles a, c, d and e; saved crashes go in `fuzz/regressions/lsp_typing/` and replay in `cargo test`.

## How to change it

- New edit traffic: a `scenario_*` method on `Rig`, picked in `run_scenario`.
- New query: a `Query` variant (so a failing run can replay it) and a field of `Snapshot`.
- Where choices come from is the `Choices` trait: `Seeded` for tests, the fuzzer's bytes for libFuzzer.
- A fixed failure becomes a test in `crates/jai-language-server/tests/fuzz_regressions.rs` or a fixture under `tests/lsp/typing/`.
- Oracle b has exemptions where a difference is legitimate (samples at `#run` and `#string` bodies, usage lints, compile-time errors, the procedure header line, hover text inside polymorphic structs). Add a new one to `sample_identifiers`, `usage_lint`, `near_region` or `in_poly_struct`, and check it with a fuzz run without it. Remaining differences are decided by a fresh session (`fresh_agrees`, `fresh_probe_agrees`).

## Configuration

`Config` in `typing_session.rs`: `scenarios`, `diff_every`, `probes`, `full_probes`, `budget`, `hang`, `outside`, `max_failures`. The example takes `--seeds A..B`, `--scenarios N`, `--diff-every N`, `--stdlib DIR`, `--list FILE`. `TYPING_FUZZ_DUMP=DIR` writes each failing document's text to `DIR`.

## Dependencies

`jai-language-server` (`Session`, `JsonSession`, and the production bounded transport reader) and `jaic`. The libFuzzer target adds `jai-wasm` and `arbitrary`.

## Protocol bytes

The `lsp_json` target feeds every input through the production `transport::BoundedReader`, which bounds framing before the pinned `lsp-server` decoder allocates a body. Complete rejected bodies can be followed by another frame; malformed framing stops that input. UTF-8 inputs also exercise newline-separated messages through `JsonSession` after initialization. Keep this harness on the production reader when transport APIs change; do not recreate framing in the fuzzer. Its body limit is 1 MiB.
