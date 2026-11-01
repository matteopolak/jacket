# Parallel type checking

A design for checking procedure bodies on several threads. Sema itself is not parallel yet. What exists: a measurement probe (`sema/body_probe.rs`, `-dev_body_probe`), the `Send + Sync` conversion of the shared world (`sema/sync_world.rs`), and a one-thread prototype of the batch/abandon/commit protocol (`sema/speculative.rs`, [stage 1](#stage-1-one-thread-prototype)).

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
1. (done) One-thread prototype of the view/delta/commit protocol; go or no-go on commit cost and abort rate. Result: go, see [stage 1](#stage-1-one-thread-prototype).
2. Threads for a restricted subset.
3. Widen: interning, new procs, signatures and layouts resolved locally. Abort rate under 5%.
4. Check unreferenced declarations the same way.

## Stage 1: one-thread prototype

Built with the `speculative-bodies` cargo feature (off; without it every hook compiles away) and switched on with `-dev_speculative_bodies` or `JAIC_SPECULATIVE_BODIES=1`. A build without the feature ignores the flag with a note on stderr. `-dev_body_probe` prints the counters.

### How it works

`drain_bodies` calls `drain_speculatively` instead of the plain pop loop. It looks at the top 64 queued bodies (they stay queued) and checks each one speculatively, in order, against the world as it is. The real `Compiler` is the view: the shared tables are wrapped (`Probed` in `body_probe.rs`), and a write to one that a body may not change abandons it by unwinding (`speculate_body` catches it). Whatever a body did before then is undone.

A body that finishes leaves a `Delta`: the finished `ir::Func`, its constants, the scopes and entities it made (with ids past the end of the tables), the string and constant globals it made (provisional ids), the types of its expressions, and a list of deferred operations (record a call instance, a local declaration, and so on). When the body's turn comes, `commit_body` shifts the new scope and entity ids to the end of the tables, creates the globals in the order the serial path would, rewrites the `GlobalAddr` instructions and relocations, replays the deferred operations and stores the function.

A body is committed only if it finished and a stamp (namespace generation, files, modules, compile-time runs, sources) is unchanged since it was checked. Otherwise it runs on the normal serial path. The queue is only ever popped one body at a time, so a serial body that drains it (compile-time code) sees the same queue as without the flag. That mattered: taking batches off the queue first changed the order globals were created in on a large program.

What is allowed during a body: reads; appends to the scope and entity tables; read-before-write interning of types that already exist; memo caches; an editor that is idle. Everything else on the abort list (a new type, a new procedure, a first-use entity, a thunk scope, an error or warning, compile-time execution, a message, `#placeholder`, an editor that is live, a body inside a body) abandons the body and is counted by the first table it wrote.

In debug builds `spec_fingerprint` hashes 54 plain fields of the shared state before and after every speculative body and asserts they are equal, which is how a missed write shows up.

### Measured

`-check`, release build, one thread. "Committed" is the share of batched bodies that were committed instead of abandoned.

| Workload | Bodies batched | Committed | Abandoned, top blockers | Commit cost | View path, committed / abandoned | Serial path |
| --- | --- | --- | --- | --- | --- | --- |
| `examples/game2d/game` | 1,810 | 25% | first-use entity 40%, thunk scopes 7%, serial body invalidated 7%, new types 5% | 0.74 us/body | 7.3 / 50 us | 36 us |
| `examples/ui-gallery` | 694 | 22% | first-use entity 37%, resolving depth 9%, invalidated 7%, `out.program` 6% | 0.54 us/body | 7.8 / 47 us | 36 us |
| ~48 stdlib modules (`-no_dce`) | 1,964 | 46% | procedure table 14%, resolving depth 10%, thunk scopes 7%, `out.program` 6%, perf counters 6% | 0.57 us/body | 6.7 / 48 us | 31 us |
| 600k generated lines | 30,346 | 73% | first-use entity 22%, perf counters 2%, invalidated 2% | 0.77 us/body | 9.2 / 55 us | 48 us |

`-check` wall time with the flag on is slower than off (600k lines: 0.71s off, 1.2s on; game2d: 0.10s and 0.17s). That is the cost of checking an abandoned body twice, and is expected on one thread; the prototype measures the protocol, not a speedup.

### Identical output

With the flag on, diagnostics and `-emit_ir` are byte-identical to the flag off on `game2d`, `ui-gallery`, a program importing about 48 stdlib modules, and the 600k-line program (349 MB of IR), apart from `@frozen.*` globals, which hold heap addresses. The `jaic` unit tests and the `jaic-cli` test binaries that compile programs pass with `JAIC_SPECULATIVE_BODIES=1`.

An unrelated finding: importing `Print_Vars` makes the order of the `type_info` globals in `-emit_ir` vary from run to run, with or without the flag.

### Go or no-go

Go. The protocol costs under 1 us per committed body to commit and 3-9 us to check, against 30-50 us for a body on the serial path, so commit does not bound the speedup. The abort rate is high on real programs (55-78% abandoned) but concentrated: a handful of tables account for nearly all of it, and each is a table stage 3 already plans to make local. The numbers favour a program with many similar bodies; small programs commit under half.

### What stage 2 has to widen first

Order by share of abandoned bodies:

1. First-use entity resolution (`ns.entities`): 22-40% everywhere. A body that is the first to name a global, a type or a procedure resolves it in place. Resolving it ahead of the batch, or resolving it into the delta, is the biggest single step.
2. Resolution bookkeeping (`epochs.resolving_depth`, `ns.thunk_scopes`): 7-17% on small programs, from constants evaluated while checking a body.
3. New procedures, signatures and types (`pr.procs`, `ty.types`, `ty.default_images`): up to 14% on library-heavy code.
4. `out.program` and `meta.perf`: counters and program-level tables written as a side effect; they need to move into the delta.

Compile-time execution stays serial.

### What is left

Threads; a read-only view type in place of write interception on the whole `Compiler` (the per-thread caches and the non-`Sync` fields listed below go with it); batches sized by the work and not a fixed 64; and the three tables above.

## Send and Sync of the shared world

A worker holding `&Compiler` clones values out of it, and cloning an `Rc` writes a count, so everything a frozen compiler hands out is `Arc`: the syntax tree, source text, `Value`, `Signature`, `Entity`, `Types`, and the state structs. `tree_is_sync` in `sync_world.rs` checks it at compile time.

Still not `Sync`: `Compiler` itself, the file table, `Rc<dyn FileSystem>`, the `RefCell` name caches, the interpreter, and the editor's span cache. Stage 2 splits `Compiler` into a frozen part and a per-thread context; the stage 1 prototype still uses the whole `Compiler` as the view.

## Process-wide state

Statics that stay global, and why: the symbol interner and record field names (a `Sym` must mean the same spelling on every thread), AST ids (unique wherever parsed; threads take blocks of 256), interpreter callback slot tables and crash buffers (C and signal handlers can only reach statics), profile totals, the render style, and the allocator's memory-limit counters.

## Other parallel work

- Codegen already runs in units of about 5,000 instructions per thread ([compile speed](compile-speed.md)).
- Skipping IR under `-check` saves little (IR building is 7-12% of `-check`).
- Parse-ahead is the sibling work.

## How to change it

The probe hooks are `lower_body_uncached`, the `body_queue.push` sites, `resolve_entity`, `signature`, `layout_struct_with_runtime_dependencies` and `finish_program`. New shared tables need a delta/commit story before workers can touch them.

## Configuration

- `speculative-bodies` cargo feature (off): build the stage 1 prototype.
- `-dev_speculative_bodies` / `JAIC_SPECULATIVE_BODIES=1`: check bodies speculatively (needs the feature; off by default; the environment variable is also how the test suites force it on).
- `body-probe` cargo feature (off): also count in-place writes.
- `-dev_body_probe`: print the mutation histogram and queue-tree critical path at the end of `finish_program`.
- `-dev_body_fifo`: lower queued bodies first-in-first-out (study only; output differs).
