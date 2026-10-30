# IDE facts and span lookups

## What it is

While `Compiler::ed` records, checking keeps what each name resolved to, what each expression's type was and which bodies checked to the end (`IdeFacts`, `crates/jaic/src/ide/facts.rs`). Features that hold a piece of syntax (the language server, `jailint`) ask about it by source span. The span lookups in `crates/jaic/src/ide/span_index.rs` are the one place those questions are answered.

## How it works

Each lookup is a method on `Compiler`. Its map is built the first time a lookup of that kind is asked and shared by every later one until the record changes.

| Query | Answers |
|---|---|
| `ide_entities_at(span)` | the entities the name at `span` (a use) resolved to; several when a polymorphic body was checked once per instance |
| `ide_locals_declared_at(span)` | the runtime variables declared at `span`: a local's name, a parameter's whole declaration, a loop's whole statement |
| `ide_procs_with_header(span)` | the procedures made from the header at `span` (instances of a polymorphic one share it) |
| `ide_uses_of_proc(proc)` | where `proc` is named, in the order recorded |
| `ide_procs_named_at(span)` | the procedures the name at `span` refers to (the reverse of `ide_uses_of_proc`) |
| `ide_body_is_clean(span)` | the procedure body at `span` checked to the end in every instance, so what was recorded inside it is the whole story |

The list answers are `Found<T>` (`jaic::ide`): a cheap shared view that dereferences to a slice, empty when nothing is recorded. It stays valid whichever lookup is called next, so a rule can hold one and ask another.

**The sketch mark.** `ide_sketch_polymorphs` checks polymorphic procedures nothing called with stand-in types. The references, entities and procedures it adds come after `IdeFacts::sketch_from`, and every lookup above stops there. This is applied in one place (`Compiler::ide_sections`); a new lookup built on `ide_sections` gets it for free. Anything that reads `IdeFacts::refs` or the compiler's entity and procedure tables directly must apply the mark itself (`IdeFacts::sketched`).

**Which files.** Names are indexed wherever references were recorded. Locals, headers and bodies are indexed for files the analysis records (`IdeRoot`, open documents); a span in another file finds nothing.

**Cache.** The maps are dropped when the number of references, entities or procedures changes, when a sketch begins, or when recording starts, stops or resumes. They are not dropped when a procedure only changes its `body_state`, so ask after `ide_check_all`, not during a compile.

## How to change it

- A new lookup: add a `OnceCell` to `Sections` in `span_index.rs`, build it from the part of the record before the mark, and add a method that returns `Found`.
- Prefer a lookup here over a map built in a feature: the second copy is the one that forgets the sketch mark.
- `refs_at` and `member_use_at` (`facts.rs`) answer by position (an offset inside a span) rather than by the exact span of syntax; use them for cursors.

## Configuration

None. The lookups follow the `IdeRoot`s the analysis was started with.

## Dependencies

`Compiler` (entities, procedures and their body states), `IdeFacts` (references, the sketch mark, the recorded roots). Tests: `crates/jaic/tests/ide_span_index.rs`.
