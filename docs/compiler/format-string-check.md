# Format string check

## What it is

A compile-time check that a literal `print`-style format string uses as many arguments as the call passes. `print("% is %\n", name)` and `print("done\n", n)` both draw a compile warning (as in Jai), and the build goes on.

```
warning: incorrect number of arguments supplied to `print`: the format string requires 2 arguments, but 1 argument is given
  --> main.jai:4:5
   |
 4 |     print("% is %\n", name);
   |     ^^^^^^^^^^^^^^^^^^^^^^^
   = help: pass a value for every `%`, or write `\%` for a percent sign
```

## How it works

`check_format_call` (`crates/jaic/src/sema/format_check.rs`) runs in `call_procs` once overload resolution has picked the callee. A callee is print-like when:

- a `string` parameter is directly followed by a variadic `..Any` parameter, and
- its body passes both parameters to one call (`print`, `sprint`, `tprint`, `log`, `log_error`, `assert`'s message and user wrappers). A `greet :: (name: string, extras: ..Any)` that uses them apart is not a format.

The check is skipped when the format argument is not a string literal, when the call spreads an array (`..args`) or names a variadic argument, or when the callee has no body. Directives are counted the way `__format_to_builder` in `stdlib/Basic/Print.jai` reads them (`arguments_used`): `%` and `%0` take the next argument, `%N` takes argument N and a following `%` continues after it, `%00` takes none, `%%` is two arguments in a row, and `\%` (byte 31 once lexed) is a plain percent sign. The arguments needed are one past the highest index used.

Both directions are warnings (`Sema::warn`) at the start of the call, worded like Jai's: too few and too many differ only in the help line. jaic has no `-D warnings`, so a warning never fails a build.

## How to change it

- Keep `arguments_used` in step with `__format_to_builder`, and with `jailint::format_string` (used by the language server's hover, which works without type checking).
- The language server shows the warning through `Analysis::check_warnings` (the compiler's `warnings`, severity Warning, code `check`). The server has no syntactic count check of its own.
- Tests: `format_string_argument_count_is_checked` in `crates/jaic-cli/tests/diagnostics.rs` and the unit test in `format_check.rs`.

## Configuration

None.

## Dependencies

`lexer::lex` (to find the calls in a wrapper's body) and the `string`/`Any` parameter shapes of the bundled `Basic` module. The `format_arg_count` jailint rule that used to report the same mistakes was removed; a `jailint.toml` that still names it is accepted and ignored.
