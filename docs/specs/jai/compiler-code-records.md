# Compiler code records and editing

## What it is

The public `Compiler` records are a mutable, user-facing syntax graph used to inspect `Code`, construct changed `Code`, and modify intercepted procedure bodies. This page specifies their public shape and behaviour; it does not prescribe the compiler's internal representation or a source-text conversion strategy.

## How it works

### Graph conversion and edits

The node discriminator is `Code_Node.kind`; there is no separate `tag` field. A bare `Code_Node` defaults to `.UNINITIALIZED` (`0`), but `Code_Ident` overrides its inherited `kind` default to `.IDENT` (`3`). Default initialization of a `Code_Ident`, including `Basic.New(Code_Ident)`, therefore produces an identifier node without the caller assigning `kind`. This comes from that record's explicit member initializer and ordinary struct-default rules, not from every derived type inferring its tag.

So a caller that default-initializes an identifier and assigns only `name` already has an `.IDENT` node when it passes it as `*Code_Node`. In contrast, a bare base node, an uninitialized allocation, raw zeroed memory, an explicit discriminator overwrite, or unrelated storage cast to `*Code_Ident` does not gain the derived record's initializer from its pointer type. `compiler_get_code` does not guess identifier semantics from `name` when the discriminator is still `.UNINITIALIZED`.

The following records also initialize their inherited discriminator: `Code_Binary_Operator` (`.BINARY_OPERATOR`), `Code_For` (`.FOR`), `Code_Procedure_Body` (`.PROCEDURE_BODY`), `Code_If`, and `Code_Procedure_Call`. These initializers are part of the member inventory, so regeneration keeps them. Records not listed have no implied default from their name; the explicit defaults in the member inventory are authoritative.

`compiler_get_nodes(code)` returns a root pointer and a flattened array of expressions at all levels. Every call returns a fresh user-level AST, so destructive edits do not affect another returned copy. Edits made through the flattened array, such as changing literal payloads, are visible through the root's links.

`compiler_get_code(node, code_to_copy_scope_from = #code,null)` produces `Code` from the given graph, including edits reachable from the root. The optional argument is a scope donor. Binding precedence with a null donor, conflicting resolved identifiers, and insertion scopes are unspecified. Source formatting, comments, exact spelling, record pointer equality after conversion, and the traversal order of the flattened array are not guaranteed.

`compiler_modify_procedure(w, body)` submits a procedure body for the selected workspace. A typical edit, such as an instrumenting profiler, replaces `body.block.statements` with a new slice holding newly allocated syntax records alongside original statements. The compiler must accept this mixed graph, keep the meaning of the original statements, and apply the submitted edits. Round-tripping an unchanged statement must not change its meaning or make an otherwise supported procedure uneditable.

### Syntax-bearing public fields

The complete member inventory is in [Compiler](../stdlib/modules/compiler.md). These fields matter most when editing and reimporting records:

| Construct | Public representation and required distinction |
| --- | --- |
| Floating literal | `Code_Literal.value_type = .NUMBER`; `value_flags.FLOAT` distinguishes its `_float64` union payload from integer payloads. `DEFAULTS_TO_FLOAT64` and `REQUIRES_FLOAT64` are additional public flags. These are not a separate literal kind. Preserve the numeric value and type semantics when changing an enclosing statement. Exact decimal spelling is not guaranteed. |
| Integer literal | The same `.NUMBER` variant uses `_s64` or `_u64`, with `MINUS_SIGN`, `HEX`, `BINARY`, and `IS_A_NUMBER` flags. Union alternatives share storage; only one is meaningful at a time. |
| Enum definition | `Code_Enum` exposes `internal_type_inst`, `internal_type`, `external_type`, `block`, `notes`, `marked_as_complete`, `marked_as_specified`, and `is_flags`. Member declarations use the block/declaration graph. There is no public enum-member-expression node kind. |
| Enum member expression | Explicit qualification and contextual dot notation are ordinary expression syntax, distinct from an enum definition. Preserve qualification and contextual lookup when reimporting, including a contextual member in a call argument. `Code_Ident` exposes `name`, `resolved_declaration`, and flags including `IS_RHS_OF_DOT_DEREFERENCE`. The exported graph shape for each enum-member spelling is unspecified; there is no enum-member public record. |
| Index expression | `Code_Binary_Operator` with `operator_type = Operator_Type.ARRAY_SUBSCRIPT` (`500`), `left` as the indexed expression, and `right` as the subscript, for an expression such as `items[slot]`. `[]` is not an infix source operator. There is no public `INDEX` node kind. |
| Range or collection loop | `Code_For.iteration_expression`, optional `iteration_expression_right`, and `block` distinguish collection iteration from a range with two endpoints. Preserve both endpoints and the loop body. |
| Loop variables | `ident_it` and `ident_it_index` are syntax identifiers used to create `ident_decl` and `index_decl`; those declaration pointers are null until type checking. Preserve explicit names, identifier flags, and binding roles. |
| Loop modifiers | `want_replacement_for_expansion`, `want_pointer_expression`, `want_reverse_expression`, and `for_flags` are public. `For_Flags` has `POINTER = 0x1`, `REVERSE = 0x2`. `macro_expansion_procedure_call` is ignored on import. Conditional modifier expressions are representable even though `Program_Print` does not fully print them. |
| If/switch | `Code_If` exposes `condition`, `then_block`, `else_block`, `if_flags` (`IS_SWITCH_STATEMENT`, `IS_IFX`, `IS_STATIC`, `MARKED_AS_COMPLETE`), `static_if_flags`, and `static_if_accepted_case`. The accepted-case pointer is set only for a type-checked static if. Preserve branch and switch semantics; evaluation metadata is not source syntax. |
| Case | `Code_Case` exposes one `condition: *Code_Node`, `then_block`, `owning_if`, and `marked_as_fallthrough`. A null condition is the default case, and explicit fallthrough is preserved. There is no public `labels` array. |
| Array type | `Code_Type_Instantiation.array_element_type`, `array_dimension`, and `inst_flags` distinguish fixed, view, and resizable types. A null dimension without `RESIZABLE` denotes a view. Preserve the element-type graph and dimension expression; do not substitute an untyped placeholder. |
| Pointer/type directive | `pointer_to`, `type_directive_target`, `type_valued_expression`, `must_implement`, and the relevant `inst_flags` carry type syntax. `result: *Type_Info` is export-only and ignored on import. |
| Array literal | `Code_Literal.value_type = .ARRAY` selects `array_literal_info`, whose fields are `element_type`, `alignment`, `array_members`, and `array_literal_flags.WAS_UNTYPED`. This is separate from an array type. |

`Code_Comma_Separated_Arguments.arguments` holds elements with `node` and `modifier`. It is not the exported representation of comma-separated case labels. A case has one constant value, a default case, or explicit `#through`. How several labels sharing one body are exported is unspecified; implementations must not invent a public field for it.

### Procedure identity and body ownership

`Code_Procedure_Header.body_or_null` may start out null. When compilation of that procedure body finishes, the compiler updates the retained header's pointer asynchronously, so a header obtained from a message or call browsing can be checked later for its body. Readers of that pointer must synchronize.

`Code_Procedure_Body` exposes `header`, `block`, and `body_flags.ALREADY_MODIFIED`, which reports an earlier modification. Body ownership tracks the actual workspace procedure the header and body represent; a display name cannot distinguish overloaded or unnamed procedures. Modification has no nonempty-name precondition, and an inline struct-member body is not ineligible because its header has an empty name. Once a body is exported as an editable workspace body, its header/body relationship must be kept so modification targets the right procedure.

Pointer equality among independently requested AST copies, headers from different workspaces, or polymorphic instances is not guaranteed. The fresh-copy rule applies to `compiler_get_nodes`; retained intercepted headers follow the body-publication rule above.

### Copies and compiler-owned metadata

New records and replacement statement arrays are valid inputs. Public syntax fields are authoritative for edits; unchanged input records may keep compiler-owned identity or semantic information. Implementation-private identity fields need not be Jai-visible members.

Shallow or deep copying of whole intercepted procedure bodies, transfer to another workspace, creating a procedure by copying a header, and keeping hidden metadata when a new record copies another's public fields are unspecified. So is import precedence for stale resolved links or `serial` values on caller-created copies.

A private association with original records is an implementation option, not a required algorithm. It must not erase supported public-field edits or invalidate unchanged children. For caller-created records, build what the public syntax says; copying a numeric serial does not transfer procedure ownership. Passing through the original source text is neither required nor a substitute for applying edits.

## How to change it

Update the [Compiler member inventory](../stdlib/modules/compiler.md) when the public schema changes. Keep unspecified import, copy and grouping cases marked unspecified here. A `Program_Print` limitation is not a compiler API restriction.

Good contract tests combine an enclosing edit with a kept floating literal, contextual enum member, indexed assignment, named range loop, switch/default/fallthrough, array type, or inline member procedure.

## Configuration

Workspace selection, interception flags, syntax flags, and the optional scope donor affect these operations. There is no configuration selecting a serialization format or requiring textual source reconstruction.

## Dependencies

`Compiler` supplies the records and compiler operations. `Preload` supplies `Code`, `Workspace`, type descriptors, and `For_Flags`. Compile-time execution and insertion consume generated `Code`; workspace interception supplies procedure bodies.
