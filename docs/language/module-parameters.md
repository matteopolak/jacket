# Module parameters

`#module_parameters` lets an importer configure a module. The first list configures one instance per distinct argument set; the second (program parameters) configures the single instance the whole program shares. An import passes arguments in the same two-list shape {#modparam.1}.

```jai
// Greeter/module.jai
#module_parameters(LOUD := false, Count: s64 = 1)(TRACE := false);

// importer
G :: #import "Greeter"(LOUD = true, Count = 2)(TRACE = true);
```

## How it works

- Different first-list values create different instances {#modparam.2}; equal values share one {#modparam.3}.
- A plain `#import "M"` after `#import "M"(FLAG = true)` is a second instance with the defaults. It joins an existing instance only when that instance's first-list arguments are all program parameters {#modparam.13}.
- Program parameters are shared by every import: a plain `#import "Program_Param"` and `#import "Program_Param"()(VERBOSE = true)` see the same types and both observe `VERBOSE == true` {#modparam.4}. Setting a different value after the module used it is an error {#modparam.5}.
- Inside the module, including its `#load`ed files, parameters are ordinary constants {#modparam.10}. A trailing block, `#module_parameters(...) { ... }`, declares names the defaults may use {#modparam.12}.
- An unknown argument name (``argument `X` is not a parameter of this module``) or a literal of the wrong kind (a number for a `bool` flag) is an error {#modparam.11}.
- Checking an `#if` that compares an enum or a number doesn't search imported modules for operator overloads, so it never loads a module before a later `#import "Basic"()(MEMORY_DEBUGGER = true)` inside that `#if` has set the program parameters {#modparam.14}.

## Types

- A typed parameter (`SAMPLES: s32`) gives a scalar argument its type {#modparam.6}. Enum and aggregate arguments carry their own type; `.Member` takes the parameter's type {#modparam.7}.
- An untyped parameter (`FRAMES := 3`) is a constant like `FRAMES :: 3`. Given an untyped number it stays untyped, so it can pass to any numeric type it fits {#modparam.8}.
- A typed argument (`cast(u8) 3`, or `N : s64 : 3`) keeps its type and isn't converted {#modparam.9}; jaic rejects the mismatch.

## How to change it

All in `sema/modules.rs`: `declare_module_parameter`, `resolve_import`, `note_program_params`, `apply_program_params`, `program_instance`. State is `Module::params`, `param_entities` and `program_params`. A program-parameter setting patches the declaration only when the parameter has no written type. Tests: `tests/compiler/regressions/*module-parameters.jai`.
