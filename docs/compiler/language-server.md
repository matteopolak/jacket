# Shared Jai language server

## What it is

`jai-language-server` is one JSON-RPC language server used by both native editors and the browser playground. The native `jailsp` binary adds `Content-Length` stdio framing. The browser links the same Rust `JsonSession` into `jai_wasm.wasm`.

Run by hand, `jailsp --help` explains that an editor starts it; `--version`/`-V` prints the version; a file argument or unknown option is an error (exit status 2) that points at `jaic check file.jai`, and a terminal on stdin gets a note that it is waiting for LSP messages. Malformed framing ends the session with `error: jailsp stopped: invalid LSP input: ...`.

It has two layers:

- **Syntax** (always on): lexer/parser diagnostics, format-string checks, semantic tokens, document and workspace symbols, folding, and go-to-definition among the open documents. These come from the `jaic` lexer and parser.
- **Semantic** (when the session has an `Environment`): everything that needs the type checker. The open documents are compiled with `jaic`, compile-time code included, and the compiler's editor facts are queried. This includes [jailint](../tools/jailint.md)'s lints and their quick fixes.

### Feature list

| Feature | LSP method | Layer |
|---|---|---|
| Diagnostics: lexer and parser errors, `#load` targets, format strings | `textDocument/publishDiagnostics` | syntax |
| Diagnostics: the type checker's first error, also in code nothing calls (`jai-check` as `code`, see [dead-code elimination](../language/dead-code-elimination.md)) | `textDocument/publishDiagnostics` | semantic |
| Diagnostics: jailint lints (rule as `code`, `jailint` as `source`) | `textDocument/publishDiagnostics` | semantic |
| Hover: types of locals, members, procedures (every overload), structs, enums, constants | `textDocument/hover` | semantic |
| Hover: memory layout of types (size, alignment, padding), fields (offset, size, alignment) and aggregate-typed variables | `textDocument/hover` | semantic |
| Hover on a macro call: the macro's body with the arguments substituted | `textDocument/hover` | semantic |
| Hover on `#insert`: the inserted code | `textDocument/hover` | semantic |
| Hover on `#run`: its value and type, and what it printed | `textDocument/hover` | semantic |
| Hover on `#if` / `#ifx` / `#assert`: whether the condition held (per instance) | `textDocument/hover` | semantic |
| Hover on a format string: each `%` with the argument it formats and its type | `textDocument/hover` | syntax + semantic types |
| Completion: scope-aware names, members after `.`, directives after `#`, `#load`/`#import` paths | `textDocument/completion` | semantic, syntax fallback |
| Go to definition (also into modules and the stdlib) | `textDocument/definition` | semantic, syntax fallback |
| Go to the file of a `#load` / `#import` / `#import,file` / `#import,dir`, and of a module name (`B` in `B.print`) | `textDocument/definition` | environment |
| Document links on `#load` / `#import` strings | `textDocument/documentLink` | environment |
| Go to type definition | `textDocument/typeDefinition` | semantic |
| Find references, document highlights | `textDocument/references`, `textDocument/documentHighlight` | semantic |
| Rename (locals, globals, procedures with their overload declarations, types, modules, constants) | `textDocument/prepareRename`, `textDocument/rename` | semantic |
| Signature help, with the overload the call resolved to active | `textDocument/signatureHelp` | semantic |
| Inlay hints: inferred types of `x :=`, parameter names of literal arguments (only for parameters that share their type with another, so `print`'s format string gets none), `#run` values | `textDocument/inlayHint` | semantic |
| Code actions: show an expansion, inline an `#insert`, replace a `#run` with its value, add the `#import` an unknown name needs, apply a lint's fix (`quickfix`) | `textDocument/codeAction` | semantic |
| Commands `jai.showExpansion`, `jai.showPolymorphs` | `workspace/executeCommand` | semantic |
| Expansion documents (`jai-expansion:` URIs) | `jai/expansion`, `jai/source` (non-standard) | semantic |
| Code lens: how many polymorphs each polymorphic procedure has, and their bindings | `textDocument/codeLens` | semantic |
| Semantic tokens: types, procedures, macros, `$T`, constants, enum members, modules, directives, notes, `%` | `textDocument/semanticTokens/full` | syntax, refined by semantic |
| Document symbols (outline), workspace symbols | `textDocument/documentSymbol`, `workspace/symbol` | syntax |
| Folding ranges: blocks and runs of `#import`/`#load` | `textDocument/foldingRange` | syntax |
| Stdlib and module sources for read-only viewing | `jai/source` (non-standard) | environment |

Not supported: renaming struct members, formatting (see [jaifmt](../tools/jaifmt.md)), pull diagnostics, more than one type-checker error per compile, references to struct members.

## How it works

### Documents and syntax

`Session` admits versioned documents into a closed set of open files. Each version is lexed and parsed (`analysis.rs`) into declaration rows. The parser stops at its first error, so a version that does not parse has no rows. Completion then falls back to the rows of the last version that parsed (`Session::parsed`).

Positions use UTF-16, including supplementary characters and CRLF. Edits apply to a temporary copy and publish atomically. A stale version or invalid edit keeps the previous text.

### Semantic analysis

`semantic.rs` compiles on demand, only when a request needs it, and caches by source text:

1. **Overlay.** Open documents are laid over the environment's file system (`OverlayFs`). Natively that is the disk plus the repository stdlib; in the browser it is the bundled stdlib `VirtualFs`.
2. **Root.** The check starts from the document that `#load`s the requested one and is loaded by none (`Session::root`).
3. **Recording.** `Compiler::ide` is set to `IdeFacts` for files under the root's directory. While checking, sema records what each identifier and member names, with its type, and the source extent of block and procedure scopes. It also records expansions and calls. See [Editor facts](#editor-facts-in-jaic).
4. **All bodies.** `ide_check_all` then lowers every non-polymorphic procedure body in those files, not only what `main` reaches, so helpers nobody calls yet still have facts.
5. **Isolation.** Compile-time code runs in a `SandboxHost`, so `#run` output never reaches the protocol's stdout. The host is shared with `IdeFacts::output`, which is how a `#run` hover shows what it printed. It also has an interpreter block budget (`Interp::block_budget`), so an edit that makes `#run` loop forever traps instead of hanging. The compile also gets a workspace registry (`build::Workspaces`) on the same host, so a metaprogram's `#run` can create workspaces; each new workspace compiler gets the same budget.

Half-typed text usually does not parse. `repair` blanks lines with spaces, so byte offsets stay put, until the text parses: first the cursor's line (for completion and signature help), then the line the parser reports. If the error is reported on an empty line or a lone `}`, it blanks the last non-empty line before it instead, because that is where a missing `;` belongs. Every semantic feature goes through `Session::checked` (repair, compile or reuse the cache, look up the file), so hover, inlay hints and tokens of one version share one compile.

### Hover and completion

For completion, the word being typed and any `a.b.` chain before it are cut out of the text first. The probe text therefore stays the same while a word is typed, and the cached compile is reused for every keystroke.

- **Completion** asks `ide_scope_at` for the innermost recorded scope at the cursor.
  - **Plain names:** `ide_visible` walks the scope chain. It includes locals of the current procedure declared before the cursor, enclosing declarations, `using` members, imported modules' exports including re-exports, and Preload.
  - **After `a.b.`:** `ide_receiver` resolves the chain and `ide_members` lists the struct fields (through `using`), enum members, struct constants, or array/string/`Any` fields.
  - Results are filtered by the typed prefix (case-insensitive), and keywords are appended.
  - **After `#`:** the directives in `analysis.rs` `DIRECTIVES` (labels include the `#`, details are one-line descriptions), without compiling.
  - **Inside `#load "..."`:** files (`.jai`) and folders (`name/`) relative to the document, in the folder typed so far. **Inside `#import "..."`:** modules (folders and `.jai` files) on the import path and in the document's `modules/` folder. Entries come from the environment's `FileSystem::list_dir` plus the open documents (`path_completion`).
  - Trigger characters: `.`, `#`, `"` and `/`.
- **Hover** (`Session::hover`) tries, in order:
  1. a print-family **format string** under the cursor (`format_hover`);
  2. a **directive** token `#insert`, `#run`, `#if`, `#ifx` or `#assert` (`directive_hover`): what it produced;
  3. the smallest recorded **reference** at the offset: a local or member as `name: Type`, each overload of a procedure as `name :: <header>`, a type with its fields or members, a constant with its value. When the name is the callee of an `#expand` macro call, the expansion is appended (`with_macro_expansion`);
  4. any **expansion** containing the cursor where there is no name (the string of `#insert "..."`);
  5. the syntax layer's declaration text, or `keyword return` on a keyword (completion details say `keyword` too; hover text does not repeat the language name).

#### Memory layout in hovers

A semantic hover ends with a layout line when there is one layout to show (`IdeHover::layout`, written by `session.rs` `layout_line`):

```text
Pair :: struct { a: u8; b: s32; }
size 8, align 4 (3 bytes of padding)

b: s32
offset 4, size 4, align 4 (3 bytes of padding before)
```

- **Types** (structs, unions, enums, aliases, a `Poly(args)` instance): `size`, `align` and, for a struct, the bytes no field covers. A polymorphic struct's definition has none; hovering the name in `Node(int)` shows that instance (`IdeFacts::instances`, keyed by the callee's span).
- **Fields**, at their declaration or at `a.b`: the offset in the struct named (through `using` and `#as using` members held by value, and in the struct holding an anonymous `struct { ... }` / `union { ... }`), size, the alignment the layout used (a member's `#align` replaces the type's), and the gap since the previous fields of the same struct. `#place` / `#overlay` members show their shared offset.
- **Variables and constants** of structs, arrays, strings and `Any` show their type's size and alignment; scalars do not.

The numbers are the compiler's own (`StructInfo::fields[i].offset`/`align`, `Types::size_of`/`align_of`), the ones `size_of` and `type_info` report for the workspace's target; the language server computes nothing. Field declarations are not names the checker resolves, so `layout_struct` records each field's span (`IdeFacts::fields`, via `ide_note_layout`), and `a.b` records the type of `a` (`IdeFacts::receivers`). A field laid out in several structs (a polymorphic struct's instances) and a name checked with different types in a polymorphic body get no layout rather than one instance's.

Hover contents are built once as blocks (`hover.rs` `HoverText`: code, program output, prose, format rows and section starts) and written in the format the client asked for. `JsonSession` reads `capabilities.textDocument.hover.contentFormat` at `initialize`: if it lists `markdown`, hovers are `"kind": "markdown"`, otherwise `"kind": "plaintext"`. `Session::hover` is plain text; `Session::hover_as(uri, position, MarkupKind)` picks.

The Markdown is standard (CommonMark plus nothing a plain renderer would miss), so VS Code and other editors render it without an extension:

- Code (signatures, declarations, overload sets with one declaration per line, produced code, `#run` values) is a ```` ```jai ```` fenced block. Fences and inline code spans are longer than any backtick run inside (`` `total `` in a macro body).
- Prose is escaped (`hover::escape`); directive and keyword names in it are code spans: `` `#if`: the condition is true, the first branch is compiled ``, `` keyword `return` ``.
- Produced code and compile-time output go last, each in a section: a `---` thematic break, then the label as a paragraph that is only emphasis (`*expands to*`, `*expands to (2 of 3)*` for polymorphic instances, `*prints*`), then the code. What `#run` printed is a ```` ```text ```` block.
- The format-string hover is the literal in a `jai` block, then a list with one item per `%`: the specifier as code, `→`, and the argument with its type as code (prose when it is missing). The hovered specifier is bold when there is more than one.

Plain text keeps the older layout: the same content with a `─── label ───` line where Markdown has a section break, and `▸` marking the hovered format row.

Examples (Markdown):

````markdown
```jai
square :: (x: int) -> int #expand
```

---

*expands to*

```jai
total += (total + 2);
return (total + 2) * (total + 2);
```
````

````markdown
```jai
#run = 30: s64
```

---

*prints*

```text
computing
```
````

````markdown
```jai
"% and %2 of %1\n"
```

- `%` → `total: s64`
- **`%2`** → `s: s64`
- `%1` → `total: s64`
````

The same in plain text:

```text
square :: (x: int) -> int #expand
─── expands to ───
total += (total + 2);
return (total + 2) * (total + 2);
```

```text
"% and %2 of %1\n"
  %  → total: s64
▸ %2 → s: s64
  %1 → total: s64
```

The hosted playground asks for Markdown, draws a break followed by an emphasis-only paragraph as a rule with the label set into it, and styles the list as rows (see the portfolio's `docs/jai-language-features.md`). It reads only standard Markdown, so a new hover kind needs no client change unless it wants a layout of its own.

### Definition, references and type definition

- **Definition** (`ide_definition`) uses the reference at the cursor: an entity's declaration, every procedure of an overload set (aliases under their own name), or a struct. Spans are narrowed to the declared name; a procedure's span starts at its literal, so the name is found earlier on its line (`name :: (`). Targets can be modules or stdlib files the client never opened.
- **References** (`ide_references`) map every recorded reference to a target (a local or global entity, a procedure, a type or a module) and return those sharing the cursor's. A procedure's declaration resolves to the same `ProcId` its calls name, and a struct declaration to the same `TypeId` its uses name. Only files under the root's directory are recorded, so references inside the stdlib are not listed. Struct members are not tracked (a member reference does not record its owner type). Document highlights are the references within the document.
- **Rename** edits every reference that spells the old name (in open documents the text is checked; recorded files that are not open are edited by range). `prepareRename` refuses names the check did not record or that are members, and the new name must be an identifier that is not a keyword. A backticked caller name inside a macro body (`` `total ``) is a reference to the caller's local and is renamed with it.
- **Module names.** A name that evaluates to a module (`B` in `B :: #import "Basic"; B.print`) goes to the start of the module's entry file (`modules[m].files[0]`). `Module.name` records what the member resolved to (procedures, a type, a nested module) rather than a plain member, so definition and references follow names reached through a module, through `using`, and through re-exports (`module_lookup`, `exported_using_imports`). Modules are never merged: each `#import` is its own module and its names are reached through it.
- **Type definition** (`ide_type_definition`) takes the reference's type, strips pointers and arrays, and returns the struct or enum declaration.

### Editor facts in jaic

`crates/jaic/src/sema/ide.rs` holds `IdeFacts` and the name queries; `ide_meta.rs` holds the metaprogramming and call facts. The hooks are small:

- `check_expr` wraps `check_expr_kind` and records `Ident`, `Member` and `InferredMember` results (`ide_note_expr`). `check_ident` leaves the resolved entity in `IdeFacts::last_entity`.
- `add_entity` records each declaration's name span (`ide_note_entity`, with `IdeRef::decl` set).
- `ide_scope_span` is called where block scopes are made: procedure bodies, `check_scoped`, `if`/`case` arms, `while` bindings, `for` loops and block expressions.
- `eval_insert_operand` records what every `#insert` evaluated to (`ide_note_insert`): a string as is, a `Code` value as its source text (a block without its braces).
- `check_run` records each `#run` value, its type, its literal form and what it printed (`ide_note_run`). The `#insert -> string { ... }` form runs through `check_run_inner` and is recorded as an insert only.
- `eval_static_condition` records each `#if`, `#ifx` and `#assert` result (`ide_note_condition`). The compiler only has the condition's span, so `ide_widen` extends it back to the directive that precedes it; conditions with no directive in front (loop flags) are not recorded.
- `expand_macro` records the macro's body with arguments substituted (`ide_note_macro`, below).
- `call_procs` records every resolved call (`ide_note_call`): the overload set, the chosen procedure, and for each argument its span, parameter, and type.

Each site keeps up to four distinct results (`MAX_VARIANTS`), because a polymorphic body or a macro inside one can expand differently per instance; hovers then list them. Totals are bounded (`MAX_EXPANSIONS`, `MAX_CALLS`, `MAX_TEXT`).

**Macro substitution** works on tokens of the macro body's source (`substitute`): a parameter becomes its argument's source text (parenthesized unless it is an atom), `..args` of a variadic parameter becomes the arguments, `#insert c` of a `Code` parameter becomes the code, and the backtick of a caller-scope name is dropped because after expansion the name is the caller's. Backticked `return`/`defer`/`break` keep it. A name used as a member (`p.x`) or redeclared (`x :=`) is not replaced. This is a textual view of the expansion; it does not re-run the compiler, so it shows what the code does, not the exact checked tree.

All hooks do nothing when `Compiler::ide` is `None`, which is the case outside the language server.

Completion lists classify unresolved declarations by syntax (`ide_entity_name`): a procedure literal is a function, `struct`/`enum` is a type. Listing a module's exports therefore never compiles the whole module. Hover resolves the one entity it shows.

### Expansions: code actions, commands and documents

For an `#insert`, `#run` or macro call under the cursor (`code_actions` picks the smallest expansion containing it):

- **Show expansion** — a code action with the command `jai.showExpansion`, argument `{uri, position}`. Executing it returns the expansion object below; a client may instead call `jai/expansion` itself.
- **Inline #insert** (`refactor.inline`) — replaces the directive with its code. In statement position the trailing `;` is included and the code is indented to match; in expression position the code is parenthesized. Offered when every instance inserted the same code.
- **Replace #run with its value** (`refactor.inline`) — for a scalar, string or type result of a `#run` that printed nothing.

`jai/expansion` (request, non-standard) takes `{textDocument, position}` and returns `null` or:

```json
{
  "uri": "jai-expansion:///jai-script/main.jai?14:4",
  "kind": "insert",
  "text": "// Expansion of the #insert at main.jai:15:5\ninserted := 40 + 2;\n",
  "source": { "uri": "file:///jai-script/main.jai", "range": { "start": {...}, "end": {...} } }
}
```

`kind` is `insert`, `run`, `if` or `macro`. The URI encodes the document path and the expansion's start (0-based line and UTF-16 character), so `jai/source` with that URI recomputes the same text. Clients that open URIs lazily (VS Code `TextDocumentContentProvider`, the browser editor's read-only views) can therefore show it without keeping the response. The text is Jai with `//` comments, so it highlights as Jai.

`jai.showPolymorphs` (`{uri, position}` of a code lens) returns the bindings of each instance, such as `["T = s64", "T = string"]`.

### Lints and quick fixes

`lints.rs` runs [jailint](../tools/jailint.md) on each open document whenever diagnostics are published:

- **Compile.** `semantic.rs` always compiles with `IdeFacts::lint` set, so the facts jailint needs (expression types, casts, which names and imports were used) are recorded alongside the editor facts. `Analysis::lints` lints a file once per compile and caches the result. Diagnostics and code actions on the same text therefore reuse one compile and one lint pass, and so do hover and inlay hints.
- **When.** Only documents that parse are linted, from their real text, never a repaired one, so a half-typed line clears the lints until it parses again. Without an `Environment` (no type checker), no lints are published.
- **Settings.** The nearest `jailint.toml` above the document gives levels and excludes. An open one wins over one on disk: a client that sends `textDocument/didOpen` for a URI ending in `/jailint.toml` (any `languageId`), then `didChange`/`didClose` as it is edited, configures the documents under that directory. This is how the browser build, which has no disk, gets a workspace's settings. Settings files get no diagnostics of their own, and one that does not parse yet means the defaults. Changing one drops the cached lints (`Cache::forget_lints`), not the compiles. `deny` becomes an error, `warn` a warning. `format_arg_count` is always off here, because the syntax layer's `jai-format` diagnostics already cover format strings, including while the text does not parse.
- **Diagnostics.** The code is the rule name (`unused_variable`), the source is `jailint`, and `codeDescription.href` links to the rule's section of [the jailint docs](../tools/jailint.md#rules) (`lints::rule_url`). The message is the finding followed by its help line.
- **Quick fixes.** `code_actions` adds a `quickfix` for each lint with a fix that touches the requested range, or that the request's `context.diagnostics` names (matched by `source: "jailint"`, rule `code` and range). Each carries its lint as `diagnostics`, the rule as `data.rule`, and `isPreferred: true` when the fix is machine-applicable. Titles are the fix's description in sentence case (`Remove the unused variable`); identify the rule by `data.rule` or the diagnostic's `code`, not the title. Lint fixes come after the expansion actions.
- **Fix all.** A `source.fixAll.jailint` action applies every machine-applicable fix in the document that does not overlap an earlier one (`jailint::fix::choose`, the same choice as `jailint --fix`) as one edit, titled `Fix N lint problems`. `context.only` filters every action by kind (`source.fixAll` or `source` select it, `quickfix` leaves it out). `codeActionKinds` advertises `quickfix`, `refactor.inline` and `source.fixAll.jailint`.

### Missing imports

`imports.rs` turns the type checker's error into "Add `#import`" quick fixes when it is an unknown identifier that a standard-library module declares (`print` without `#import "Basic";`):

- **Where the modules come from.** The compiler works them out with the error's "did you mean" help (`Compiler::with_name_suggestion` in `sema/suggestions.rs`) and puts them on the diagnostic as data, `Diagnostic::fixes.imports` (`jaic::source::ImportSuggestion`: a module, and the name to bind it to for the namespaced form). Nothing parses the message. The order is the help's: a module named like the unknown name first (`Math.sqrt` gets `Math :: #import "Math";`), then the modules that declare it at their top level outside `#scope_file`/`#scope_module`, common ones (`Basic`, `String`, `Math`, ...) first, at most four. `semantic::Analysis::missing_imports` computes them the first time a code action asks (it reads the standard library) and caches them with the compile.
- **Actions.** One `quickfix` per module, titled ``Add `#import "Basic";` ``, carrying the `jai-check` diagnostic. It is offered when the error touches the requested range, or when `context.diagnostics` names it (`source: "jai"`, `code: "jai-check"`, same range). `isPreferred` is set only when the compiler found one module; with several (`log` is in `Basic` and `Math`) none is.
- **The edit.** It goes into the document the error is in, even when that file is `#load`ed by another one. After the last top-level `#import` (on the next line), else above the first line of code, below leading comments, followed by a blank line. A module the file already imports the same way (same module, same bound name) is not offered.
- **Not in fix-all.** `source.fixAll.jailint` stays jailint's machine-applicable fixes. Choosing a module is a decision about the program, and only the first compile error is known at a time, so this is a quick fix only.
- **Changing it.** The ranking (`COMMON_MODULES`, `MAX_IMPORTS`) and the help wording live in `sema/suggestions.rs`; where the line goes is `imports::insertion`.

### `#load` and `#import` links

`links.rs` finds `#load "..."` and `#import[,file|,dir] "..."` in the token stream (so links work while the text does not parse; `#import,string` has no file). `Session::link_target` resolves each with the compiler's own functions, `jaic::sema::import_entry` and `find_module_in` (which `Compiler::find_module` and `resolve_import` also call):

| Directive | Target |
|---|---|
| `#load "a/b.jai"` | relative to the loading file |
| `#import "Name"` | the first of `<file's dir>/modules`, then each import path (`-import_dir`s, then the stdlib), trying `Name.jai` before `Name/module.jai` |
| `#import,file "x.jai"` | relative to the importing file |
| `#import,dir "x"` | `x/module.jai` relative to the importing file |

Open documents count as files, so an unsaved module resolves. Import paths come from the environment's options for the check root, as for compiling; a target that does not exist gives no link.

- **Definition** on the directive or its string returns the target file at line 0.
- **`textDocument/documentLink`** returns `{range, target}` with the range on the string literal (quotes included).

In the browser the stdlib is bundled under `/stdlib`, so targets are `file:///stdlib/Basic/module.jai` and the client reads them with `jai/source`, like any definition into the stdlib.

### Inlay hints

`Session::inlay_hints` returns, within the requested range:

- **Types** (kind 1) after the name of `x := value` and `a, b := f()` (`ide_declared_types`: declaration references of locals). Skipped when the value starts with the type's name (`Thing.{}`) or casts to it, and when instances disagree.
- **Parameter names** (kind 2) before positional literal arguments (numbers, strings, `true`/`false`/`null`, `#char`, `.ENUM`), from the call facts. Named, variadic and spread arguments get none, nor does an argument spelled like its parameter.
- **`#run` values** after the directive's operand (`= 30`), when it computed one value and the operand is not that literal already.

### Format strings

`format.rs` finds print-family calls in the token stream, so it works while the text does not parse. The family is `PRINT_FAMILY`: `print`, `sprint`, `tprint`, `print_to_builder`, `print_color`, `log`, `log_error`, `log_warning`, `assert`. The format string is the first string-literal argument among the first two (`print_to_builder(builder, "...")`, `assert(cond, "...")`). Arguments after it count, except named ones and those after `,,` (context overrides).

Directives follow `stdlib/Basic/Print.jai`, not older documentation:

| Text | Meaning |
|---|---|
| `%`, `%0` | the next argument |
| `%N` | argument N (1-based); a following `%` continues with N+1 |
| `%%` | two directives: the next two arguments |
| `%00` | prints nothing |
| `\%` | a literal percent sign (not a directive) |

They feed three features:

- **Semantic tokens:** the string token is split, and each directive is a `formatSpecifier` token.
- **Diagnostics** (`jai-format`): an error on a directive that refers past the last argument (printing would fail), a warning on each argument no directive uses. A `..spread` argument disables the check.
- **Hover:** anywhere on the string, the summary shown above. Types come from the recorded call whose span contains the string (the variadic `Any` arguments keep their checked types).

### Signature help

`open_call` scans back from the cursor to the unmatched `(` and reads the callee (`name` or `Module.name`), the argument index, and a `name =` being typed. If the check recorded a call starting at that callee, its overload set is shown with the chosen overload active. Otherwise (the usual case while typing, when the line does not parse) the cursor's line is blanked and `ide_callee` looks the callee up in the scope at the cursor. A named argument selects its parameter; past the last parameter the last (variadic) one stays active. Triggers: `(` and `,`.

### Semantic tokens

The legend is append-only so older clients keep their mapping:

- types: `keyword`, `string`, `number`, `variable`, `function`, `type`, `property`, `parameter`, `macro`, `operator`, `namespace`, `typeParameter`, `enumMember`, `decorator`, `formatSpecifier`;
- modifiers: `declaration`, `readonly`, `macro`.

The syntax layer classifies declarations from the parse rows. With an environment, `ide_classes` refines every recorded identifier: a use of a type is `type`, of a procedure `function`, of an `#expand` procedure `function` + `macro`, of a constant or enum member `readonly`, of a module `namespace`, of a struct field `property`. `$T` and `$$T` and every use of that name inside the procedure are `typeParameter` (from tokens, so it works without checking). Directives (`#run`) stay `macro` as before; notes (`@note`) are `decorator`.

### Syntax-only extras

- **Workspace symbols** search the declaration rows of every open document (top-level and nested), case-insensitive substring.
- **Folding ranges** pair `{}`, `()` and `[]` across lines (ending on the line before the closer) and fold runs of `#import`/`#load` lines as `imports`.
- **Code lenses** come from `ide_polymorphs`: each polymorphic procedure in the file with its instances' bindings (`T = s64`). Instances exist only for calls the check reached.

### Protocol and browser

`JsonSession::handle_json` handles the lifecycle (initialize/initialized, shutdown/exit), document open/change/close, cancellation, every method in the feature list, and two non-standard requests:

- `jai/source` (`{uri}` → text or `null`): the text of a definition target the client has not opened (a browser editor can show stdlib files read-only), from the open documents or the environment's file system, or of a `jai-expansion:` URI;
- `jai/expansion` (`{textDocument, position}` → expansion or `null`).

`initialize` advertises each provider, the token legend, the commands, and `experimental.jai.expansions: true`.

Completion kinds map to LSP numbers in `protocol.rs`: function 3, field 5, variable 6, module 9, keyword 14, file 17, folder 19, enum member 20, constant 21, struct 22. Clients (including the hosted playground's editor) map those numbers to icons.

The worker carries `{type: "lsp", id, message}`. The wasm bridge (`crates/jai-wasm/src/language_server.rs`) keeps the session in a `thread_local`, because the compiler state uses `Rc`.

## How to change it

- **Record more facts.** Add a hook in sema that calls an `ide_*` method guarded by `self.ide.is_some()`, and keep the hook cheap. Name facts go in `ide.rs`; metaprogramming and call facts in `ide_meta.rs`. To show more in hover, extend `ide_hover`/`ide_entity_hover`, or `Session::describe` for expansions. Build hover text from `hover::Block`s rather than formatting strings, so both the Markdown and the plain-text form follow; a new block kind needs a case in `HoverText::plain` and `HoverText::markdown`. For more completion sources, extend `ide_visible`/`ide_members`.
- **New expansion kind.** Add an `IdeExpansionKind`, record it where the compiler evaluates it, and handle it in `kind_name`, `describe`, `expansion` and `code_actions` (`features.rs`).
- **Print-family procedures** are a name list (`format::PRINT_FAMILY`), because diagnostics are published without compiling. A user wrapper is still recognized by the hover's type lookup (`IdeCallInfo::format_param`), but not by diagnostics or tokens until its name is added.
- **Format semantics** live in `jailint::format_string` (shared with the `format_arg_count` rule; `format::specs` wraps it). Keep them in step with `__format_to_builder` in `stdlib/Basic/Print.jai`.
- **Scope extents.** A new kind of block scope needs an `ide_scope_span` call, otherwise completion inside it sees the enclosing scope only.
- **Repair heuristics** live in `session.rs` (`repair`, `blank_line`). They must keep byte offsets unchanged, because positions are mapped back into the real text.
- **Token legend.** Append to `TOKEN_TYPES`/`TOKEN_MODIFIERS` in `lib.rs` and to `semantic_tokens_wire` together; never reorder.
- **Environment.**
  - Native: `main.rs` `native_environment` (import paths, Preload).
  - Browser: `language_server.rs` `environment` (bundled stdlib, wasm target).
  - Keep these in step with how `jaic` and the playground compile.
- **Syntax features** stay in `analysis.rs`/`format.rs`. Typed results live in `model.rs`; extend its types and the wire mappings in `protocol.rs` together.
- **Lints.** Rules live in `crates/jailint` (see [how to add a rule](../tools/jailint.md#how-to-change-it)); the server picks them up without changes. `lints.rs` maps findings to diagnostics and fixes, and `DiagnosticCode::Lint` carries the rule name to the wire.

Tests:

- `crates/jai-language-server/tests/semantic.rs`: hover, completion while typing, member completion, and hover with a broken line elsewhere.
- `crates/jai-language-server/tests/features.rs`: expansion hovers (macro, `Code` argument, `#insert`, `#run` with output, `#if` true/false/per instance), format-string hover and diagnostics, lint diagnostics and quick fixes, "Add `#import`" quick fixes (one module, several, namespaced, a `#load`ed file, asked for by diagnostic), the Markdown form of each hover kind, inlay hints, code actions and expansion documents, semantic tokens, references, type definition, signature help (recorded and while typing), workspace symbols, folding, code lenses, keyword wording, and the JSON protocol for each request.
- `crates/jai-language-server/tests/links.rs`: definition and document links for `#import` (stdlib, `modules/`, `Name.jai` before `Name/module.jai`, missing module), `#import,file`, `#import,dir` and `#load`; module names; names through a module, `using` re-exports and plain imports.
- `crates/jai-language-server/tests/protocol.rs`: hover format negotiation (`markdown` listed or not).
- Unit tests: `hover.rs` (escaping, fences, sections), `links.rs` (directive scanning), `format.rs` (directive semantics), `features.rs` (call scanning, inlining, declarations), `jaic/src/sema/ide_meta.rs` (substitution, dedent).
- `crates/jai-wasm/src/language_server.rs`: hover, completion, inlay hints, expansions, format strings and `#import` links into the bundled stdlib through the wasm bridge with the bundled stdlib.
- `node tools/check_scripting_wasm.mjs <jai_wasm.wasm>`: the same against the real WebAssembly module.

## Configuration

```sh
cargo run -p jai-language-server --bin jailsp
cargo test -p jai-language-server
cargo build --release -p jai-wasm --target wasm32-unknown-unknown
node tools/check_scripting_wasm.mjs target/wasm32-unknown-unknown/release/jai_wasm.wasm
```

- `JAIC_STDLIB` overrides the stdlib directory the native server reads. The default is the repository's `stdlib/`.
- A `modules/` folder next to the root document is searched first.
- `jailint.toml` (nearest above a document): lint levels and excluded paths.
- `semantic.rs` constants:
  - `BLOCK_BUDGET`: interpreter blocks per analysis.
  - `CACHED`: compiles kept, 3.
- `ide_meta.rs` constants: `MAX_EXPANSIONS` (4,096), `MAX_CALLS` (16,384), `MAX_VARIANTS` (4 per site), `MAX_TEXT` (64 KiB per expansion).
- `features.rs`: `HINT_CHARS` (40), the longest inlay hint label.
- Hover format comes from the client: `textDocument.hover.contentFormat` in `initialize` (Markdown when it lists `markdown`, else plain text).
- Default `Limits`:
  - 32 documents, 256 KiB per document, 4 MiB total.
  - 1,024 completion items and workspace symbols, 8,192 tokens.
- Documents must use absolute `file:///...` URIs. The browser uses `file:///jai-script/<name>`. Expansion documents use `jai-expansion:///<path>?<line>:<character>`.

## Dependencies

- `jaic`: lexer, parser, sema with `IdeFacts` and `ide_meta`, interpreter `SandboxHost`.
- `jailint`: lint rules, `jailint.toml` settings and the shared format-string reader.
- `serde` / `serde_json` for JSON.
- The browser adapter links into `jai_wasm.wasm` and is reached through `engine.lsp(message)` ([browser compiler](../browser/playground.md)). The hosted playground's editor lives in the portfolio repository.
- Protocol: [LSP 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/) over [JSON-RPC 2.0](https://www.jsonrpc.org/specification).
