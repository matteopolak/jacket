# Polymorphism, compile-time execution and code generation

Compile-time values, polymorphic declarations, macros, reflection, and workspaces are part of the language. This page specifies them and separates compile-time evaluation from runtime control flow.

## Compile-time constants and `#run`

`::` creates a constant binding. Compile-time type values, literal values, and results of compile-time evaluation can be used in type positions and static conditions. `#run expression` evaluates a procedure or block during compilation and exposes its result as a constant. This is distinct from runtime execution: it may create declarations, compute type information, and prepare generated code before the program runs.

```jai
capacity :: 6;
square :: (n: s64) -> s64 { return n * n; }
SQUARE_OF_CAPACITY :: #run square(capacity);
Buffer :: struct ($T: Type) { data: [capacity] T; }
```

`#if` compiles one branch based on a constant condition, typically platform and build constants such as `OS`, `CPU`, and `IS_CROSS_COMPILING`. A mutable variable is not accepted as a constant merely because its initializer looks obvious. `is_constant` selects compile-time assertions and conditional baking. Constantness is a type-checking property, not an optimizer guess.

Compile-time execution depends on the platform it runs on. Ordinary target-workspace `#run` is distinct from explicit `#run,host`, and foreign or system calls are not automatically remapped to the host ABI. See the execution boundary below.

### Cross-target compile-time execution

| Form | Compilation and lookup domain | Native-operation boundary |
| --- | --- | --- |
| Ordinary `#run` in a cross-target workspace | Target bytecode, target `OS`/`CPU`, and target types and declarations | Runs inside the host compiler process. Target foreign libraries, syscall constants, and assembly must actually work on the host; native calls are not remapped. |
| `#run,host` | An isolated host workspace with host `OS`/`CPU` and explicitly declared host imports | Host module variants are compiled and called. Target identifiers and types are not visible. |
| Host metaprogram work | The metaprogram's own workspace | Can do host work and pass generated source or results into the target workspace through the public compiler APIs. |

For ordinary cross-target `#run` there are two exceptions: the `Context_Base` portion of the context is initialized with host workspace values (including the allocator and logger procedure pointers), and calls to `runtime_support_default_allocator_proc` are redirected to the host workspace's equivalent. Allocation therefore works without target platform functions becoming host functions. A target procedure can still run target-specific code before it reaches a host context procedure, so the host logger pointer alone does not make every logging call work across architectures.

Consequently, a Linux-target timing call that passes Linux clock identifiers to a macOS native call, or a Linux errno-symbol lookup that does not exist on macOS, is outside what ordinary `#run` supports. Native constants and symbols are not translated between platforms. Cross-compilation itself, and compile-time native calls that are valid on the host, are still supported.

`#run,host` is the form for such host operations. Its imports must be declared inside the isolated host procedure; importing a host module into the outer target workspace is not equivalent. The isolated procedure takes no arguments, and its result is limited to built-in primitive types, strings, or arrays of built-in primitives. Target-defined aggregates cannot be transferred, and outer target names cannot be captured. In a workspace that is not cross-compiling, `#run,host` behaves like ordinary `#run`.

A cross-target workspace is configured through the `Compiler` API, not a command-line `-os` flag: create a workspace, read its `Build_Options`, set `os_target = .LINUX` and `cpu_target = .X64`, then apply the options before adding target source. `output_type = .OBJECT_FILE` avoids needing target libraries or a linker. `runtime_support_definitions = .ENTRY_POINT_AND_INIT` is optional, for when an externally linkable entry point is wanted. Begin interception, add source with `add_build_file` or `add_build_string`, and read that workspace's `Message_Complete.error_code`. The host metaprogram can suppress its own output with `set_build_options_dc`.

## Polymorphic procedures and structs

`$T` declares a compile-time type variable inferred from a call. The procedure body is type-checked for the concrete type arguments, and a result can depend on the inferred type. Polymorphic parameters can be named and reused across parameters and results. `$value` requires a compile-time value argument, while `$$value` is an optional bake: it is constant when the caller passes a constant, and a runtime value otherwise. `is_constant` and `#if` let a body specialize when that optional value is known at compile time. Auto-bakes make eligible arguments compile-time parameters when calls pass constants; fixed polymorphic parameters are distinct from inferred bakes.

`#bake_arguments` creates a specialized callable by fixing selected arguments of an existing procedure. Unlike default arguments, its result is another callable with fewer inputs.

Polymorphic structs take compile-time type or value parameters and create specialized aggregate types. Type restrictions constrain which types may be substituted. `#type,distinct` and `#type,isa` create type variants with different compatibility rules. `#modify` blocks inspect, validate, or customize types at compile time. Specialization caching, overload ranking, and the full constraint grammar are unspecified.

## Macros, `Code`, and `#insert`

An `#expand` procedure is a compile-time macro expansion. Its arguments can include code values, and backticks refer to names visible at the call site. A backtick identifier is caller-scope name lookup; it does not splice a `Code` parameter by that parameter's name. Use `#insert` with the `Code` value to insert that argument. `#insert` accepts a string of Jai source, a `Code` value, or code returned from `#run`, and its scope options make inserted code resolve in the caller's scope.

```jai
scale_result :: (factor: s64) #expand {
    `result *= factor;
}

apply_scale :: () {
    result := 7;
    scale_result(4);
}
```

Backtick binding rules, hygiene, evaluation timing, and insert-scope modifier details beyond this are unspecified. Code values can also be parsed and modified through compiler APIs: `compiler_get_nodes` and related helpers expose syntax records and convert edited records back to `Code` (see [compiler code records](compiler-code-records.md)). Such edits are compile-time work, not runtime reflection.

## Reflection and compiler integration

`type_info(T)` requests runtime metadata for a type. For a named declaration, `Type_Info.name` is the declared name: a type declared as `Derived` reports `"Derived"`. A type alias is the same type, not a new nominal declaration, so an internal alias name is never reported in place of the public declaration name. The name format of parameterized type instances is unspecified. Only metadata the program needs is kept, and struct directives can reduce it further, including omitting it or representing procedure types as void pointers. Metadata reduction does not change the value's compile-time type.

The `Compiler` module supports custom metaprograms, injecting build strings and files, workspaces, compiler messages and interception, source browsing, hooks, custom checks, module browsing, and caller-code inspection. These are APIs available to a metaprogram entry point, not intrinsic syntax, and an implementation should keep language semantics distinct from tool and workspace API behaviour.

`#no_reset` can carry compile-time-initialized global data into the runtime output, but ordinary pointers are not relocated. The default metaprogram performs startup and build behaviour and can be replaced or extended. Command-line arguments after the metaprogram separator go to the metaprogram. Inline `#asm` may run during compile-time execution, subject to architecture and backend support.

## Implementation concerns

- Keep compile-time evaluation separate from runtime IR while sharing the type rules.
- Type variables and baked values affect signatures and specialization; keep them compile-time.
- `#if` excludes a branch semantically but both arms must parse.
- `#insert` and macro expansion need the caller scope and inserted-code location represented deliberately.
- Runtime `Type_Info` has a size cost, and reduction directives change metadata only.
- Cross-compilation must not assume host compile-time execution behaves like execution on the target.
- Grammar details, error recovery, expansion recursion limits, and the exact metaprogram lifecycle are unspecified.
