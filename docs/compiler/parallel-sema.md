# Parallel type checking

Sema checks procedure bodies on one thread. Checking them on several was designed and shelved: the expected gain is 1.1x to 1.3x on type checking, under 10% of a debug build, which does not pay for the risk. Codegen and link are where build time goes. Revisit it only if large programs make checking dominate.

What exists is experimental tooling, all off by default:

| Piece | Where | What it does |
| --- | --- | --- |
| Body probe | `sema/body_probe.rs`, `-dev_body_probe`, cargo feature `body-probe` | Counts which shared tables each procedure body writes while it is lowered, and prints a histogram when the compile ends. |
| `Send + Sync` check | `sema/sync_world.rs` | Compile-time proof that what a frozen `&Compiler` hands to workers (syntax tree, `Value`, `Signature`, `Entity`, `Types`) is `Arc`-based and `Sync`. A `Cell`, `RefCell` or `Rc` added to one of those stops compiling. |
| Speculative bodies | `sema/speculative.rs`, `-dev_speculative_bodies` or `JAIC_SPECULATIVE_BODIES=1`, cargo feature `speculative-bodies` | A one-thread prototype that checks a batch of queued bodies, abandons any that write shared state beyond the scopes and entities they own, and commits the rest in queue order. Without the feature the flag only prints a notice. |

The prototype shows the commit protocol keeps output identical. It also shows what stops bodies from running speculatively: many add a type, a procedure, a global or a lazily resolved entity.

## Constraints a design has to meet

- Work is wide, not deep, so bodies are the right unit. Programs dominated by compile-time execution stay serial.
- Function and global ids, and the symbol names built from them (`name.<id>`), follow lowering order and reach binaries, backtraces, codegen-unit assignment and `Stack_Trace_Node.hash`. Completion-order numbering needs a stable key (declaration order, instance bindings) for every place an id's order is visible, or a canonicalizing renumber pass before codegen.
- A query a body acts on must give its final answer or make the body wait. Today's `EntityState::Resolving` and `LayoutState::InProgress` answers are errors, which is right only on the same thread.
- A body must be abandonable and rerunnable with the same result, so every write to shared tables is an insert of something content-addressed or claimed.
- Process-wide state stays global: the symbol interner and record field names (a `Sym` must mean the same spelling everywhere), AST ids (threads take blocks of 256), interpreter callback tables and crash buffers, profile totals, the render style and the allocator's memory-limit counters.
- `JAIC_SYM_SCRAMBLE` and `JAIC_ID_SCRAMBLE` (symbol and AST ids) find places where id order leaks into output ([architecture](architecture.md#threading)). Sema ids have no such knob.

## How to change it

Keep `Probed` table writes inside the hooks in `body_probe.rs`; a new shared table the speculative path must intercept needs a `Probed` wrapper. A new field in the frozen world must pass `sync_world.rs`.

## Dependencies

`crates/jaic/Cargo.toml` features `body-probe` and `speculative-bodies`.
