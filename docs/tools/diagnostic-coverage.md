# Diagnostic coverage gate

## What it is

`tools/ci/check_diagnostics_covered.py` fails CI when the compiler gains an error or warning that no test expects. It runs in the `test` job next to the other source checks and has its own unit tests (`tools/ci/test_check_diagnostics_covered.py`).

## How it works

Diagnostics are message templates at call sites, so the script reads the source rather than a registry:

1. It scans `crates/jaic/src` outside test code (test files and `#[cfg(test)]` modules) for `err(span, ..)`, `self.err(..)`/`self.error(..)` (lexer and parser) and `Diagnostic::error`/`Diagnostic::warning`, takes the first string-literal or `format!` argument, and turns each `{...}` placeholder into a wildcard.
2. It builds the test corpus: every tracked text file under `tests/`, `crates/*/tests/`, test modules in `crates/*/src` and `tools/test_*` files (so negative corpus `check` strings, rejection programs' expected text and Rust test assertions all count), with Rust string escapes undone.
3. A template is tested when its first 14 literal characters (placeholders match any text) appear in order on one line of the corpus: tests usually quote the start of a message, not all of it. The match is approximate: two messages that start alike share a test, and a test that only quotes the end of a message does not count. A template with less than six literal characters never matches and stays on the list.

The untested ones are in `tools/ci/untested_diagnostics.txt`. The list only shrinks: a new untested template fails the check, and so does a listed one that is now tested or deleted. `--update` rewrites the list; review its diff, since a new line is a diagnostic added without a test.

A diagnostic built from a message in a variable (`err(span, msg)`) is not seen; keep messages inline at the call.

## How to change it

- Add a test for a listed diagnostic: a `tests/corpus/negative/<name>.jai` case with a `kind: negative` manifest entry whose `negative.check` is the text (see [testing](testing.md)), then run `--update` and commit the shorter list.
- Rename a message: update the test that expects it and run `--update`.
- Other call shapes (a new constructor for diagnostics): extend `CALL` in the script and its test.
- `--list` prints the untested templates.

## Dependencies

`git ls-files` (the corpus is the tracked files) and Python 3 only.
