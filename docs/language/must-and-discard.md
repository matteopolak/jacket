# `#must` results and `#discard` parameters

## What it is

`#must` on a result makes discarding it a compile error {#must.1}. `#discard` on a parameter means the callee cannot use it {#must.2} and the caller never evaluates the argument (it is only type-checked) {#must.3}. That is how `Basic.assert` costs nothing when assertions are off {#must.4}.

## How it works

### `#must`

The parser sets `Return::must` per result (`-> int, bool #must` marks the second) {#must.5}. Each time `emit_call` in `sema/calls.rs` emits a call or expands a macro, it records the call's span, name and `#must` flags in `last_call_must`.

The check happens when code is generated, not during type checking. That is deliberate: a call passed as `Code` to a macro that never runs it does not violate `#must` {#must.6}.

- A call statement: `check_stmt` (`sema/stmt.rs`) calls `check_must_used(span, 0)`, so any `#must` result is an error.
- `a := f();`: `check_must_used(span, names)`, so only results beyond the declared names may not be `#must` {#must.7}. `_ := f();` counts as using it {#must.8}.
- Calls through procedure values and operator overloads don't carry `#must`; the flag lives on the declaration {#must.9}.

```
error: the result of 'only' is marked #must and cannot be discarded
```

### `#discard`

- `ParamInfo::discard` mirrors the flag. `discarded_args` (`sema/calls.rs`) finds arguments that go to a `#discard` parameter in any candidate, and `precheck_args_deferring` checks them with `check_expr_no_emit`: types and constants, no IR. `param_value` passes a zero of the parameter type; a macro gets no local.
- Since the argument is still type-checked, `#discard x: $T` infers `T` {#must.10} and type errors are still reported {#must.11}.
- Defaults (`#discard loc := #caller_location`) are not evaluated either {#must.12}.
- Inside the callee the name is a dummy constant in `Compiler::discard_params`; any use fails in `entity_operand` (`sema/expr.rs`) with `'x' is a #discard parameter and cannot be used in the procedure`.

`stdlib/Basic/module.jai` declares `assert` with `#discard` parameters when `ENABLE_ASSERT` is false, so `assert(expensive())` never calls `expensive()`.

## How to change it

- A new place that drops call results (say, assigning a multi-result call to one target) must call `check_must_used` right after checking the call. `last_call_must` identifies the call only by span, so another call emitted in between hides it.
- New call forms must route arguments through `precheck_args_deferring` (or honour `discarded_args`), or the argument is evaluated before the callee is known.
- Tests: `tests/stdlib/must-return-values.jai`, `tests/stdlib/discard-parameters.jai`, and negative programs `tests/corpus/negative/must-*.jai` and `discard-*.jai`.

## Configuration

`Basic`'s `ENABLE_ASSERT` module parameter selects the `#discard` form of `assert`.

## Dependencies

`parser/procedure.rs`, `sema/calls.rs`, `sema/stmt.rs`, `sema/expr.rs`, `sema/procs.rs` (`ParamInfo`), `stdlib/Basic/module.jai`.
