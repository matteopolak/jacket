# Language server: refactorings, call hierarchy and selection ranges

## What it is

Three groups of rust-analyzer-style features in `jai-language-server`, in addition to the [feature list](language-server.md#feature-list):

- **Refactor code actions** (`refactor.extract`, `refactor.inline`, `refactor.rewrite`) from `src/refactor.rs`.
- **Call hierarchy** (`textDocument/prepareCallHierarchy`, `callHierarchy/incomingCalls`, `callHierarchy/outgoingCalls`) from `src/hierarchy.rs`.
- **Selection ranges** (`textDocument/selectionRange`, "expand selection") from `src/selection.rs`.

All three are advertised in `initialize` (`codeActionKinds`, `callHierarchyProvider`, `selectionRangeProvider`) and work in VS Code and, for the code actions, in the browser playground (see [Playground](#playground)).

## Refactor code actions

| Action | Kind | Offered when |
|---|---|---|
| *Extract into variable* | `refactor.extract` | The selection is exactly one expression inside a procedure body |
| *Extract into procedure* | `refactor.extract` | The selection is whole statements of a top-level procedure |
| *Inline variable `x`* | `refactor.inline` | The cursor is on the name or a use of `x := value;` |
| *Add missing cases (N)* | `refactor.rewrite` | The cursor is in the header of an `if x == {` on an enum with members no `case` names |
| *Add missing fields (N)* | `refactor.rewrite` | The cursor is inside `Type.{...}` and some named fields are absent |
| *Convert ifx to if/else* | `refactor.rewrite` | The cursor is on the line of `x := ifx ...;`, `x = ifx ...;` or `return ifx ...;` |
| *Convert if/else to ifx* | `refactor.rewrite` | The cursor is on the line of an `if`/`else` whose branches both assign the same place, or both `return` |

The actions are computed from the parse tree of the open document (`jaic::parser`), with three facts from the type-checked compile (`Session::checked`): the types of declared locals (`ide_declared_types`), the references of a name (`Session::references`) and the members of an enum or struct (`ide_members_at` in `jaic::sema::ide_meta`). The tree is walked with `jailint::syntax` (`walk`, `Node`), which does not enter procedure bodies; `proc_bodies` finds those.

An action is offered only when its edit is safe, and declines otherwise. A client that sends `context.only` without a `refactor` kind never pays for the analysis.

### Extract into variable

`foo(a + b)` with `a + b` selected becomes `value := a + b;` on the line above and `foo(value)`. The name comes from the call or member (`get_count(x)` gives `count`), or is `value`; a name the document already uses gets a number. Declined when:

- the expression is a whole initializer, a whole statement, a callee, an assignment target or an address (`*x`), a type (a name like `Hash_Table`), or has no value of its own (`.{...}` or an `.NAME` without a type);
- moving it would change when, or whether, it runs: the condition of a `while`, the right side of `&&`/`||`, the branches of `ifx`, a `case` value;
- the statement is not in a list (the body of `if c stmt;` has no line to insert before).

Parentheses around the selection are kept out of the extracted value (`(a + b)` extracts `a + b`).

### Extract into procedure

Whole statements selected (the selection starts at the first statement and ends at the last, a trailing `;` included or not, with nothing else on their lines) become a new procedure `extracted` placed after the enclosing top-level procedure. The call replaces them at the same indentation.

- **Parameters**: the locals and parameters of the enclosing procedure that the selection reads, in order of first use. A type is the one written in the declaration, else the compiler's (`x := 5` is `s64`).
- **Results**: variables the selection declares directly and that later statements of the same block use. They become the procedure's return values and are declared by the call: `a, b := extracted(x);`.
- **Declined** when the selection `return`s, `defer`s, `break`s or `continue`s out of itself (or uses a label), uses `using`, `#insert`, `#run`, backticks, `#code`, `#asm`, procedures, lambdas, structs or enums, assigns or takes the address of (`*x`, `array_add(*x, ...)`) a variable declared outside it, shadows a name it also reads, needs a type nobody knows (the implicit `it` of a loop outside the selection), or sits in a polymorphic, `#expand`, `#c_call` or nested procedure. Parameters are constants in Jai, so a modified outside variable cannot become one; it is left to the user.

### Inline variable

`x := a + b;` is removed and each use of `x` becomes `a + b`, in parentheses where precedence needs them (not around an argument or a whole statement). Uses come from the compiler's references, so shadowed names are not confused. Declined when `x` has a written type, is assigned, has its address taken or is a `for` pointer target, is used where the tree walk cannot see it (inside a lambda), has no uses, or is initialised with a call and used more than once (it would run twice).

### Add missing cases and fields

- **Cases**: `case .NAME;` for each enum member the switch does not name, in declaration order, before a `case;` default if there is one, else before the closing `}`. They have no body (a case that does nothing). Needs the enum type of the value from the compile; works with `#complete`.
- **Fields**: `name = value` for each named field of a `Type.{...}` literal that is not there, with a value that suits the type (`0`, `0.0`, `false`, `""`, `null`, `.{}`, `.[]`, the first member of an enum, `---` otherwise). `using`, `#as` and overlaid fields are left out. Declined for literals with positional values (`Point.{1, 2}`) and for `.{...}` without a type. A literal on one line stays on one line (`Point.{ a = 1, b = 0 }`), a multi-line one gets one field per line at the indentation of the last field.

### `ifx` and `if`/`else`

```jai
big := ifx n > 3 then 10 else 20;      big: s64;
                                       if n > 3 {
                                           big = 10;
                                       } else {
                                           big = 20;
                                       }
```

The declared type of `big` comes from its written type or the compiler's. `x = ifx ...` (also `+=` and the like) and `return ifx ...` convert the same way. The reverse needs both branches to be one statement (a bare statement or a block with one) that assign the same place with the same operator, or `return` one value; `else if` chains are left alone.

### Formatting

Every edit is written the way [jaifmt](../tools/jaifmt.md) lays code out, so formatting the result changes nothing: one statement per line, braces on the header's line, `case` lines at the indent of the others, one indentation unit per level (the smallest indent step of the document: tabs if it uses them, else 2 to 8 spaces, default 4), `Type.{ a = 1 }` with spaces inside the braces when empty or on one line, and the line ending the document uses. This was checked by formatting the outputs with a built `jaifmt`; keep it so when changing a template.

## Call hierarchy

- **Prepare**: the cursor on the name of a procedure declared as `name :: (...)`, or on a use: `Session::definition` finds the declaration (in any compiled file, modules and the stdlib included) and the file is parsed for the declaration around it. Procedures nested in procedures and in structs are found too. Not on a procedure: `null`.
- **Incoming calls**: the references of the procedure's name (`Session::references`, so overloads and cross-file uses work) that are followed by `(`, each assigned to the procedure whose body holds it, grouped by caller with every call site as `fromRanges`. A use outside any body (a global `#run`) has no caller and is skipped.
- **Outgoing calls**: the calls the compile recorded (`ide_calls`) inside the procedure's body, each resolved to the overload it chose, grouped by callee. Calls of procedures nested in this one belong to them. `fromRanges` cover the callee as written (`a.b` too).

Both directions need the item's file to be an open document, since they query that document's compile; items for callees in modules can be expanded to their own outgoing calls only when those files are open. An item carries its URI and ranges and nothing else (`data` is unused), so any client's round trip works.

## Selection ranges

For each position the chain is the token under the cursor (plus the text inside a string), then every expression and statement around it (parenthesised operands included), the body of each block between its braces and with them, `case`s, procedures and structs, up to the whole file. Ranges are deduplicated and nested strictly. A document that does not parse still gets the token and the file.

## How to change it

- **New refactoring.** Add a method to `Cx` in `refactor.rs`, call it from `refactor_actions`, and give it a kind that `codeActionKinds` in `protocol.rs` advertises (all three `refactor.*` kinds are). Use `Cx::find` (first node in procedure bodies) or `Cx::innermost` (smallest around the cursor) to locate the tree node, `extent`/`stmt_extent` for source ranges and `in_list` before inserting a statement. Spans from the parser leave out parentheses around the first operand (`(1 + 2) * 3` starts at `1`), so always take extents from `extent`, never from `Expr::span` directly.
- **Facts the compile has to give** go in `jaic::sema::ide_meta` as an `ide_*` query (`ide_members_at` is the example), reached through `Session::checked`.
- **Declining is the default.** When a case is unclear, return without an action and add a test to `declines_*`.
- **Tests.** Applying an action in tests re-opens the result and requires no compile error (`compiles` in `tests/refactor.rs`), and `offered_refactorings_always_parse` applies every action offered on four stdlib files at several selections and requires the result to parse.

## Configuration

None. The features need no settings; the refactors that need types (inline variable, fill, extract into procedure) are not offered when the program has no `Environment` (the syntax-only session) or does not compile far enough to know the types.

## Playground

`crates/jai-wasm` passes JSON-RPC through to `JsonSession`, so the new requests are answered in the browser too. The playground's editor asks for `textDocument/codeAction` with its selection, so the refactorings show in its light-bulb menu without a change. It does not ask for call hierarchy or selection ranges (CodeMirror has no matching UI), which are therefore not shown there.

## Dependencies

- `jaic`: the parser, `sema::ide_meta` (`ide_members_at`, `ide_declared_types`, `ide_calls`).
- `jailint::syntax`: the tree walker (public since the refactorings use it).
- The session's `references` and `definition`, and `Session::checked` for the cached compile.

## Tests

- `crates/jai-language-server/tests/refactor.rs`: each refactoring and what it declines, the offered-edits-parse sweep.
- `crates/jai-language-server/tests/hierarchy.rs`: call hierarchy (declaration or use, incoming, outgoing, modules, nested procedures), selection ranges and the JSON requests.
- `editors/vscode/test/e2e/suite/extension.e2e.ts`: a refactoring, the call hierarchy and expand-selection through VS Code's commands.
