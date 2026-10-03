# Module parameters

## What it is

Module parameters configure independently compiled module instances. The graph binds private scalar, float, UTF-8 string, source-nominal enum, and type values; program parameters apply one configuration to every instance of the same module source.

## How it works


The second list sets program parameters once, before any other import of that entry file. Deferred requests reserve their source order, so a later import cannot initialize defaults while an earlier settings import waits for a constant. Fully bound settings include defaults and are reused by later instances.

`#module_parameters` belongs in the module entry file. Its declaration block creates real module-private declarations before defaults bind. Binding retries parameter defaults that depend on another parameter declared later. Persistent discovery resumes incomplete parameter lists without republishing completed parameters or changing their IDs. Required values, duplicate names, type mismatches, and unresolved defaults produce located diagnostics.

Modern Simp and Vk-Engine source uses enum defaults in these blocks:

```jai
#module_parameters(Import_Mode := ImportMode.Foreign) {
    ImportMode :: enum u8 { Foreign; Main; }
}
#if Import_Mode == .Foreign { #load "foreign.jai"; }
else { #load "main.jai"; }
```

An importing file can supply `(Import_Mode=.Main)`. `ParameterValue::ContextualMember` represents this unresolved request only; binding resolves it before a completed graph exposes parameters. `ParameterValue::Enumeration(EnumParameter)` retains the defining `DeclarationId` and checked integer representation. Different enum declarations remain incompatible even when their bits match. Semantic resolution materializes the enum's `TypeId` through its own nominal registry using that declaration identity; the graph never invents a second runtime type registry.

Weak decimal float arguments retain an immutable, span-free exact expression key. They round when the parameter supplies its precision, so a named decimal passed to a `float64` parameter does not round through float32 first. Inferred float parameters supply their default precision; explicit bit-pattern float arguments retain raw width and payload identity. Deferred integer or guard errors inside a named weak float expression retain the defining file through aliases and module coercion. A decimal range error caused by the selected float width still points to the parameter's materializing site. Diagnostic origins do not change request identity.

Each module instance has separate declaration identities, including block enums. A program enum setting is rebound to a later instance only when both enum declarations belong to instances of the same module entry and refer to the exact same source location. This shares the configured member while retaining each instance's nominal identity.

Enum comparisons bind contextual members against the other operand and validate nominal identity. Comparisons nested in boolean or conditional expressions are lowered to boolean constants before the pure evaluator runs. Selected branches register real declarations and source dependencies; loaded files keep their own scopes. `enum_flags` masks support `&`, `|`, `^`, and `~` without losing the declared representation or source nominal identity. A known enum operand supplies context for `.MEMBER` and `~.MEMBER` masks. Equality and inequality permit weak integer zero on either side of an `enum_flags` value; strong integer zero, nonzero integers, ordinary enums, and other nominal enum types are rejected. Standalone contextual complemented import arguments still require a header-context bridge.

`ModuleGraph::load_with_target(path, options, provider, target)` accepts an explicit `jai_types::BuildTarget`; `target()` exposes those facts. `OS`/`BUILD_OS` and `CPU`/`BUILD_CPU` use the source-defined `Operating_System_Tag` and `CPU_Tag` declarations in the defining file or designated Preload scope. A caller's ordinary declaration shadows these target names. Preload's own entry file remains usable before the root module publishes its file list; an unavailable source scope stays pending. The target's source tag selects an actual enum member, including its explicitly assigned value; there is no host or numeric-ordinal fallback. Driver callers supply the target produced by native target configuration.

Type parameters bind builtin types, source record/enum/distinct-alias declarations, transparent aliases, and structural pointer, array, and procedure types. A procedure type with a single `void` result normalizes to an empty result list before request identity is computed. `ModuleType` keys contain typed structure and real `DeclarationId` values. They carry no independently allocated `TypeId`. Semantic resolution materializes every bound type parameter in the existing `TypeRegistry` before compiling bodies; parameter annotations and value lookups use that same identity. A later parameter may depend on a type parameter, for example `#module_parameters(T: Type = int, Count: T = 3)`; scalar values undergo checked coercion to the selected type. Typed source constants also use their defining instance's bound type: `count:T:2` checks the actual width, while `chosen:Kind:.B` preserves the source enum identity. Before value-dependent aliases are defined, semantic preparation publishes ready bound module type annotation facts from the canonical parameter map, including transparent source aliases.

Basic's interface header `REPLACEMENT_INTERFACE: $I/interface Memory_Debugger_Interface = Memory_Debugger_Interface` binds both the named parameter and its `$I` type variable. Named record replacements must supply each explicitly typed field or procedure with the required type/signature. Procedure checks include parameter and result types, convention, context, and variadic shape; extra members are allowed. Type equality in graph conditionals compares the bound structural/source identity, so sentinel defaults can choose source dependencies. Program type settings rebind declarations only when they originate at the exact same source location in a later instance of the same module entry.

Nominal restrictions use a separate proof. For example, `#module_parameters(BaseType:Type=int, R:$I/BaseType=BaseType)` binds the supplied `R` type only after semantic preparation proves identity or declared `using` ancestry to `BaseType`. It retains the full supplied type rather than substituting the ancestor type. Structural field matches and conversion-only fields cannot establish nominal ancestry. Generic record arguments use the same normalized specialization identities as ordinary compilation; modifier-dependent ancestry stays pending until committed source outcomes are available.

Local annotations such as `item:$I` use lexical bindings first, then procedure specialization bindings, then the defining module's canonical parameter map. A module value parameter cannot masquerade as a type variable, and an unmaterialized type parameter retains a readiness diagnostic. This lets program parameter defaults keep separate nominal identities across module instances while using the same source procedure body.

Inserted source dependencies read effective captured scalar, type, and enum values before the quote's graph namespace fallback. A procedure specialization's actual bound arguments still take precedence. Captured type values contain source `ModuleType` keys; preparation materializes them through the existing registry and specialization owner into a map keyed by the genuine expansion file and captured name. Both type resolvers and body lookup require that canonical map. Missing readiness remains a diagnostic rather than creating a type in a second registry. Own inserted declarations shadow captured names. [Module source origins](module-source-origins.md) preserves the destination and capture environment for effect replay.

Generic record applications and inherited interface fields require the retained [semantic parameter discovery](module-parameter-discovery.md) path. It returns canonical template/source identities and normalized formal arguments, then rematerializes them in the existing final registry. Low-level graph-only loading still reports `Pending` for this work. Aggregate-valued parameters, inline/generated nominal identities, modified interfaces, promoted methods, and inferred or baked generic interface signatures remain pending with source diagnostics.

## How to change it

Edit `jai-modules/src/params.rs` for argument keys, scalar/float coercion, and the parameter binding worklist. Edit `enum_parameters.rs` for enum values, nominal comparisons, and explicit target selection; `enum_masks.rs` handles pure flags mask expressions. Edit `type_parameters.rs` for structural source keys, dependent value coercion, and interface conformance. Keep nominal identity tied to declarations, not names or integer bits alone. `loader.rs` controls request ordering and separate source instances.

Extend semantic consumers alongside any new `ParameterValue` variant. Unresolved request variants must never become published parameter bindings. Extend `jai-sema/src/modules/aggregates/types/module_parameters.rs` and both type resolvers when adding type key forms. `prepared_session.rs` seeds canonical module type annotations before value-dependent aliases; `constants.rs` follows transparent aliases in their defining file and keeps deferred scalar diagnostics at their original source. Aggregate values and specialization require session-owned registry identities; do not introduce independently allocated `TypeId` values into sema.

`tests/insertion-module-parameters.rs` checks captured scalar, source type, and enum arguments against actual inserted imports, including a conflicting destination scalar. `tests/type-parameters.rs` checks source identities, dependent numeric bounds, and interface signature errors. `jai-sema/tests/module-type-parameters.rs` executes parameterized procedure bodies through the rewritten VM, including nominal type arguments and interface variables. `tests/enum-parameters.rs` uses `SourceOverlay` to exercise modern private defaults, nominal mismatch errors, program values across instances, and explicit target values whose ordinals intentionally differ from Preload's values.

## Configuration

`GraphOptions::import_dirs` controls named module searches. `load_with_provider` uses explicit filesystem/overlay inputs without target facts. `load_with_target` additionally supplies OS, CPU, byte order, and layout; only source-defined target tags currently affect dependency selection here.

## Dependencies

The module graph uses `jai-source` identities, `jai-syntax` structured declarations, `jai-eval` pure constant expressions, and `jai-types` checked integers, IEEE float values, and explicit build targets. It does not depend on sema, VM execution, or native reference binaries.
