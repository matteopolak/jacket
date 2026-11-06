# IDE facts and span lookups

While `Compiler::ed` records, checking keeps what each name resolved to, each expression's type, and which bodies checked to the end (`IdeFacts`, `crates/jaic/src/ide/facts.rs`). Features holding a piece of syntax (language server, `jailint`) ask about it by source span through the lookups in `ide/span_index.rs`.

## How it works

Each lookup is a `Compiler` method, built on first use and shared until the record changes.

- `ide_entities_at(span)`: what the name at `span` resolved to (several for polymorphic instances).
- `ide_locals_declared_at(span)`: runtime variables declared there.
- `ide_procs_with_header(span)` / `ide_procs_named_at(span)` / `ide_uses_of_proc(proc)`: procedures by header, by name, and where a procedure is named.
- `ide_body_is_clean(span)`: the body checked to the end in every instance.

List answers are `Found<T>`, a cheap shared slice view that stays valid across further lookups.

- **Sketch mark.** `ide_sketch_polymorphs` checks polymorphic procedures nothing called, with stand-in types. What it adds comes after `IdeFacts::sketch_from`, and every lookup stops there (applied once in `Compiler::ide_sections`). Code reading `IdeFacts::refs` or the entity tables directly must apply `IdeFacts::sketched` itself.
- **Files.** Names are indexed wherever references were recorded; locals, headers and bodies only for recorded roots (`IdeRoot`, open documents).
- **Cache.** Dropped when reference, entity or procedure counts change, a sketch begins, or recording starts or stops. Ask after `ide_check_all`, not during a compile.

## Lifetime sidecar

With lint facts on (`IdeFacts::lint`), each checked body also records a compact event list for `pointer_lifetime` (`ide/lifetimes.rs`, `IdeFacts::lifetime_bodies`, per file through `lifetime_bodies_in`). It holds per-instance local identities, typed loads, stores and memory reads, index and field projections, view conversions, pointer copies (offsets, casts), returns and call results, selected callees with argument-to-parameter mapping, `#if` choices, macro and `#insert` output, and the final control edges from IR lowering (branches, switch, labelled jumps, short circuits, `defer` cleanup). No IR or second syntax tree is kept.

- Recorded only for real bodies: never for type-only checks, sketches or speculative contexts. A body publishes when it checks to the end; a failed retry removes its earlier record.
- Dependency bodies are recorded too: `jailint` infers each procedure's effects on its parameters from them (`lifetime_summary.rs`, callee-first over call-graph cycles). Call records keep each argument's type and constant value, and indirect calls through `Allocator.proc` are marked so allocate, resize and free modes can be told apart. Each body also lists its stack slots (owned storage, not escapes) and which blocks return normally.
- Reads that are not uses are skipped in O(1): a bare name as an assignment target or an `==`/`!=` operand is marked on the `FnCtx` before it is checked (`lifetime_nonuses`), so no later pass filters events.
- A struct and its first field share an address, so the lint keys places by address and checked type.
- Resource facts: local, scope and statement ends, by-value parameter copies, `Any` backing places versus spilled temporaries, and returns recorded after cleanup runs (with the value captured before it). A macro `defer` the caller owns is marked definite or conditional. Expanded macro and `#insert` events map back to the caller's source, so findings anchor at the call.
- Element facts: selected subscripts, integer `+`/`-` with constants, pointer offsets with their stride, and `for` entry and advance, so the lint can tell which slot a pointer or cursor names.
- Call targets: procedure addresses and callable results keep their finite set of possible procedures; anything else is Unknown. Runtime `if`/`switch`/`while`/`for` decisions keep their spans so strict findings can print the path.
- To add a fact, record it where the checked operands are known (`FnCtx::lifetime_store` for any store), never by re-reading source text.

## How to change it

- A new lookup: add a `OnceCell` to `Sections` in `span_index.rs`, build it from the record before the mark, return `Found`.
- Prefer a lookup here over a map in a feature; the second copy forgets the sketch mark.
- For cursor positions use `refs_at` / `member_use_at` in `facts.rs`, which match by offset instead of exact span.

Tests: `crates/jaic/tests/ide_span_index.rs`.

## Owned reference queries

`Compiler::ide_reference_index()` exports a `ReferenceIndex` that can answer after the compiler is dropped. For example, `let index = compiler.ide_reference_index();` followed by `index.references(file, offset)` preserves the live query's selection and reference semantics. File and target numbers belong to that exported value; callers must retain its source table and prove input freshness before using positions in a newer document.

The language server owns [snapshot validity and memory pressure](lsp-snapshots.md); the compiler owns symbol resolution, member precedence and overload closure. Extend both the live and owned query in `ide/meta/references.rs`, then compare every cursor in the relevant fixture with `tests/ide_reference_index.rs`.
