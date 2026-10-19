# Language server typing sessions

A seeded checker that types into documents the way an editor does and asks the language server the same questions before, during and after, to find answers that get lost, change or contradict a fresh start. Use it after touching `jai-language-server`, the IDE parts of `jaic` (`parser/ide.rs`, `sema/ide*.rs`) or incremental analysis.

## What it is

`crates/jai-language-server/examples/support/typing_session.rs` opens real files in a `Session`, records the baseline (diagnostics, semantic tokens, definition and hover at sampled identifiers), then sends incremental `TextChange`s with rising versions and queries between them. It needs no expected answers, only oracles:

| oracle | checks |
|---|---|
| a | no request panics or runs past the time budget; a watchdog ends a hung process (exit 112) |
| b | with the edited region balanced, definition, hover, tokens and diagnostics outside it equal the baseline mapped through the edits |
| c | back at the original text, everything equals the baseline |
| d | every Nth version equals a fresh `Session` on the same text, unless some open document reports an incomplete lexical edit (the session leaves an edited comment or literal out of its analysis, and so does what loads it; a fresh session lexes the text as it is) |
| e | diagnostic ranges are ordered, inside the document, on character boundaries, without duplicates |
| f | the server's text equals the checker's copy after each change |
| g | text that lexes has no literal left out of the analysis |
| h | asking the baseline's requests twice gives the same answers |

Traffic: typing a statement or declaration one character at a time (auto-closed brackets and quotes included), backspacing, deleting and retyping a token or line, pastes, unclosed brackets, strings and comments closed later, undo to the exact original, whole-document reverts, and the same edits in a second open document that the first loads. Edits stay inside one region of a document (a procedure body, or the end of the file), so the rest has a known answer.

## How to run it

The fast slice, a few fixed seeds over `tests/lsp/typing/` (a program and the file it `#load`s) and `tests/lsp/partial/`:

```sh
cargo test -p jai-language-server --test typing_session
```

At scale, over any files (stdlib tests, examples), release build, one process:

```sh
cargo run --release -p jai-language-server --example typing_fuzz -- \
    --seeds 0..20 --scenarios 6 --diff-every 7 stdlib/Hash/tests/scalars.jai examples/*.jai
# or: --list FILE (one path per line), --stdlib DIR
```

Each failure prints the file, seed, oracle, the last edits and the document text; a difference found by oracle d is then minimised by delta debugging to the shortest history of edits that still shows it. The exit status is 1 on any failure. A file that loads or imports a sibling opens the sibling too. Run one process at a time: a session compiles the document many times.

libFuzzer drives the same checker (oracles a, c, d, e; b is off because it has known cases a fuzzer would only rediscover):

```sh
fuzz/run.sh lsp_typing 600
```

The input is a document, then optionally a NUL and the bytes that choose the edits. Saved crashes go in `fuzz/regressions/lsp_typing/` and replay in `cargo test` (see [testing tools](testing.md)).

## How to change it

- A new kind of edit traffic is a `scenario_*` method on `Rig`, picked in `run_scenario`.
- A new query is a `Query` variant (every request goes through it so a failing run can be replayed) and a field of `Snapshot`.
- Where the choices come from is the `Choices` trait: `Seeded` for the example and test, the fuzzer's bytes for libFuzzer.
- A fixed failure becomes a test in `crates/jai-language-server/tests/fuzz_regressions.rs` or a fixture under `tests/lsp/typing/`.
- Oracle b has exemptions where a difference is expected, each checked by a fuzz run without it: samples at a `#run` and in `#string` bodies (the value a `#run` shows follows from what another body does; the target answers differ), usage lints and compile-time execution errors (they follow from what a body does: a `#run` of a procedure fails where it is written when that procedure stops returning a value), and diagnostics on the procedure header line (a missing return or an unused parameter is reported there). Hover text inside a polymorphic struct is compared only for its presence and range (`in_poly_struct`): it shows the types of whichever instance there is, or the stand-ins of the editor's own sketch. Definitions and tokens there are still compared. A syntax hover ("Source syntax declaration") quotes the whole declaration, which may enclose the edited region, so it may change as long as it keeps its range and quotes the document as it now reads. A new exemption belongs in `sample_identifiers`, `usage_lint`, `near_region` or `in_poly_struct`. The `#if` blocks and the line after a body's closing brace are no longer exempt.
- A use of a name an analysis never checked (a call inside a body that cannot be checked on this host) is answered by checking its callee on demand, like semantic tokens do (`ide_note_callee_at`), so hover and definition do not depend on which request came first. Test `typing_regressions.rs`.

## Configuration

`Config` (in `typing_session.rs`): `scenarios`, `diff_every`, `probes`, `full_probes`, `budget`, `hang`, `outside`, `max_failures`. The example takes `--seeds A..B`, `--scenarios N`, `--diff-every N`, `--stdlib DIR`, `--list FILE`. The environment is the one `jailsp` starts with: the stdlib directory and `find_preload`.

## Dependencies

`jai-language-server` (public `Session` API only) and `jaic`. The libFuzzer target adds `jai-wasm` for the playground environment and `arbitrary` for the byte-driven choices.
