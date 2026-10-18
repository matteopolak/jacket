# Jai language specification

These pages specify the Jai language that `crates/jaic` implements. The samples are short illustrations of each rule, not complete grammar productions. Where a rule is not pinned down, the page says it is unspecified; programs must not rely on unspecified behaviour.

- [Core syntax, declarations, types and values](core-syntax-and-types.md)
- [Procedures, control flow, data structures and memory](procedures-control-and-memory.md)
- [Polymorphism, compile-time execution and code generation](polymorphism-and-metaprogramming.md)
- [Modules, workspaces and scopes](modules-workspaces-and-examples.md)
- [Compiler code records and editing](compiler-code-records.md)
- [Public-language contract regression fixtures](../../../tests/compiler/language/README.md)

## Unspecified areas

The following are not fully specified by these pages. An implementation should report a clear diagnostic or a deliberate unsupported-feature error rather than silently picking a rule borrowed from C, Rust or another language:

- the complete token and expression grammar, including escape processing, precedence of uncommon operator combinations, and parse recovery;
- declaration order, shadowing, ambiguous name lookup, and scope-transition corner cases;
- endpoint behaviour of empty and reversed ranges, and less common iterable and pointer-iteration forms;
- evaluation order of side-effecting call arguments, and interactions between default and baked arguments;
- overload selection, polymorphic specialization identity and caching, and the full type-restriction grammar;
- the complete rules for implicit numeric conversions, overflow, enum conversions, union safety, and aggregate initialization;
- macro hygiene, backtick binding, expansion recursion limits, and every `#insert` scope option;
- import search paths, cycles, visibility interactions, replacement precedence, and module-instance identity;
- the full relationship among host execution, target compilation, workspace state, and compile-time side effects.
