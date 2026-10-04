# jaic regression sweep

## What it is

`tools/jaic-sweep.py` runs the `jaic` binary over sets of Jai programs and reports pass/fail. It is the main
regression gate for compiler and stdlib changes.

## How it works

Each set yields `(id, path, mode, expectation, extra args)`; the tool runs `jaic <mode> <path> <args>` from the file's
directory and compares the exit code (and stdout, for `corpus` cases with a recorded runtime expectation).

| set | source | mode |
|---|---|---|
| `corpus` | `tests/corpus/manifest.json` cases with a `runtime` record | run, exact stdout + exit code |
| `stdlib` | `tests/stdlib/*.jai` | run, exit code 0 |
| `upstream` | `tools/upstream-cases.json` (paths under `corpus/upstream/`) | per case, exit code 0 |
| a path | that file | run |

## How to change it

- New regression program: drop a self-checking `tests/stdlib/<name>.jai` (return non-zero / `assert` on failure).
- An upstream entry point started working: add it to `tools/upstream-cases.json` so it stays working.
- Known failures today: `reflection-initializer`, `getrect-rh-negative-control`.

## Configuration

`--jaic PATH` (default `/Volumes/CodexBuilds/targets/jai-dev/debug/jaic`), `--filter TEXT`, `--verbose`,
`--timeout SECONDS`. The `upstream` set needs `python3 tools/fetch_upstreams.py` first (the corpus is gitignored).

## Dependencies

Python 3 standard library, a built `jaic-cli`.
