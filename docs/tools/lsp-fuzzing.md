# Language server typing sessions

A seeded checker that types into documents the way an editor does and asks the language server the same questions before, during and after, to find answers that get lost, change or contradict a fresh start. Use it after touching `jai-language-server`, the IDE parts of `jaic` (`parser/ide.rs`, `sema/ide*.rs`) or incremental analysis.

## What it is

`crates/jai-language-server/examples/support/typing_session.rs` opens real files in a `Session`, records the baseline (diagnostics, semantic tokens, definition and hover at sampled identifiers), then sends incremental `TextChange`s with rising versions and queries between them. It needs no expected answers, only oracles:

| oracle | checks |
|---|---|
| a | no request panics or runs past the time budget; a watchdog ends a hung process (exit 112) |
| b | with the edited region balanced, definition, hover, tokens and diagnostics outside it equal the baseline mapped through the edits |
| c | back at the original text, everything equals the baseline |
| d | every Nth version equals a fresh `Session` on the same text |
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
- Oracle b has exemptions where a difference is expected: positions inside `#string` bodies and `#if` blocks (which target answers differs), usage lints and compile-time execution errors (they follow from what a body does), and diagnostics on the procedure header and closing line. A new exemption belongs in `sample_identifiers`, `usage_lint` or `near_region`.
- Known open finding: `stdlib/Input/tests/windows-device-decoding.jai` still fails oracle b on nearly every seed, now on a different cause. On a host without the Windows procedures, bodies that call them fail to check, and which names inside such a body keep a recorded semantic fact depends on how far the checker got. A module alias (`W`, `Basic.assert`) is now answered from the scope when no fact was recorded (`ide_module_name_at`, `crates/jaic/src/sema/ide/definition.rs`; test `module_alias_uses.rs`), but a plain name of a module-level procedure (`unregister(..)` in a later body) still falls back to the syntax answer after an edit elsewhere, whose hover carries the "Source syntax declaration" note.

## Configuration

`Config` (in `typing_session.rs`): `scenarios`, `diff_every`, `probes`, `full_probes`, `budget`, `hang`, `outside`, `max_failures`. The example takes `--seeds A..B`, `--scenarios N`, `--diff-every N`, `--stdlib DIR`, `--list FILE`. The environment is the one `jailsp` starts with: the stdlib directory and `find_preload`.

## Dependencies

`jai-language-server` (public `Session` API only) and `jaic`. The libFuzzer target adds `jai-wasm` for the playground environment and `arbitrary` for the byte-driven choices.
