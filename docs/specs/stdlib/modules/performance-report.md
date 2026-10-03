# Performance_Report


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `get_plugin :: () -> *Plugin`


## Contracts and constraints

This is a metaprogram plugin that requests compiler timing, `#run`, polymorph, and bytecode reports, buffers a textual summary, and writes it when compilation completes. It may emit a diagnostic if the compiler did not provide the report message or another plugin disabled the relevant flags. It depends on compiler plugin message types plus Program_Print and sorting helpers; it reports compiler measurements, not runtime profiling data.
