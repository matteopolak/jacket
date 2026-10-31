# Parallel type checking

## What it is

A design, with measurements, for checking procedure bodies on several threads. Nothing here is implemented in `sema` itself. The code so far is a study probe (`crates/jaic/src/sema/body_probe.rs`, `-dev_body_probe`) and the `Send + Sync` conversion of the shared world (`crates/jaic/src/sema/sync_world.rs`). Motivation: on large `-check` runs the reference compiler wins only through parallelism (about 1.9 cores to `jaic`'s 1), and about 60% of `jaic -check` time on a generated 60k-line program is `drain_bodies` → `lower_body` → `check_block_stmts`.

## What the study found

Measured with `jaic -check -dev_body_probe` (release build). A body's "own" growth excludes bodies lowered on demand inside it.

| Workload | Bodies | Bodies that grew no shared table ("pure") | Self time in pure bodies | Bodies that ran compile-time code | Queue-tree critical path |
| --- | --- | --- | --- | --- | --- |
| generated 60k lines | 3,153 | 68% | 26% | 0.6% | 4.2 ms of 124 ms (depth 17, 1 root) |
| Focus (`corpus/upstream/focus-editor--focus/first.jai`) | 4,583 | 39% | 4% | 10% (77% of body time: the build metaprogram) | 131 ms of 423 ms (depth 21, 163 roots) |
| 100 `tests/compiler/regressions` programs | 18,859 | 33% | not recorded | not recorded | not recorded |
| hello world | 130 | 34% | 5% | 10% | not recorded |

Two conclusions:

- **Bodies are not independent readers.** About a third to two thirds of bodies are pure by table growth, but they hold a small share of the time. The rest append to shared tables while being checked: types (12% of bodies on the 60k program), procedures and IR functions (25%), globals (16%), strings (3%), and, most of all, lazily resolve entities (24%), signatures (25%) and struct layouts (3%) that nothing resolved yet. "Pure" here only means no table grew; the in-place writes (entity state, memo caches such as `module_names`, `lenient_failures`, `local_consts`) are counted separately and decided one by one in the stage 0 audit below.
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
| 0 (done) | Audit in-place writes during `lower_body` (extend the probe with a `&mut` audit: wrap the memo maps in a type that counts writes); decide for each: delta, replay or abort. Make AST, `Value`, `Signature`, `ProcLit` `Send + Sync` (shared with the parse-ahead work). | 3–5 days | A written table of every shared write; `Compiler: Sync` for the immutable phase. |
| 1 | Prototype the protocol with one thread: a `Checker` view over the same `Compiler` for the simplest bodies (no calls to unresolved procs, no types created), delta + commit + remap, behind `JAIC_THREADS`. Measure commit cost and abort rate on the 60k program. Go/no-go on the commit-cost estimate here. | 1–2 weeks | Commit cost per body and abort rate, with identical `-emit_ir`. |
| 2 | Add threads for the restricted subset; batches, pool, ordered commit. | 3–5 days | First wall-clock gain on 60k. |
| 3 | Widen: type and string interning in the delta, new procs (instance keys), signatures/entities/layouts resolved locally, globals. Abort rate target under 5%. | 3–4 weeks | Most bodies speculative. |
| 4 | Check the unreferenced declarations after `finish_program` the same way (the program's own unused bodies are independent); tune batch size and pool start threshold. | 1 week | Remaining gain. |

What to change first: stage 0's `Send + Sync` conversion (it unblocks the parse work too) and the write audit. Cheaper wins that stand alone and do not depend on any of this: fewer allocations in `lookup_full` (25% inclusive, 4.6% in malloc itself; a per-scope name cache keyed by `namespace_generation` is the likely fix) and mimalloc (about 8%).

## Stage 0: audit of in-place writes during `lower_body`

With the `body-probe` feature (`cargo build --release -p jaic-cli --features body-probe`), `-dev_body_probe` counts every mutable borrow of shared compiler state made while a body is lowered, not only table growth. The shared tables of `Compiler` (`pr`, `ty`, `ns`, `out`, `src`, `caches`, `diag`, `ct`, `meta`) are wrapped in `Probed<T, W>` (`body_probe.rs`): reads go through `Deref`, anything that takes `&mut` goes through `DerefMut` and counts one write for that state. Plain fields (`namespace_generation`, `in_progress_misses`, `ct.executions`, `last_call_must`, ...) call `body_probe::write` where they change. The counts are an upper bound: a `&mut` that ends up changing nothing still counts (type interning counts every constructor call, hit or miss; `Editor` counts every fact call it then ignores). Writes made by bodies lowered on demand inside a body are charged to those bodies. Indexing the procedure or entity table for assignment also notes whether the element is older than the body (`pr.procs[i]` for another body's procedure, `ns.entities[i]` for an entity made before the body began), and `scope_mut` notes writes to root, module, file and struct scopes. Those are the writes that a second reader of the frozen world could observe. Without the feature `Probed` is a transparent wrapper and the hooks compile to nothing, so the perf gate sees no cost.

Measured with a release `jaic` built with that feature, `-check -dev_body_probe -serial_parse -threads 1` on the generated 60k program (3,217 bodies), the first 100 programs of `tests/compiler/regressions` (17,326 bodies) and Focus through the `tools/bench` sandbox (4,913 bodies). "Bodies" is the share of bodies whose own lowering wrote the state at least once, as 60k / regressions / Focus; "writes" is the total on the 60k program.

Decisions:

- **local**: a worker-local delta. The worker owns the value, new ids come from a local range, the commit appends it.
- **copy**: a private copy per worker of a pure cache.
- **replay**: the delta records the write and the commit applies it in queue order.
- **abort**: the body is dropped and run serially.

| State | Written by | Bodies (%) | Writes | Decision | Why |
| --- | --- | --- | --- | --- | --- |
| `pr.procs[own]` (`body_state`, `sig`) | `lower_body_uncached`, `signature` | 100 / 100 / 100 | 29,383 | local | The body's own record; the commit sets `Done` and the cached signature. |
| `pr.lenient_failures` (remove own) | `lower_body_uncached` | 100 / 100 / 100 | 3,217 | replay | One keyed remove per successful body, in any order. |
| `pr.lowering_depth` | `lower_body_uncached` | 100 / 100 / 100 | 6,434 | local | Nesting counter of one body; each worker starts at zero. |
| `pr.last_call_must` | `check_call`, statements | 84 / 86 / 83 | 22,313 | local | Per-body scratch that lives on `Compiler` by accident; moves into the body context. |
| `diag.dependencies` (`begin_body`, `end_body`) | `lower_body_uncached` | 100 / 100 / 100 | 6,446 | replay | The edges a body records are consumed in order; buffer per body, append at commit. |
| `meta.export.expression_types[own]` | `check_expr` | 100 / 100 / 100 | 257,408 | local | Keyed by the body's id; merged at commit. |
| `meta.export.call_instances` | `calls/matching.rs` | not separated | | replay | Keyed by call span; appended in queue order. |
| `meta.perf` | `perf.rs` counters | 99 / 98 / 92 | 9,196 | local | Per-worker counters summed at commit. |
| `ed` (editor facts) | every `ide` recording call | 100 / 98 / 100 | 791,462 | local | `jaic` is not live, so each call returns at once and the `&mut` is its only cost. Editor builds keep the serial path. |
| `out.program` (function bodies, globals) | end of `lower_body_code`, constants | 100 / 100 / 100 | 8,794 | local | The body's `Func` is filled once; new globals get local ids that the commit remaps. |
| `ns.scopes` (own `Proc`, `Block`, `Macro` scopes) | `new_scope`, `bind_name` | 100 / 100 / 100 | 47,749 | local | Scopes a body creates and fills are its own; ids come from a local range. |
| `ns.entities` (appends: locals, parameters) | `new_entity` | 100 / 96 / 98 | 34,934 | local | Same, appended at commit. |
| `ns.entities[i]`, older than the body; `epochs.resolving_depth` | `resolve_entity` (`Unresolved` to `Done`) | 25 / 46 / 16; 25 / 47 / 23 | 7,693; 9,776 | abort | First version: a body that needs an unresolved entity is retried after the serial thread resolved it. Stage 3 resolves non-`#run` entities locally and commits them idempotently. |
| `pr.procs[i]`, another body's procedure | `signature`, instantiation, calls | 26 / 45 / 40 | 19,465 | abort | Same lazy resolution for a procedure. Stage 3 gives a callee a local proc range plus the instance key. |
| `pr.body_queue`, `pr.func_targets`, `pr.func_procs` | a call to a procedure that has no body yet | 25 / 42 / 39 | 3,208 each | replay | The commit enqueues in the order the serial compiler would have. |
| `ty.types` (intern) | every type constructor | 51 / 72 / 69 | 11,255 | local | A hit is a lookup that borrows `&mut`, so it needs a read-only path. A miss (13% of the 60k bodies) goes to a delta and is remapped through the interner at commit. |
| `out.strings`, `out.packs`, `out.const_addr_globals` | string and constant literals | 4 / 24 / 12 | 453 | local | Interned by content; the commit dedupes. |
| `caches.module_names` | `module_declarations` | 27 / 58 / 56 | 25,086 | copy | Pure cache of the declarations, stamped with `namespace_generation`. A worker starts each batch with empty copies; a miss only costs a recompute. |
| `caches.module_misses` | `module_lookup` | 26 / 55 / 54 | 24,773 | copy | Same. |
| `caches.scope_misses` | `lookup_full` | 27 / 72 / 65 | 9,754 | copy | Same. |
| `caches.using_member_free` | `using` member search | 1 / 6 / 7 | 3,289 | copy | Same. |
| `ns.reexport_visiting` | `module_declarations` | 27 / 56 / 55 | 49,842 | copy | A search stack that is empty between calls. |
| `pr.local_decl_types` | local declarations | 87 / 63 / 59 | 11,067 | replay | Keyed by AST id and name index; an idempotent insert. |
| `pr.local_consts`, `pr.const_macro_params` | constants and macro parameters | 1 / 22 / 13 | 63, 62 | replay | Keyed by the body's own entities. |
| `pr.first_arguments`, `pr.proc_decl_notes`, `pr.ifx_operands` | calls, declaration notes, `ifx` | 1 / 22 / 14; 1 / 15 / 6; 0 / 3 / 0 | 54, 69, 4 | replay | Keyed by span or procedure; idempotent. |
| `ns.thunk_scopes` | constant evaluation | 10 / 28 / 35 | 1,862 | replay | Maps a scope to its thunk scope; the new scope is local and the entry is replayed. |
| `ns.overload_sets_checked` | overload resolution | 1 / 11 / 3 | 32 | replay | A set insert that gates a one-time warning; the warning is replayed in order. |
| Struct state: `ty.struct_asts`, `field_declarations`, `field_types`, `field_type_pending`, `field_using_order`, `default_images`, `pr.callable_fields`, `ns.scopes[i]` of a struct, layout in `ty.types` | first use of a struct (`layout_struct`) | 3 / 10 / 3 | 691 to 5,540 | abort | Lazy resolution of a declaration, like an entity. Stage 3 makes it local and idempotent. |
| Enum and nominal state: `ty.enum_member_scopes`, `enum_member_entities`, `enum_base_spans`, `nominal_origins` | first use of an enum or nominal type | 2 / 5 / 1 | 17 to 282 | abort | Same. |
| Type information: `ty.snapshot_infos`, `infos`, `failed_infos`, `building_infos`, `default_globals`, `initializers`, `generic_struct_infos`, `deferred_infos` | `type_info`, defaults, `get_type_table` | 21 / 14 / 16 for `snapshot_infos`; 0 to 3 for the rest | 2,627; 1 to 302 | abort | Builds globals and functions in several steps with state kept across calls. |
| `ct.interp`, `ct.executions`, `ct.frozen` | constant evaluation, `#run`, `#assert` | 1.5 / 15 / 13; 1.4 / 12 / 11; 0.1 / 0.1 / 0.3 | 3,386; 207; 38 | abort | Runs compile-time code on the one interpreter and its memory: the serial section. |
| `epochs.namespace_generation`, `src.sources`, `src.insert_depth`, `src.insert_sites`, `ns.libraries`, `ns.default_scopes`, `pr.codes`, `pr.code_scopes`, `pr.anonymous_procs`, `pr.callable_entities` | `#insert`, `#import`, `#load`, `#library`, quoted code, anonymous procedures | 1 / 3 / 5 for `namespace_generation`; 0 to 1 / 0 to 3 / 0 to 6 for the rest | 620; at most 3 each | abort | A body that changes what names mean invalidates every cache another body used. |
| `epochs.in_progress_misses`, `epochs.placeholder_misses`, `epochs.mapping_imports` | failed lookups that wait on unfinished work | 0 / 0.1 / 0 | 0 | abort | A failure that reads these is not final; the body is retried serially. |
| `diag.warnings`, `diag.deferred_errors`, `caches.declaring_imports`, `caches.stdlib_view`, `src.library_files` | diagnostics and their suggestions | 0 / 0 / 0.1 | 0 | abort | Any diagnostic aborts, so messages, their order and their notes stay byte-identical. |
| Everything else in the shared tables | | 0 in all three runs | 0 | local if it ever fires | `pr.provisional_errors`, `auto_bake_variants`, `parked_bodies`, `multi_consts`, `code_procs`, `live_procs`, `exports`, `calls_in_flight`, `discard_params`, `interpolated`, `ty.variadic_cases`, `cases_building`, `info_flags`, `table_funcs`, `member_aliases`, `using_filters`, `ns.modules`, `module_cache`, `module_failures`, `add_contexts`, `pokes`, `deferred_pending`, `unsettled`, `src.files`, `file_by_path`, `provided_texts`, `canon_cache`, `ahead`, `ct.top_level_runs`, `asserts`, `twin`. A few change in the rare body that loads a module or runs a `#run` (`ns.modules`, `src.files`, `ns.unsettled` in 2 of the 100 regression programs); those bodies already abort on `namespace_generation` or `ct.executions`. |

Reading the table:

- Most writes go to the body's own record, scopes, entities and per-body maps. They need a local id range and a commit step, nothing else.
- What sets the abort rate is lazy resolution: older entities, other bodies' procedures, struct and enum state. It touches a quarter of the bodies on the 60k program and up to 46% on the regressions, so stage 1 measures the abort rate with these aborting and stage 3 widens.
- The four name-lookup caches and the re-export search stack are the most frequent shared writes after the per-body ones, and a private copy costs nothing to keep correct.
- `lower_epoch()` only reads (`src.added`, `src.files`, `ns.modules`, `ct.runs_done`, `namespace_generation`); it needs no decision of its own.
- Reference counts are writes too. Cloning an `Rc<ProcLit>`, `Rc<Signature>` or a `Value` on a worker writes memory that other workers read, so the types a frozen `&Compiler` holds have to be `Arc` (the next section).

## Send and Sync of the shared world

A worker holding `&Compiler` clones values out of it, and a clone of an `Rc` writes a count that other workers read. So everything a frozen compiler hands out is `Arc`, and `crates/jaic/src/sema/sync_world.rs` (plus `tree_is_sync` at the end of `jaic-syntax/src/ast.rs`) asserts `Send + Sync` for each type at compile time. A `Cell`, `RefCell` or `Rc` added to one of them fails the build there.

Now `Send + Sync`:

- The syntax tree: `File`, `Stmt`, `Expr`, `Decl`, `ProcLit`, `ProcHeader`, `StructLit`, `EnumLit`, `CodeBody`, `RunBody`, `AsmBlock`, `Import`, `Case`, `Note`. Shared nodes (`Rc<ProcHeader>`, `Rc<Decl>`, `Rc<StructLit>`, and the like) are `Arc`.
- Source text and docs: `SourceFile::text` is `Arc<str>`, as are doc comment text and the IDE text copies.
- `Value`, `Aggregate`, `Signature`, `ParamInfo`, `ProcTypeParams`, `ProcInfo`, `Scope`, `Entity`, `Types`, `TypeKind`.
- The state structs `Namespaces`, `ProcState`, `TypeState` and `Output`.

Per-body values that never leave the thread that checks the body stay `Rc` (the cleanup marks in `lower.rs`, interpreter values, the workspace registry `SharedWorkspaces`). They are not part of the world.

Not yet, and why:

- `Compiler` itself. It still holds the pieces below.
- `SourceMap` / `FileTable`: a `Cell`-based file table behind an `Rc`.
- `SourceState`: `Rc<dyn FileSystem>`.
- `NameCaches`: `RefCell` memo tables (`declaring_imports`, `stdlib_view`) and `library_files`. These are the private-copy-per-worker rows in the audit table.
- `CompileTime`: the interpreter (`ValueStack` raw pointers, `ZeroedBlock`) and the build workspaces. It stays on the main thread; bodies that touch it abort to serial.
- `ExpressionTypes`: a `RefCell` span cache the editor reads.
- The `Probed` wrappers on the remaining state structs are plain data, so they are `Sync` once their contents are; the blockers are the fields above.

Stage 1 splits `Compiler` into a frozen part (the types above) and a per-thread context that holds the rest.

`parse_ahead.rs` no longer needs its `Handoff` wrapper with an `unsafe impl Send`: a tree is `Send` on its own. The relocation pass stays, because it rewrites the file id inside every span, not because of reference counts; `relocate` still takes `&mut` access through the `Arc`s, which is sound while one thread owns the tree.

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
- **Skipping IR under `-check`** (a builder that records nothing for bodies no compile-time code runs) is not worth it yet. Building IR is about 7–12% of `-check` time (gen-60k, Focus), and a crude no-op builder saved 3% wall and 9% peak RSS on gen-60k. The checker still needs the builder's values, terminator state and callee lists: reachability in `procs/drain.rs` walks the emitted calls. A body that a later `#run` needs would have to be checked again, repeating its side effects. Parse (14–30%) and name lookup (15–17%) cost more.
- **Parse and load**: the sibling parse-ahead work.
- **Unreferenced-declaration checking** after the reachable program: same machinery as above (stage 4), and the least entangled with the metaprogram.

## How to change it

The probe is `crates/jaic/src/sema/body_probe.rs`; hook points: `lower_body_uncached` (begin/end), the two `body_queue.push` sites, `resolve_entity`, `signature`, `layout_struct_with_runtime_dependencies`, and `finish_program` (report). New shared table in `Compiler`? Declare it as `Probed<T, { W::Name as usize }>` and add `Name "owner.field"` to the `writes!` list in the probe, so its in-place writes are counted; a plain field that changes in a body calls `body_probe::write(W::Name)` next to the change. `Probed` has a `DerefMut`, so code that only reads must not take `&mut` (read through `scope(id)`, not `scope_mut(id)`), or the count is inflated; assigning a whole new value needs `.into()`. A table that grows adds a counter to `snapshot` as before. Remove `next_body` and the FIFO switch when the study is done; it exists only to show order sensitivity.

## Configuration

- `body-probe` (cargo feature of `jaic` and `jaic-cli`, off by default): count in-place writes; without it the probe still reports table growth and says the writes were not counted.
- `-dev_body_probe`: print the per-body mutation histogram, the in-place write counts and the queue-tree critical path to stderr at the end of `finish_program`. Programs that run a metaprogram print one report per compiler; the last one holds the totals of the thread.
- `-dev_body_fifo`: lower queued bodies first-in-first-out (study only; output differs from the default).

## Dependencies

`sema` (`procs.rs`, `scope.rs`, `decls.rs`, `structs.rs`), the interner and parse cache from the parallel lex/parse work, `ir::Program`, `interp` (stays serial).
