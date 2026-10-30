# Diagnostic coverage gate

## What it is

`tools/ci/check_diagnostics_covered.py` fails CI when the compiler gains a diagnostic code that no test names. It runs in the `test` job next to the other source checks and has its own unit tests (`tools/ci/test_check_diagnostics_covered.py`).

## How it works

Every diagnostic has a code, defined once in `crates/jaic/src/diagnostics/catalog.rs` (see [diagnostics](../compiler/diagnostics.md)). The script reads that table and looks for each code in the test corpus: every tracked text file under `tests/`, `crates/*/tests/`, the standard library's own `tests/` folders (their `.stderr` goldens), test modules in `crates/*/src` and `tools/test_*` files.

A code is tested when a test names it as:

- `[name]`, the header of a rendered diagnostic (`error[type-mismatch]`), in a golden file or an assertion;
- `"name"`, a JSON field: the `code` of a `tests/corpus/manifest.json` negative case, or a language-server diagnostic code;
- `Code::Variant`, a Rust test that builds or matches the code.

The message text no longer counts: wording can change freely, the code is what a test pins down.

The untested codes are in `tools/ci/untested_codes.txt`. The list only shrinks: a new untested code fails the check, and so does a listed one that is now tested or deleted. `--update` rewrites the list; review its diff, since a new line is a code added without a test.

## How to change it

- Cover a listed code: add a `tests/corpus/negative/<name>.jai` case with a `kind: negative` manifest entry whose `negative` object has the expected `check` text and the `code` (the sweep, `tools/differential/jaic_sweep.py`, asserts both), or a crate test that expects `error[<code>]`. Then run `--update` and commit the shorter list.
- Rename or retire a code: see [diagnostics](../compiler/diagnostics.md); a renamed code moves to `RETIRED` and its tests change with it.
- `--list` prints the untested codes.

## Dependencies

`git ls-files` (the corpus is the tracked files) and Python 3 only.
