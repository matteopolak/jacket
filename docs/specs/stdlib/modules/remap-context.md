# Remap_Context


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Remap_Context :: () #expand`
- `remap_context :: () -> (result: #Context)`
- `remap_context :: (foreign_context: *void, foreign_type: *Type_Info_Struct) -> (result: #Context)`


## Contracts and constraints

`remap_context()` returns a copy of the active context when its runtime type is already Jai’s `#Context`; otherwise it delegates using the active context pointer and its type metadata. `remap_context(foreign_context, foreign_type)` returns a default-initialized `#Context`, then copies fields from the foreign structure that have a same-named destination member with exactly the same runtime byte size. Missing fields and same-name fields with different sizes are skipped, leaving the destination field at its default value; this is a tolerant partial mapping, not a validation failure. The foreign type must be non-null and must have a first member named `base` whose type is a struct; malformed metadata is logged and entries may be skipped. The returned value owns its copied fields by value, but pointers within copied fields remain shallow copies and retain their original referents/lifetimes. `context_info` is set to Jai’s own context type. Mapping metadata is cached by foreign type pointer; callers must keep type metadata stable. `Remap_Context()` expands to a context push in the caller's lexical scope: the remapped context is active for the remainder of that scope, and the prior context is restored when that scope exits, including an early exit. It is not scoped only to the expansion's internal block. `VERBOSE` defaults true and controls diagnostic messages; the optional stack-trace adjustment is build-option dependent.

## Module parameters

```jai
#module_parameters (VERBOSE := true)();
```
