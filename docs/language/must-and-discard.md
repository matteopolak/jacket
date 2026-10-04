# `#must` results and `#discard` parameters

## What it is

Two procedure-level contracts. `#must` on a result makes discarding it a compile error. `#discard` on a parameter makes the
callee unable to use it and the caller never evaluate the argument (the argument is only typechecked), which is how
`Basic.assert` costs nothing when assertions are off.

## How it works

`#must`:

- The parser sets `Return::must` for each result (`-> int, bool #must` marks the second one).
- `Compiler::emit_call` (`sema/calls.rs`) records the call's span, procedure name and its `#must` flags in `last_call_must`
  every time it emits a call or expands a macro, so the check is made when code is generated, not when types are
  checked (CHANGELOG: it is legal to violate `#must` temporarily, for example in a call that is only passed as `Code` to a
  macro that never runs it).
- `check_stmt` (`sema/stmt.rs`) clears the record, checks a call statement, and calls `check_must_used(span, 0)`: any `#must` result
  is an error. A declaration `a := f();` calls `check_must_used(span, names)` so results beyond the declared names
  must not be `#must`. `_ := f();` counts as using the result.
- Calls through procedure values and operator overloads do not carry `#must` (the flag lives on the declaration).

```
error: the result of 'only' is marked #must and cannot be discarded
```

`#discard`:

- `ParamInfo::discard` mirrors the parameter flag. `Compiler::discarded_args` (`sema/calls.rs`) finds the arguments of a call
  that go to a `#discard` parameter in any candidate, and `precheck_args_deferring` checks them with `check_expr_no_emit`
  (types and constants only; no IR). `param_value` passes a zero of the parameter type; for a macro, no local is created.
- Because the argument is still typechecked, polymorphic parameters (`#discard x: $T`) infer their type from it, and
  a type error is still reported.
- The parameter is bound as a dummy constant recorded in `Compiler::discard_params`; `entity_operand` (`sema/expr.rs`)
  reports `'x' is a #discard parameter and cannot be used in the procedure` on any use.
- Default values of discarded parameters (`#discard loc := #caller_location`) are not evaluated either.
- `stdlib/Basic/module.jai` defines `assert` with `#discard` parameters when `ENABLE_ASSERT` is false, so
  `assert(expensive())` never calls `expensive()` then.

## How to change it

- A new place that drops call results (for example assignment to a single target of a multi-result call) should call
  `check_must_used` right after checking the call expression, before another call is emitted.
- Argument handling for new call forms must go through `precheck_args_deferring` (or pass `discarded_args`), or the
  argument is evaluated before the callee is known.
- Gotcha: `last_call_must` only identifies a call by its span, so call `check_must_used` immediately after the call is checked.
- Tests: `tests/stdlib/must-return-values.jai`, `tests/stdlib/discard-parameters.jai`; negative programs
  `tests/corpus/negative/must-*.jai` and `discard-*.jai` (run with `python3 tools/jaic-sweep.py negative`).

## Configuration

None. `Basic`'s `ENABLE_ASSERT` module parameter selects the `#discard` form of `assert`.

## Dependencies

`parser/procedure.rs` (flags), `sema/calls.rs`, `sema/stmt.rs`, `sema/expr.rs`, `sema/procs.rs` (`ParamInfo`, parameter binding),
`stdlib/Basic/module.jai`.
