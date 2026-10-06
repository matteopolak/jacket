# Module parameters

## What it is

`#module_parameters` lets an importer configure a module. Two lists exist: the first configures one instance per distinct argument set; the second ("program parameters") configures the single instance shared by the whole program.

## How it works

Declaration in the module entry file:

```jai
#module_parameters(LOUD := false, Count: s64 = 1)(TRACE := false);
```

Import with arguments in the same two-list shape:

```jai
G :: #import "Greeter"(LOUD = true, Count = 2)(TRACE = true);
G.greet("bob");   // prints "HELLO bob" twice, then "helper trace=true ..."
```

- Different first-list values create different instances (`Count = 1` and `Count = 2` print one and two greetings). Equal values share an instance.
- Program parameters are shared: in `tests/stdlib/program-module-parameters.jai` a plain `#import "Program_Param"` and `#import "Program_Param"()(VERBOSE = true)` see the same types, and both observe `VERBOSE == true`. Setting a different value after the module already used it is an error (`program_instance`).
- A scalar argument takes the parameter's declared type (`SAMPLES: s32`; `tests/stdlib/typed-module-parameters.jai`). Enum and aggregate arguments carry their type; `.Member` arguments take the parameter's type.
- A parameter without a written type (`FRAMES := 3`) is a constant like `FRAMES :: 3`. Given an untyped number (a literal, or a `N :: 2` constant) it stays untyped, so it passes to any integer or float type its value fits (`tests/stdlib/untyped-module-parameters.jai`). ui_builder's Pixel_Maker relies on this: `MAX_FRAME_IN_FLIGHT := 3` goes to an `NSUInteger` (`u64`) parameter. A typed argument (`FRAMES = cast(u8) 3`, or a `N : s64 : 3` constant) keeps its type. Evidence: the reference how-to on module parameters says parameters become constants, and the how-to on literal types says a `::` constant is used exactly like its literal. Whether an explicitly typed constant (`N : s64 : 3`) also converts when it fits is not settled: neither the corpus nor the reference passes one to a narrower or differently signed integer without a cast, so jaic keeps rejecting it.
- Parameters are ordinary constants inside the module, including its `#load`ed files (`TRACE` was read from a loaded `impl.jai`).
- An unknown argument name is silently ignored (verified); nothing diagnoses it.
- A trailing block, `#module_parameters(...) { ... }`, declares names that defaults may use.

## How to change it

Declaration is `declare_module_parameter` plus the `ModuleParameters` arm of `declare_stmt`; argument evaluation is in `resolve_import` (`eval_const_literal` reports whether a number is untyped: untyped scalars are stored with type `VOID`, typed numbers with their type, and the parameter entity's `untyped_const` flag follows); program-parameter plumbing is `note_program_params`, `apply_program_params` and `program_instance`. All are in `crates/jaic/src/sema/modules.rs`. Per-module state is `Module::params`, `param_entities` and `program_params` (`crates/jaic/src/sema/mod.rs`).

Gotcha: a program-parameter setting patches the declaration's value only when the parameter has no written type; typed parameters wait for the import itself.

## Configuration

None.

## Dependencies

`consteval.rs` evaluates argument expressions in the importer's scope. Regression modules: `tests/stdlib/modules/Program_Param`, `tests/stdlib/modules/Typed_Param` and `tests/stdlib/modules/Untyped_Param`.
