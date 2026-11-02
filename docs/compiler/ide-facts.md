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

With lint facts on (`IdeFacts::lint`), each checked body also records a compact event list for `pointer_lifetime` (`ide/lifetimes.rs`, `IdeFacts::lifetime_bodies`, per file through `lifetime_bodies_in`). It holds per-instance local identities, array-element derivations, stores, selected callees with argument-to-parameter mapping, `#if` choices, macro and `#insert` output, and the final control edges from IR lowering (branches, switch, labelled jumps, short circuits, `defer` cleanup). No IR or second syntax tree is kept.

- Recorded only for real bodies: never for type-only checks, sketches or speculative contexts. A body publishes when it checks to the end; a failed retry removes its earlier record.
- Dependency bodies are recorded too, so callers can later use their effects.
- Reads that are not uses are skipped in O(1): a bare name as an assignment target or an `==`/`!=` operand is marked on the `FnCtx` before it is checked (`lifetime_nonuses`), so no later pass filters events.
- To add a fact, record it where the checked operands are known (`FnCtx::lifetime_store` for any store), never by re-reading source text.

## How to change it

- A new lookup: add a `OnceCell` to `Sections` in `span_index.rs`, build it from the record before the mark, return `Found`.
- Prefer a lookup here over a map in a feature; the second copy forgets the sketch mark.
- For cursor positions use `refs_at` / `member_use_at` in `facts.rs`, which match by offset instead of exact span.

Tests: `crates/jaic/tests/ide_span_index.rs`.
