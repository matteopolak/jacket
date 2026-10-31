# Diagnostic coverage gate

`tools/ci/check_diagnostics_covered.py` fails CI when the compiler gains a diagnostic code that no test names. It runs in the `test` job; unit tests are in `tools/ci/test_check_diagnostics_covered.py`.

## How it works

Every diagnostic code is defined once in `crates/jaic-syntax/src/diagnostics/catalog.rs` ([diagnostics](../compiler/diagnostics.md)). The script looks for each code in the tracked text files under `tests/`, `crates/*/tests/`, the stdlib's `tests/` folders, test modules in `crates/*/src` and `tools/test_*`. A code counts as tested when a test names it as:

- `[name]`, a rendered header (`error[type-mismatch]`) in a golden or assertion;
- `"name"`, a JSON field such as the `code` of a negative manifest case;
- `Code::Variant` in a Rust test.

Message wording doesn't count; the code is what a test pins down.

Untested codes are listed in `tools/ci/untested_codes.txt`. The list only shrinks: a new untested code fails, and so does a listed one that is now tested or deleted. `--update` rewrites it (review the diff: a new line is a code added without a test); `--list` prints it.

## How to change it

- Cover a listed code: add `tests/corpus/negative/<name>.jai` with a `kind: negative` manifest entry (expected `check` text and `code`; the sweep asserts both), or a crate test expecting `error[<code>]`. Then `--update`.
- Rename or retire a code: a renamed code moves to `RETIRED` and its tests change with it.
