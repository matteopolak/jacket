# Module parameters

## What it is

`#module_parameters` lets an importer configure a module. The first list configures one instance per distinct argument set; the second (program parameters) configures the single instance the whole program shares.

## How it works

```jai
// Greeter/module.jai
#module_parameters(LOUD := false, Count: s64 = 1)(TRACE := false);

// importer
G :: #import "Greeter"(LOUD = true, Count = 2)(TRACE = true);
```

- Different first-list values create different instances; equal values share one. A plain `#import "M"` after `#import "M"(FLAG = true)` is a separate default instance.
- Program parameters are shared by every import; setting a different value after the module used it is an error (`program_instance`).
- Inside the module, parameters are ordinary constants. A trailing block declares names the defaults may use.
- An unknown argument name or a literal of the wrong kind is an error.
- A typed parameter (`SAMPLES: s32`) gives a scalar argument its type; `.Member` takes the parameter's type. An untyped parameter (`FRAMES := 3`) given an untyped number stays untyped, so it can pass on to any numeric type it fits. A typed argument (`cast(u8) 3`) keeps its type and is not converted; jaic rejects the mismatch because no known code needs the other behaviour.

## How to change it

All in `sema/modules.rs`: `declare_module_parameter`, `resolve_import` (untyped scalars are stored with type `VOID`; see `eval_const_literal`), `note_program_params`, `apply_program_params`, `program_instance`. State is `Module::params`, `param_entities`, `program_params` in `sema/mod.rs`. A program-parameter setting patches the declaration only when the parameter has no written type. Tests: `tests/stdlib/program-module-parameters.jai`, `typed-module-parameters.jai`, `untyped-module-parameters.jai`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- An import passes arguments in the same two-list shape {#modparam.1}:
- Different first-list values create different instances {#modparam.2}; equal values share one {#modparam.3}.
- Program parameters are shared: a plain `#import "Program_Param"` and `#import "Program_Param"()(VERBOSE = true)` see the same types and both observe `VERBOSE == true` {#modparam.4}. Setting a different value after the module has already used it is an error (`program_instance`) {#modparam.5}.
- Inside the module, including its `#load`ed files, parameters are ordinary constants {#modparam.10}. A trailing block, `#module_parameters(...) { ... }`, declares names the defaults may use {#modparam.12}.
- An argument name the module does not declare is an error (``argument `X` is not a parameter of this module``), and so is a literal of the wrong kind for a bool, string, integer or float parameter (a number for a `bool` flag) {#modparam.11}. A plain `#import "M"` after `#import "M"(FLAG = true)` is a second instance with the defaults, not the configured one; it joins an existing instance only when that instance's first-list arguments are all program parameters {#modparam.13}.
- A typed parameter (`SAMPLES: s32`) gives a scalar argument its type {#modparam.6}. Enum and aggregate arguments carry their own type; `.Member` takes the parameter's type {#modparam.7}.
- An untyped parameter (`FRAMES := 3`) is a constant like `FRAMES :: 3`. Given an untyped number (a literal or `N :: 2`) it stays untyped, so it can be passed on to any numeric type it fits {#modparam.8}. ui_builder relies on this: its `MAX_FRAME_IN_FLIGHT := 3` goes to an `NSUInteger` parameter.
- A typed argument (`cast(u8) 3`, or `N : s64 : 3`) keeps its type {#modparam.9}. Whether such a constant should also convert when it fits is unsettled; no known code needs it, so jaic rejects it.
