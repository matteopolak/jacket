# Modules, workspaces and scopes

This page specifies module and import structure, scope directives, and the boundary between language behaviour and build tooling.

## Imports and modules

`#import "Name";` introduces a module. Modules can be imported by name or by file, from local files and subdirectories, and their members are reached through the module. `using` exposes selected members in the current scope, and can also apply to aggregate members; it is name resolution, not code copying. `using,only(names) Alias :: #import ...;` and `using,except(names) Alias :: #import ...;` bind the module under `Alias` and also bring the selected exports into the containing scope; the alias stays usable for qualified lookup. `only` admits the listed names, and `except` admits every export except the listed names. Name lists can be computed, and a filter may name something the module does not export. Declarations in file and global scope are shared across the files of a module.

`using,map(mapper)` applies to struct members. At compile time, the mapper receives the aggregate's member names and can rewrite a name, replace it with the empty string to omit that member, or leave it unchanged. Assignments through mapped member names update the original member. The mapper may change the name slice it is given, and any replacement strings must stay valid after it returns; it must not change the compiler-owned string contents. On a module import the mapper receives the names the module exports and renames or omits them the same way; the import's namespace alias keeps the original names. Mapping of overloaded operators is unspecified. `only` and `except` keep the original names.

Whether `Exported :: Impl.mutable_global` is rejected or captures an immutable value is unspecified; it is not a live alias of mutable storage. Use a `using` import or re-export to expose the original declaration. `type_of` on a foreign variable gives only its type, not a storage alias.

`#import` and `#load` differ. An import compiles a separate module with its own module scope, brought in anonymously or under a namespace. `#load` adds another file scope under the application or module scope. `#import,file`, `#import,dir`, and `#import,string` select source explicitly, including generated source.

Named `#import` aliases and `using` declarations follow the active scope directive. A plain top-level anonymous `#import` after `#scope_file` still contributes to the containing module's import environment, so a loading file can use its names; this does not make named aliases or file-private constants public. Each loaded file starts in its own file scope with `#scope_export` active; the caller's active directive does not carry across `#load`. Within a file, `#scope_file` selects the file-local scope and `#scope_export` selects the containing application scope or a module's importer-visible export scope. A file-local declaration can shadow a same-named enclosing declaration. Lookup goes from the current file scope to the enclosing application or module scope, then to the preload; an imported module cannot see into the importing application's scope. `using value;` exposes aggregate or enum members in the current scope, and `using name: Type;` declares and exposes at once. Shadowing can make short names ambiguous, so resolution follows scope nesting, not file load order.

Standalone aggregate `using,only(...)` and `using,except(...)`, including computed name lists, select which original member names are exposed, and a filter may name a member that does not exist. Module-import `using,only(...)` and `using,except(...)` select importer-visible exports while keeping the module alias for qualified access. A bare `using Alias :: #import ...` can be deduplicated, while repeated filtered imports are not deduplicated and can conflict. How separate filtered imports merge overloaded procedures, how conflicting non-procedure exports resolve, and whether aggregate filters compose with `map` are unspecified.

`#module_parameters` is only valid at the top level of an imported module; it is an error in the program's main file and in files the program `#load`s. Module parameters are compile-time inputs to a module, including local module instances configured by caller-provided values. Imports with identical parameter argument lists are deduplicated; different lists create separate module instances, even when defaults would make the effective values equal or when equivalent named arguments appear in a different order. This rule may change. Import replacement lets build configuration select alternate source for a dependency. The two are separate: module parameters configure a module instance, while replacement changes which implementation is loaded.

`Runtime_Support` supplies built-in runtime facilities and context members. Whether an explicit parameterized import of it shares the implicit runtime module instance is unspecified, and any reserved-module exception is not a general module identity rule.

Import search paths, import cycles, overload merging across several export sources, and replacement precedence are unspecified, and must not be taken from host filesystem conventions.

Scope directives place declarations explicitly: `#scope_file` places following declarations in the current file scope, while `#scope_module` and `#scope_export` place them in the containing module scope. Both module-scope forms make names available across the module's files, but only `#scope_export` adds them to the names exposed by anonymous imports and `using`. Named imports permit qualified access to both module-scope and exported declarations; file-scope declarations remain inaccessible. In an application that cannot itself be imported, the distinction has no effect. Generated module files can use `#placeholder NAME;` as a substitution point filled by the build or metaprogram that emits or loads the source. These directives control declaration visibility and generated-source integration; they are not runtime statements. `#program_export` marks procedures or data for an exported program or library interface; actual symbol export and ABI depend on the selected output target and platform.

## Build and workspace model

A workspace is a compilation unit with build options, target OS and CPU, output mode, and a metaprogram context. A custom metaprogram can add files and build strings, create another workspace, change its options, intercept compiler messages, and wait for completion. Cross-compilation separates host from target, and object-file output avoids needing target libraries or linkers.

`#run` and `#modify` can expose compile-time host and target boundaries. A metaprogram can pass build strings explicitly when compile-time file operations would otherwise run against the wrong platform. This is build-system behaviour, distinct from `#if` selection.

## Source and build inputs

`#string` embeds non-Jai source, including end-delimited blocks. `#file`, `#filepath`, and `#location` give source location and path values. These are compile-time inputs; paths are not runtime working-directory paths. `#compile_time` is a compile-time context query usable in conditions. Encoding and escaping rules beyond the delimiter forms are unspecified.

Arguments after the command-line separator go to the metaprogram, so command-line parsing is a build concern, not a language feature. Build options also set output kind and target properties; target-specific libraries and linkers are external build dependencies.

A library's local convention (for example, ownership or error handling) is not a language guarantee.

## Implementation notes for `jaic`

- Keep module loading and name lookup separate from expression parsing and type checking; import and file scopes take part in declaration collection.
- Keep file scope, module scope, caller scope for inserted code, and workspace-level metaprogram state distinct.
- Represent the compile target and host compile-time execution separately.
- Treat import replacement and module parameters as independent inputs to module resolution and instantiation.
- Search paths, cycles, visibility, replacement precedence, and module cache identity are unspecified; report clear diagnostics rather than guessing.
