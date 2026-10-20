# Parallel type checking

## What it is

A design, with measurements, for checking procedure bodies on several threads. Nothing here is implemented in `sema` itself; the only code is a study probe (`crates/jaic/src/sema/body_probe.rs`, `-dev_body_probe`). Motivation: on large `-check` runs the reference compiler wins only through parallelism (about 1.9 cores to `jaic`'s 1), and about 60% of `jaic -check` time on a generated 60k-line program is `drain_bodies` → `lower_body` → `check_block_stmts`.

## What the study found

Measured with `jaic -check -dev_body_probe` (release build). A body's "own" growth excludes bodies lowered on demand inside it.

| Workload | Bodies | Bodies that grew no shared table ("pure") | Self time in pure bodies | Bodies that ran compile-time code | Queue-tree critical path |
| --- | --- | --- | --- | --- | --- |
| generated 60k lines | 3,153 | 68% | 26% | 0.6% | 4.2 ms of 124 ms (depth 17, 1 root) |
| Focus (`corpus/upstream/focus-editor--focus/first.jai`) | 4,583 | 39% | 4% | 10% (77% of body time: the build metaprogram) | 131 ms of 423 ms (depth 21, 163 roots) |
| 100 `tests/compiler/regressions` programs | 18,859 | 33% | not recorded | not recorded | not recorded |
| hello world | 130 | 34% | 5% | 10% | not recorded |

Two conclusions:

- **Bodies are not independent readers.** About a third to two thirds of bodies are pure by table growth, but they hold a small share of the time. The rest append to shared tables while being checked: types (12% of bodies on the 60k program), procedures and IR functions (25%), globals (16%), strings (3%), and, most of all, lazily resolve entities (24%), signatures (25%) and struct layouts (3%) that nothing resolved yet. "Pure" here only means no table grew; in-place writes (entity state, memo caches such as `module_names`, `lenient_failures`, `local_consts`) are not visible to the probe and must be audited by hand (stage 0).
- **The work is wide, not deep.** On the 60k program the queue tree (body → bodies it caused to be queued) has depth 17 and a critical path of 3% of the total, so parallelism is plentiful. Focus is dominated by one serial thing, the interpreter running the build program.

Order matters for output. `-dev_body_fifo` lowers queued bodies first-in-first-out instead of last-in-first-out; the 60k program still compiles and runs, but the LLVM IR differs from line 19 on (function numbering and names follow lowering order). Any parallel scheme must therefore commit results in the serial order, not in completion order.

## How it works today (what blocks threads)

`Compiler` (`sema/mod.rs`) is one struct; every check method is `&mut self` and touches its fields directly (about 250 direct accesses to `types`, `program`, `entities`, `scopes` and `procs` in `expr.rs`, `stmt.rs` and `calls.rs` alone). Per-body state is already separate: `FnCtx` (`lower.rs`) holds the IR builder, loops, defers and macro frames. Shared state a body reads or writes:

- **Read-mostly, append-only arenas:** `types` (interned `TypeKind` → `TypeId`, `structs`), `entities`, `scopes`, `procs`, `program.funcs/globals/foreigns`, `strings`, `packs`, `const_addr_globals`, `type_infos`, `anonymous_procs`/`anonymous_types`.
- **Lazy resolution in place:** `resolve_entity` (`decls.rs`) flips an entity `Unresolved → Resolving → Done`; `signature` caches `ProcInfo::sig`; `layout_struct` flips `LayoutState`. These are deterministic functions of declarations, but they allocate ids and may run `#run`.
- **Global "something happened" counters** that bodies read for memoisation: `namespace_generation`, `placeholder_misses`, `in_progress_misses`, `ct.executions`, `lower_epoch()`, `lenient_failures`/`parked_bodies`.
- **Compile-time execution:** `interp::Interp` (owned by `Compiler`) with its memory, and `workspaces` shared with metaprograms.
- **Diagnostics and facts:** `warnings`, `ide` facts, `perf`, `dependencies`, `export` (records handed to metaprograms), `last_call_must`, `calls_in_flight`.
- **Non-`Send` types:** about 170 `Rc` uses in `sema`, `Rc<ast::...>` through the AST (25 in `ast.rs`), `Rc<dyn FileSystem>`, `Value::Bytes(Rc<Aggregate>)`. Threads need `Arc` (the parse-ahead work needs the same for the AST) or per-thread copies.

## Design

**Speculative bodies, deterministic commit.** Run body checks as pure functions of a frozen snapshot, collect what each wants to add, and apply the additions serially in the order the serial compiler would have lowered the bodies.

1. **Batches.** `drain_bodies` takes the next N queued bodies (the head of the queue in serial order) as a batch. During the batch the `Compiler` is immutable and shared (`&Compiler`, `Sync`). The main thread waits. No locks on the hot path.
2. **Worker view.** Each worker checks bodies through a `Checker<'a>` that holds `&'a Compiler` plus a *delta* (new types, entities, scopes, procs, functions, globals, strings, diagnostics, a list of lazy facts it computed) and the body's `FnCtx`. All id spaces get a local range (high bit set) so a body can refer to its own additions. This is the large refactor: `&mut self` methods on `Compiler` become methods on a type that reads the base and writes the delta. Start by making the base reads go through accessors (`self.entity(id)` already does) and routing writes through `Delta`.
3. **Abort to serial, never guess.** A body is abandoned (its delta dropped, the body re-run serially in the commit loop) when it would: run compile-time code (`ct.executions`), change a namespace (`#insert`, `#import`, `#load`, `namespace_generation`), load a module, hit any error or warning, touch a `#placeholder`, emit a metaprogram message, or need a lazy resolution the worker cannot do locally (first version: any unresolved entity, signature or layout; the serial thread resolves it and the body is retried in a later batch). Failing bodies always take the serial path, so diagnostics, their order and their notes are byte-identical to today.
4. **Serial commit** in queue order: for each successful body, in order, remap local ids to global ones (append the delta's types through the interner, which dedupes by `TypeKind`; procs through the instance key; strings and packs by content), rewrite the finished IR (`Func`: types, callee ids, global ids, string ids are the only id-carrying operands), assign the pre-reserved `FuncId`, enqueue newly requested bodies in the same order the serial compiler would have, and replay the body's recorded lazy facts. Duplicates made by two bodies of the same batch (both instantiating `f(int)`) collapse at this step: the second commit finds the instance key and remaps to the first.
5. **Settle points stay serial.** `drain_bodies_lenient`, `expand_all`, metaprogram message loops and `#run` execution happen between batches exactly where they do now. A thunk that needs bodies lowered before it runs calls the existing lenient drain, which may itself use batches.

Rejected alternatives:

- *Fine-grained locks / concurrent arenas.* Ids would be handed out in thread-completion order, so IR numbering and names change run to run (see `-dev_body_fifo`). Remapping it back is the same work as the delta design without the lock-free hot path.
- *Pre-resolve every declaration, then check bodies.* Sema is demand-driven and reachability comes from checking bodies (dead-code elimination), so resolving "everything the bodies need" means checking the bodies. The abort-and-resolve loop in step 3 is this idea applied lazily.
- *Per-thread copies of the whole `Compiler`.* Memory and merge cost scale with program size on a 16 GiB machine.
- *Splitting check from lowering.* Expressions lower directly to IR as they check; the two are one pass.

Determinism rules: batch membership and commit order are the serial queue order; remapping is a pure function of (base, delta); no thread ever allocates a global id; diagnostics, warnings, ide facts and `perf` counters come from committed (or serially re-run) bodies only; every nondeterministic container iterated during commit is an ordered one. Test: `JAIC_THREADS=1` and `=N` must give identical `-emit_ir`, and the whole regression corpus is compared in CI.

## Expected gain (Amdahl)

Fractions from the profile of the 60k program (about 150 ms, one thread): bodies 60%, declaration loading incl. parse 32% (parse is the sibling parallel-lex/parse work), rest 8%. From the probe: bodies that must run serially (compile-time code or namespace changes) hold about 5% of body time on this program (far more on metaprogram-heavy ones). Commit cost is an estimate, 10–20% of body work (remap plus merge); it is the number the first prototype must measure.

| Scenario (P = worker threads) | Serial part (of original time) | Parallel part | Speedup |
| --- | --- | --- | --- |
| Bodies only, commit 20%, P=4 | 0.40 + 0.03 + 0.12 | 0.57 / 4 | about 1.45x |
| Bodies only, commit 10%, P=4 | 0.40 + 0.03 + 0.06 | 0.57 / 4 | about 1.6x |
| Bodies only, commit 10%, P=8 | same | 0.57 / 8 | about 1.8x |
| Plus parallel parse (parse ≈ 0.20 of the 0.40, at 4 threads), commit 10%, P=4 | 0.25 + 0.03 + 0.06 | 0.57 / 4 | about 2.1x |
| Same, P=8 | same | 0.57 / 8 | about 2.4x |

So the realistic target is about 1.5–1.8x from sema alone and 2–2.4x with parallel parse, at roughly 1.3–1.6x the total CPU (batch overhead, aborted attempts, duplicate work). That is comparable to the reference's 1.9 cores on top of `jaic`'s lower single-thread cost. Programs dominated by compile-time execution (Focus: most body time is the interpreter) get nothing from this; small programs (hello: 130 bodies, 4 ms) must not pay for it, so the pool starts only when a batch is large enough (suggest ≥ 64 bodies).

## Risks

- **The refactor is wide.** `&mut Compiler` is the signature of nearly all of sema. Mitigation: introduce the delta/view split behind a trait or a second struct while the serial path keeps using the real `Compiler`, and move modules over one at a time, `expr.rs`/`stmt.rs`/`calls.rs` last.
- **In-place memo writes** (not seen by the probe) can make a speculative body depend on, or corrupt, base state. Stage 0 audits them; each becomes delta-local, commit-replayed, or an abort reason.
- **Abort rate.** 24–25% of bodies on the 60k program trigger a lazy resolution; if each aborts once the wasted work is bounded (retried in the next batch) but real. Mitigation: let workers resolve non-`#run` entities and signatures locally and commit them idempotently (stage 3).
- **`Arc` conversion** costs a few percent single-threaded (atomic refcounts). Mitigation: only the shared base needs `Arc`; per-body values can stay `Rc` inside a worker if the delta is converted at commit.
- **Metaprograms** observing typechecked procedures see them in commit order, which is serial order; they never run mid-batch.
- **Memory:** deltas hold a batch's worth of IR; bound the batch size.
- **Editor builds** (`ide` facts, `ParseReuse`) keep the serial path; parallel checking is for `jaic` only.

## Recommended plan

| Stage | Work | Effort | Output |
| --- | --- | --- | --- |
| 0 | Audit in-place writes during `lower_body` (extend the probe with a `&mut` audit: wrap the memo maps in a type that counts writes); decide for each: delta, replay or abort. Make AST, `Value`, `Signature`, `ProcLit` `Send + Sync` (shared with the parse-ahead work). | 3–5 days | A written table of every shared write; `Compiler: Sync` for the immutable phase. |
| 1 | Prototype the protocol with one thread: a `Checker` view over the same `Compiler` for the simplest bodies (no calls to unresolved procs, no types created), delta + commit + remap, behind `JAIC_THREADS`. Measure commit cost and abort rate on the 60k program. Go/no-go on the commit-cost estimate here. | 1–2 weeks | Commit cost per body and abort rate, with identical `-emit_ir`. |
| 2 | Add threads for the restricted subset; batches, pool, ordered commit. | 3–5 days | First wall-clock gain on 60k. |
| 3 | Widen: type and string interning in the delta, new procs (instance keys), signatures/entities/layouts resolved locally, globals. Abort rate target under 5%. | 3–4 weeks | Most bodies speculative. |
| 4 | Check the unreferenced declarations after `finish_program` the same way (the program's own unused bodies are independent); tune batch size and pool start threshold. | 1 week | Remaining gain. |

What to change first: stage 0's `Send + Sync` conversion (it unblocks the parse work too) and the write audit. Cheaper wins that stand alone and do not depend on any of this: fewer allocations in `lookup_full` (25% inclusive, 4.6% in malloc itself; a per-scope name cache keyed by `namespace_generation` is the likely fix) and mimalloc (about 8%).

## Status of stage 0 and 1

Nothing of stage 0 or 1 is done yet (no `Send + Sync` conversion, no context split). Name lookup now writes four generation-keyed memo tables in place during body checking: `module_names`, `module_misses`, `scope_misses` and `using_member_free` (all in `Compiler`, keyed by `namespace_generation`). They belong in the stage 0 write audit: each is a pure cache of the declarations, so a worker can keep a private copy or share a read-only one per batch. Body scopes (`Proc`, `Block`, `Macro`) are deliberately not cached because they gain `using`s without moving the generation.

## Process-wide state

The statics and thread-locals in `jaic`, with why each stays. Anything not listed is a lazily initialised read-only value (`OnceLock` of a knob, a feature probe, a table built from constants).

| State | Where | Why it is global |
| --- | --- | --- |
| Callback slot tables | `interp/native/callbacks.rs`, `callbacks/win64.rs` | A C-callable thunk is a fixed code address that C calls from any thread; a static is the only state it can reach. One table per calling convention, never both live. |
| Profile totals | `interp/profile.rs` (`TOTALS`, one lock) | Every interpreter (the main compile and each metaprogram workspace) merges into one report when it is flushed or the process exits. |
| Render style | `render.rs` (`STYLE`, with a thread-local `with_style` override) | Diagnostics are rendered where no options value reaches; a tool sets it once in `main`. |
| Crash report buffer, signal handlers | `interp/crash.rs` | A signal handler can only reach a static, and must not allocate. |
| Memory limit counters | `memory_limit.rs` | They are the global allocator's counters. |
| Symbol interner | `intern.rs` (`TABLE`, `SHARD`, `NEXT`, plus a thread-local cache) | A `Sym` must mean the same spelling on every thread: parse workers, the checker and threads a `#run` starts. Append-only segments make `as_str` lock-free; new spellings take one of 64 shard locks. |
| Record field names | `records.rs` (`NAMES`, one `RwLock`, plus a thread-local cache) | A metaprogram's records can be made on any thread a `#run` starts, so a `Name` must mean the same field everywhere. The set is small; after warm-up every lookup hits the thread-local cache. |
| AST ids | `ast.rs` (`NEXT_AST_BLOCK`, plus a thread-local block of 256) | A node's id must be unique wherever it was parsed, so a worker's tree needs no renumbering. A thread touches the atomic once per 256 ids. |

## Other things that could run in parallel

- **Codegen** already does: `jaic-llvm` splits into units of 5,000 instructions on one thread each (`docs/compiler/compile-speed.md`); `-check` has no codegen. Overlapping codegen with the front end was tried and rejected there.
- **Lowering finished bodies to IR** is not a separate step (checking lowers directly), so there is nothing to split off.
- **Parse and load**: the sibling parse-ahead work.
- **Unreferenced-declaration checking** after the reachable program: same machinery as above (stage 4), and the least entangled with the metaprogram.

## How to change it

The probe is `crates/jaic/src/sema/body_probe.rs`; hook points: `lower_body_uncached` (begin/end), the two `body_queue.push` sites, `resolve_entity`, `signature`, `layout_struct_with_runtime_dependencies`, and `finish_program` (report). New shared table in `Compiler`? Add a counter to `snapshot` in the probe so the study stays honest. Remove `next_body` and the FIFO switch when the study is done; it exists only to show order sensitivity.

## Configuration

- `-dev_body_probe`: print the per-body mutation histogram and queue-tree critical path to stderr at the end of `finish_program`.
- `-dev_body_fifo`: lower queued bodies first-in-first-out (study only; output differs from the default).

## Dependencies

`sema` (`procs.rs`, `scope.rs`, `decls.rs`, `structs.rs`), the interner and parse cache from the parallel lex/parse work, `ir::Program`, `interp` (stays serial).
