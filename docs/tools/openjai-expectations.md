# open-jai expectation harness

## What it is

`tools/openjai-tests.py` runs the expectations declared in open-jai's test suite
(`corpus/upstream/withlang-dev--open-jai/test/**/*_tests.jai`) against jaic. It is an output-level conformance
check: many entries name an example program and its exact expected stdout.

## How it works

The open-jai tests are `@TestProcedure` procedures calling `expect_*` helpers that open-jai's own runner
implements. The script extracts those calls textually and maps them:

| Call | jaic | Check |
| --- | --- | --- |
| `expect_program_output[_contains]` | `run` | stdout equals / contains |
| `expect_compile_output[_contains]` | `check` | stdout equals / contains |
| `expect_compile_success` | `check` | exit 0 |
| `expect_compile_failure` | `check` | fails, message contained in output |

Commands run from the open-jai root (paths in the tests are relative to it). Trailing spaces are trimmed per line
before comparing. `expect_compiler_command_*`, example annotations and output-file checks are skipped.

Expected failures, so the suite is not a gate:

- open-jai is a separate dialect. `build.jai` and `src/` use open-jai's own `#compiler` intrinsics
  (`compiler_arg_count`, a `run_command` builtin) and its own `modules/`. Tests under `test/examples/modules`,
  `test/examples/meta` (its `Code_Node.subexpressions` API) and most of `test/examples/workspace` target those.
- Examples needing Windows, x64 SIMD `#asm`, native libraries, or an outdated API (`random_seed` returning a
  value) fail the same way they do in the Way_to_Jai survey.

The useful signal is in `examples/**` expectations, which are standard Jai programs.

## How to change it

Add a mapping to `HANDLED` for a new `expect_*` kind. To promote a passing case to a regression test, add it to
`tools/upstream-cases.json` (see [jaic-sweep](jaic-sweep.md)).

## Configuration

```bash
python3 tools/openjai-tests.py --filter examples/ -v
```

`--jaic PATH` picks the binary (default: the dev build), `-j N` sets parallelism.

## Dependencies

The open-jai checkout in `corpus/upstream/`, and a built `jaic`.
