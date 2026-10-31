# Parallel type checking

A design for checking procedure bodies on several threads. Sema itself is not parallel yet. What exists: a measurement probe (`sema/body_probe.rs`, `-dev_body_probe`) and the `Send + Sync` conversion of the shared world (`sema/sync_world.rs`).

On large `-check` runs most time is checking bodies (about 60% on a generated 60k-line program), so that is the target.

## What the study found

- About a third to two thirds of bodies grow no shared table, but they hold a small share of the time. The rest add types, procedures, globals or lazily resolve entities, signatures and struct layouts.
- The work is wide, not deep: on the 60k program the critical path of the "body queued another body" tree is about 3% of the total.
- Output depends on lowering order (function numbering and names follow it), so any scheme must commit results in serial order, not completion order.
- Programs dominated by compile-time execution (a build metaprogram) are serial no matter what.

## Design

Speculative bodies, deterministic commit.

1. `drain_bodies` takes the next N queued bodies as a batch. During the batch the `Compiler` is immutable and shared.
2. Each worker checks a body through a view holding `&Compiler` plus a delta (new types, procs, strings, diagnostics) with local id ranges.
3. A body that would run compile-time code, change a namespace, load a module, hit an error or warning, touch a `#placeholder` or emit a message is abandoned and re-run serially. Never guess.
4. The driver commits in queue order: remap local ids to global ones, rewrite the finished IR, merge diagnostics.
5. Settle points (`drain_bodies_lenient`, `expand_all`, metaprogram loops, `#run`) stay serial between batches.

Rejected: locks or concurrent arenas (ids in completion order change IR names run to run), pre-resolving declarations (sema is demand-driven), per-thread `Compiler` copies (memory), splitting check from lowering (one pass).

Rules: batch membership and commit order are the serial order; no thread allocates a global id; diagnostics and IDE facts come from committed bodies only; containers iterated at commit are ordered.

## Expected gain

Realistic target: 1.5-1.8x from sema alone and 2-2.4x with parallel parse, at 1.3-1.6x the CPU. The editor keeps the serial path.

## Plan

0. (done) Audit in-place writes during `lower_body`; make the AST, values and signatures `Send + Sync`.
1. One-thread prototype of the view/delta/commit protocol; go or no-go on commit cost and abort rate.
2. Threads for a restricted subset.
3. Widen: interning, new procs, signatures and layouts resolved locally. Abort rate under 5%.
4. Check unreferenced declarations the same way.

## Send and Sync of the shared world

A worker holding `&Compiler` clones values out of it, and cloning an `Rc` writes a count, so everything a frozen compiler hands out is `Arc`: the syntax tree, source text, `Value`, `Signature`, `Entity`, `Types`, and the state structs. `tree_is_sync` in `sync_world.rs` checks it at compile time.

Still not `Sync`: `Compiler` itself, the file table, `Rc<dyn FileSystem>`, the `RefCell` name caches, the interpreter, and the editor's span cache. Stage 1 splits `Compiler` into a frozen part and a per-thread context.

## Process-wide state

Statics that stay global, and why: the symbol interner and record field names (a `Sym` must mean the same spelling on every thread), AST ids (unique wherever parsed; threads take blocks of 256), interpreter callback slot tables and crash buffers (C and signal handlers can only reach statics), profile totals, the render style, and the allocator's memory-limit counters.

## Other parallel work

- Codegen already runs in units of about 5,000 instructions per thread ([compile speed](compile-speed.md)).
- Skipping IR under `-check` saves little (IR building is 7-12% of `-check`).
- Parse-ahead is the sibling work.

## How to change it

The probe hooks are `lower_body_uncached`, the `body_queue.push` sites, `resolve_entity`, `signature`, `layout_struct_with_runtime_dependencies` and `finish_program`. New shared tables need a delta/commit story before workers can touch them.

## Configuration

- `body-probe` cargo feature (off): also count in-place writes.
- `-dev_body_probe`: print the mutation histogram and queue-tree critical path at the end of `finish_program`.
- `-dev_body_fifo`: lower queued bodies first-in-first-out (study only; output differs).
