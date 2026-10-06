# Module parameters

## What it is

`#module_parameters` lets an importer configure a module. The first list configures one instance per distinct argument set. The second, program parameters, configures the single instance the whole program shares.

## How it works

```jai
// Greeter/module.jai
#module_parameters(LOUD := false, Count: s64 = 1)(TRACE := false);

// importer
G :: #import "Greeter"(LOUD = true, Count = 2)(TRACE = true);
G.greet("bob");   // "HELLO bob" twice, then "helper trace=true ..."
```

- Different first-list values create different instances; equal values share one.
- Program parameters are shared: a plain `#import "Program_Param"` and `#import "Program_Param"()(VERBOSE = true)` see the same types and both observe `VERBOSE == true`. Setting a different value after the module has already used it is an error (`program_instance`).
- Inside the module, including its `#load`ed files, parameters are ordinary constants. A trailing block, `#module_parameters(...) { ... }`, declares names the defaults may use.
- An unknown argument name is silently ignored.

Argument types:

- A typed parameter (`SAMPLES: s32`) gives a scalar argument its type. Enum and aggregate arguments carry their own type; `.Member` takes the parameter's type.
- An untyped parameter (`FRAMES := 3`) is a constant like `FRAMES :: 3`. Given an untyped number (a literal or `N :: 2`) it stays untyped, so it can be passed on to any numeric type it fits. ui_builder relies on this: its `MAX_FRAME_IN_FLIGHT := 3` goes to an `NSUInteger` parameter.
- A typed argument (`cast(u8) 3`, or `N : s64 : 3`) keeps its type. Whether such a constant should also convert when it fits is unsettled; no known code needs it, so jaic rejects it.

## How to change it

All in `sema/modules.rs`:

- Declaration: `declare_module_parameter` and the `ModuleParameters` arm of `declare_stmt`.
- Arguments: `resolve_import`. `eval_const_literal` reports whether a number is untyped; untyped scalars are stored with type `VOID`, and the parameter entity's `untyped_const` follows.
- Program parameters: `note_program_params`, `apply_program_params`, `program_instance`.

Per-module state is `Module::params`, `param_entities` and `program_params` in `sema/mod.rs`.

Gotcha: a program-parameter setting patches the declaration's value only when the parameter has no written type; typed parameters wait for the import itself.

Tests: `tests/stdlib/program-module-parameters.jai`, `typed-module-parameters.jai`, `untyped-module-parameters.jai`, with modules under `tests/stdlib/modules/` (`Program_Param`, `Typed_Param`, `Untyped_Param`).

## Dependencies

`sema/consteval.rs` evaluates arguments in the importer's scope.
