# Runtime safety

Runtime checks on casts, indexes and pointers, and their opt-outs: `cast,trunc`, `cast,no_check`
and `#no_abc`.

## Map

- [main.jai](main.jai): Calls the section.
- [safety.jai](safety.jai): The checks, and `BREAK_ON_PURPOSE` to trigger one.

## Things to try

- In safety.jai, set `BREAK_ON_PURPOSE` to `.NARROWING_CAST`, `.INDEX_PAST_END` or `.NULL_POINTER` and read the runtime error.
