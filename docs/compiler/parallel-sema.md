# Parallel type checking

A design for checking procedure bodies on several threads. Sema itself is not parallel yet. **The current plan is [Shared tables design](#shared-tables-design)**: bodies run in parallel on shared insert-only tables and output order comes from stable keys. The speculative commit design (Design, Plan, stages 1 and 2) is superseded where the two conflict; its prototype and measurements are kept below. What exists: a measurement probe (`sema/body_probe.rs`, `-dev_body_probe`), the `Send + Sync` conversion of the shared world (`sema/sync_world.rs`), and a one-thread prototype of the batch/abandon/commit protocol (`sema/speculative.rs`, [stage 1](#stage-1-one-thread-prototype)), [stage 2, part 1](#stage-2-part-1-first-uses) (fewer abandoned bodies) and [part 2](#stage-2-part-2-provisional-functions) (provisional function ids).

On large `-check` runs most time is checking bodies (about 60% on a generated 60k-line program), so that is the target.

## What the study found

- About a third to two thirds of bodies grow no shared table, but they hold a small share of the time. The rest add types, procedures, globals or lazily resolve entities, signatures and struct layouts.
- The work is wide, not deep: on the 60k program the critical path of the "body queued another body" tree is about 3% of the total.
- Output depends on lowering order (function numbering and names follow it), so a scheme that numbers in completion order must give every place that depends on numbering a stable key (this is what the [shared tables design](#shared-tables-design) does; the speculative design committed in serial order instead).
- Programs dominated by compile-time execution (a build metaprogram) are serial no matter what.

## Shared tables design

Status: shelved (2026-11-01), nothing implemented. On real programs the estimate is 1.1x to 1.3x on type checking, about 25 ms on game2d (under 10% of a debug build, 3% of a release build), which does not pay for the risk; codegen and link are where build time goes. Revisit if large programs make checking dominate. It supersedes the speculative design below ([Design](#design-speculative-bodies-superseded), stage 2 part 3 onwards) wherever they conflict; the stage 1 and 2 prototype and its measurements stay as the record of what a body writes.

Bodies run truly in parallel against shared, insert-only concurrent tables. Lookups are plain atomic loads. A new type, procedure or constant is inserted with a CAS; a thread that loses a race drops its entry and adopts the winner's. Ids come from `fetch_add` counters, may arrive in any order and may have gaps. Only visible behaviour has to be deterministic, so every place where an id's order reaches the output gets a stable key instead.

Rules the design rests on:

1. **Monotone reads.** A query a body acts on returns its final answer or makes the body wait or abort. It never returns "someone is building this" as if that were an error (today's `EntityState::Resolving` and `LayoutState::InProgress` answers are errors, which is only right for the same thread).
2. **Idempotent bodies.** Everything a body does to shared tables is an insert of something content-addressed or claimed. A body can be abandoned at any point and rerun later with the same result. This is what makes "abort and hand off to serial" cheap, and it replaces the speculative undo journal.
3. **Claim, then build, for anything with identity.** Procedures, nominal types, scopes and descriptor globals are created by the thread that wins a claim (a `Building(owner)` state), so no orphan `ProcInfo` or struct type appears. Build-then-CAS (loser drops) is only for content-addressed values: structural types, string and constant globals.
4. **Output order comes from keys, never ids.** See the tables below. A debug build with `JAIC_ID_SCRAMBLE` extended to sema ids (see the plan) finds the misses.
5. **A serial-only operation aborts the body** (unwinding, as `speculative::abandon` does) and the body, or the step it was in, is redone on the main thread.

### 1. Where id order reaches visible output

Mechanism: **C** is one canonicalization pass over `ir::Program` after the last body is checked and before stack-trace instrumentation and codegen (`stack_trace.rs`, `jaic-llvm`). It sorts functions, globals and foreigns by their key, renumbers them (this also closes gaps) and rewrites every reference (`remap_func_ids` in `sema/speculative/funcs.rs` already does the function half for `Call`, `FuncAddr`, `ForeignAddr`). **S** is a sort at the site. Keys are stored when the object is created: `ProcKey` = (file load order, declaration offset, instance-binding hash, ordinal of an anonymous procedure in its enclosing body); `TypeKey` is structural (nominal types by declaration key plus polymorphic arguments, others by their parts); `FuncKey`/`GlobalKey` are the key of the procedure or type the object stands for, or a content hash for strings and constants.

| Id | Site | Visible as | Stable key | How |
| --- | --- | --- | --- | --- |
| FuncId | `jaic-llvm/src/lower/symbols.rs:71` | Binary symbol `name.<FuncId>`: object symbol tables, native backtraces, debuggers | `FuncKey`; the suffix becomes the rank among functions of the same name | C |
| FuncId | `jaic-llvm/src/lower.rs:100`, `partition.rs:61,117,138,191,218` | Function order in the module, assignment to codegen units (`-codegen_units`), object order and link order | `FuncKey` order | C |
| FuncId | `stack_trace.rs:429,459` | `Stack_Trace_Node.hash` is computed from the FuncId: a runtime value programs can read | Runs after C | C |
| FuncId | `interp/mod.rs:75`, `interp/calls.rs:523` | Compile-time procedure pointer is `FUNC_TAG \| FuncId`: a `#run` that prints or hashes one sees the number. Unspecified in the language; the numbers would vary between runs | Accepted as unspecified, or `FUNC_TAG \| (key hash >> 16)` with a side table (decide in stage 0) | S |
| FuncId | `ir.md` listing, `-emit_ir` | IR function order and names | C | C |
| GlobalId | `symbols.rs:257` | Binary symbol `name.g<GlobalId>` | `GlobalKey` rank among same name | C |
| GlobalId | `sema/lower.rs:832,856`, `expr/unary.rs:154`, `convert/coerce.rs:64`, `typeinfo/views.rs:167` | Names `const.<len>`, `str.<len>`, `array_literal.<len>`, `type_info.<name>.<len>` embed the global count | Content hash for strings and constants, type key for descriptors | C (rename at C) |
| GlobalId | `sema/runtime_info.rs:72-102` | `__runtime_info` data-segment table, in GlobalId order, readable at run time | After C | C |
| GlobalId | `sema/typeinfo.rs:88-120` | Descriptor global is created on first demand (its position is the first asker's) | The type's `TypeKey` | C |
| ForeignId | `jaic-llvm/src/lower/symbols.rs:204` (wasm import order), `lower/plan.rs:172` | Declaration order of foreign symbols, wasm import section order | (library key, symbol) | C |
| ForeignId | `interp/mod.rs:78` | `FOREIGN_TAG \| ForeignId` pointer, same as FuncId | as FuncId | S |
| Library index | `sema/decls.rs:1041-1047`, `ir::Program::libraries` | Link line order (`Foreign.library` is an index) | Library declaration key (file, offset) | C |
| TypeId | `sema/runtime_info.rs:84-89`, `:210-216` | `get_type_table()` at run time and at compile time, `__runtime_info.type_table`: the order programs iterate | `TypeKey` | S |
| TypeId | `sema/runtime_info.rs:198-205` | The snapshot loop walks `TypeId(0..count)` and describes each type (which descriptors exist, and their global order) | Iterate live slots in `TypeKey` order | S |
| TypeId | `sema/debug_info.rs:153-181`, `jaic-llvm/src/debuginfo.rs:537,630` | Debug-info type order; `Program::debug_types` is keyed by the raw TypeId | `TypeKey`, remapped at C | C |
| TypeId | `build/public_api/binding.rs:171` via `scope/model.rs:144` | Workspace record types in the type table (fixed by the recent change) | EntityId of module-level declarations, still serial | none |
| ProcId | `sema/code_export/typechecked.rs:263-278` | Order of `procedure_bodies` in `TYPECHECKED` messages: `pr.procs` is scanned in ProcId order | `ProcKey` | S |
| ProcId | `sema/code_export/typechecked.rs:187`, `sema/perf.rs:53,92,107` | `PERFORMANCE_REPORT`: `poly_order` (first request), `PolyStat.instances` and call-site counts follow request order | `ProcKey` of the source; instances by (span key); first-request span is the minimum over requesters | S |
| ProcId | `sema/perf.rs:123` | `#dump` messages in ProcId order | `ProcKey` | S |
| ProcId | `sema/driver.rs:432-452` (`check_unreferenced`) | Unreferenced procedures and entities checked "in creation order", which fixes the first error | `ProcKey` / entity span key; a worklist per round | S |
| ProcId | `sema/procs/drain.rs:60-66` | The error `drain_bodies` reports is the first failed body in pop (LIFO) order | Smallest `ProcKey` among failed bodies. Changes which error shows when several bodies fail | S |
| EntityId | `scope/model.rs:144`, `calls/lenient.rs:127`, `driver.rs:363`, `typechecked.rs:34` | Declaration order for diagnostics, suggestions, type table, messages | Stays valid for module and file scope declarations: they are created serially. Entities made by bodies are not contiguous: sort by (file order, span) wherever they can mix in (`driver.rs:363` covers body scopes) | S |
| ScopeId | `hidden_imports.rs:23`, `scope/expand.rs`, `ns.unsettled` | Scan order of module and file scopes | Created serially, stays valid | none |
| ModuleId, FileId | `module_rank.rs:85`, `debug_info.rs:141`, `ide/*` sorts on `span.file` | Module rank ties, `file_paths`, diagnostics order | Created only by serial operations, stays valid | none |
| CodeId | `sema/mod.rs:616` | Identity of `Code` values; the interpreter keeps a parallel table (`ct.interp.codes`) | Not ordered in output; creation is serial because it writes the interpreter | serial |
| AstId, Sym | `jaic-syntax/src/ast.rs:38`, `intern.rs` | Nothing may depend on them; already thread-timing numbers, tested by `JAIC_ID_SCRAMBLE` and `JAIC_SYM_SCRAMBLE` | n/a | none |
| first-use span | `procs/lowering.rs:448-534`, `procs/func.rs` (`proc_func(p, span)`), `perf.rs` | Not an id: data that belongs to the winner of a race. An instance's request span, the span in an error from a memoized resolution. Differs by which body asked first | Minimum over all askers by (file order, offset); memoized errors keep the declaration span | S |
| id-keyed hash order | any `for (k, v) in map` over `HashMap<TypeId/ProcId/EntityId/..>` | Fx hashes the id, so iteration order follows the numbering. None found that reaches output unsorted | Sort before use; enforced by the scramble | S |

**Diagnostics.** Errors: the compile reports one error (`drain.rs:60-66`). With threads the failing bodies are collected and the smallest `ProcKey` wins; bodies that fail in the parallel region are discarded and rerun in the serial tail, where `drain_bodies`' retry loop and the lenient parking run unchanged. So no per-body error buffering is needed. Warnings: `warn()` (`mod.rs:679`) deduplicates by (span, message) in push order and `jaic-cli/src/run.rs:476` prints in push order. This needs **per-body buffers merged at the end of the region in `ProcKey` order** (first occurrence wins, as now). Truncating the shared warning list (`stmt/branch.rs:56`) is editor-only and stays serial.

**Compiler messages.** `TYPECHECKED` is not produced as bodies finish: `build/drive.rs:210-245` builds it from the tables at the end of a step (declarations by an entity scan, bodies by `pr.procs`). `FILE`/`IMPORT`/`PHASE` come from serial loads. So **no per-body message buffering is needed**; the two scans get sorted keys (rows above). The set of ready bodies at a step boundary is deterministic because a region drains completely before the step ends.

**Compile-time results that can observe ids:** procedure pointers (above), `get_type_table()` (above), tables keyed by pointers (addresses are heap-allocated in first-access order already, so they were never stable), record numbers handed to metaprograms (numbered in export order, which is sorted now).

### 2. Shared mutable state touched while checking a body

Names are the `W` entries of `sema/body_probe.rs`. "Writes" are mutable borrows counted during a body (an upper bound on inserts); numbers are in [section 4](#4-measurements).

| State | Treatment | Notes |
| --- | --- | --- |
| `ns.scopes`, `ns.entities`, `pr.procs`, `pr.codes`, `ty.types` kinds, `types.structs/enums/distincts`, `ty.poly_structs`, `ns.libraries`, `out.program.{funcs, func_names, reserved_sigs, globals, foreigns}` | **Append-only chunked store**, ids from per-thread blocks of a `fetch_add` counter, gaps allowed | Reads are 50 times (entities) to over 1,000 times (type kinds) the inserts; the store must make `get(id)` two loads. Iteration skips gaps and is only done at serial points |
| `ty.types` intern | **Insert-only map** `TypeKind` to `TypeId`, build-then-CAS | Hit rate 77% to 88% |
| `out.strings`, `out.packs`, `out.const_addr_globals`, `ty.default_globals`, `ty.initializers`, `ty.generic_struct_infos` | Insert-only map, build-then-CAS; the global is created by the winner, the loser's id is a gap | `str.<n>` names become content-derived (rows above) |
| `ty.infos`, `failed_infos`, `building_infos`, `deferred_infos`, `snapshot_infos`, `cases_building`, `variadic_cases` | **Once-cell per type** (`Building(owner)` then `Done`); descriptors register before they are filled because they can refer to themselves, so a thread meeting its own `Building` proceeds, another waits | `defer_infos`/`in_snapshot`/`strict_sizes` flags become per-thread; the snapshot is a serial operation |
| `EntityState` (`ns.entities[i].state`), `ns.default_scopes`, `ns.thunk_scopes`, `ty.anonymous`, `pr.anonymous_procs`, `pr.auto_bake_variants` | **Once-cell** with `Building(owner)`; get-or-create maps claim before they build | `thunk_scope_taken` and `anonymous` keys are `(AstId, ScopeId)`; claim-then-build keeps one scope per key |
| `ProcInfo`: `sig`, `sig_resolving`, `poly_checked`, `is_poly`, `target`, `body_state`, `instances` | `sig`, `target`: once-cell. `body_state`: CAS `NotNeeded -> Queued` by the first reserver, who pushes the body; `Queued -> Lowering` by the worker that pops it. `instances`: per-procedure insert-only map whose value is `Instantiating(owner)` until the instance exists | A body that needs another body *done* (`lower_with_callees`, `code_of`) is a compile-time operation: serial |
| `StructInfo.layout`, `fields`, `size`, `align`; `ty.struct_asts`, `field_types`, `field_declarations`, `field_using_order`, `using_filters`, `member_aliases`, `*_pending` | Layout is a once-cell (`Pending/Building/Done`) with the fields published together; the `*_pending` sets are cycle guards and become the same `Building(owner)` word; the rest are insert-only maps keyed by `StructId`/`ScopeId` written by the layout owner | Layout can run `#run` for an array size: serial hand-off from inside the cell, which resets it to `Pending` |
| `ty.default_images`, `ty.enum_*`, `ty.nominal_origins`, `ty.poly_aliases`, `pr.callable_*`, `pr.proc_decl_notes`, `pr.discard_params`, `pr.const_macro_params`, `pr.local_consts`, `pr.local_decl_types`, `pr.first_arguments`, `ty.info_flags` | Insert-only maps; values are pure functions of the key | `local_consts` etc. are keyed by entities the body made: could be thread-local |
| `pr.body_queue`, `parked_bodies` | Per-thread deques (work stealing through `jaic::pool`/rayon) plus an in-flight counter for termination | Today's LIFO becomes a per-thread LIFO; the order of bodies is not observable after the rules above |
| `caches.module_names`, `module_misses`, `scope_misses`, `using_member_free`, `name_caches.import_plans`, `param_is_code`, `ns.overload_sets_checked` | **Per-thread memo**, dropped when `namespace_generation` moves (they are already stamped with it; `speculative::MEMOS` lists the same four) | 70k writes on game2d, 660k on the 600k program: contention if shared |
| `ns.reexport_visiting`, `pr.last_call_must`, `pr.calls_in_flight`, `pr.asm_pins`, `pr.lowering_depth`, `epochs.resolving_depth`, `mapping_imports`, `retrying_pending`, `lookup_without_expansion`, `out.runtime_storage_dependencies.captures`, `ty.in_element_pattern` | **Per-thread** scratch (stacks and flags) | `scope/expand.rs:90` and `modules.rs:89` test "any body is Lowering" and `expand.rs:120` "any entity is Resolving" across the whole table: they must become "this thread is" |
| `diag.dependencies` (placeholder reads per body), `meta.export.expression_types` | Per-thread; merged only if the body read a placeholder, which is serial anyway | |
| `diag.warnings` | **Per-body buffer**, merged in `ProcKey` order at region end, then deduplicated | See Diagnostics above |
| `diag.deferred_errors`, `pr.lenient_failures`, `pr.provisional_errors`, `epochs.placeholder_misses`, `in_progress_misses` | Errors: discarded with the body and rerun in the serial tail. Counters: per-thread, summed | Only the lenient/serial paths read them |
| `meta.perf` (`poly_request`, `poly_instance`, `lowering`) | Per-thread buffers of (source, span) events, merged sorted by key at region end; the duration is an atomic sum | 80k writes on the 600k program, one per call to a generic |
| `ty.default_images` computing a default aggregate | Pure memo (insert-only), but evaluates `#run` for non-trivial defaults: that path is serial | `structs/defaults.rs:150` |
| `ed` (editor recorders), `ide` facts | Workers get an idle editor handle; a live editor keeps the serial path | 99% of bodies borrow `ed` mutably today, but it is idle in builds |
| `src.sources`, `src.files`, `ns.modules`, `src.insert_*`, `ns.module_cache` | Read-only during a region (written only by serial loads); the reads are 55k on game2d | |
| `src.import_trace`, `src.library_files`, `caches.declaring_imports`, `stdlib_view` | Written on error and report paths, which run serially | `RefCell`/`Rc` stay out of workers |
| `meta.export` (`type_names`, `own_types`, record state) | Only touched when a metaprogram intercepts; step-time, serial | Bodies only call `expression_types.remove` |
| Symbol interner, `AstId::fresh` | Already process-wide and concurrent (sharded interner with per-thread caches; blocks of 256 AstIds) | Reused as the model for the id counters |
| `out.program` bodies (`Func`) | Each body writes only its own `Func`; stored into `funcs[id]` with a Release store | The 6k reads are `funcs.get` of callees' signatures |
| `ct.*` (interpreter, executions, top-level runs, asserts, frozen, twin) | **Must stay serial** | [section 3](#3-what-must-stay-serial) |

**Lazy-resolution cycles.** Every once-cell above has the shape `Empty -> Building(owner) -> Done | Failed`, held in one atomic word with the owner's worker index. The rules:

1. **Same owner** meets `Building(me)`: unchanged. It is a real cycle and takes today's error path (`decls.rs:59`, `structs/layout.rs:90`, `procs/signature.rs:216`: `CircularDependency`, `StructContainsItselfUse`, `ProcSignatureDependsItself`).
2. **Another owner**: the requester publishes `waiting[me] = owner` in a table of one slot per worker, walks `waiting[owner]` and so on (at most `T` steps), and if the chain does not return to it, waits on the cell (spin briefly, then park; the owner's publish wakes it). A wait never changes what the body computes, because it returns the final answer.
3. **A wait chain that returns to the requester is a cross-thread cycle.** Every worker on the cycle is told to give up. Each unwinds with a `Retry` code; the existing error paths already put the cells back (`EntityState::Unresolved`, `LayoutState::Pending`, `sig_resolving = false`), and the worker sets a poison flag. The poisoned bodies go to the serial tail and run in `ProcKey` order with no other worker active, so a real cycle produces exactly the diagnostic the serial path gives, and a false one (two threads each holding what the other wants) just resolves.
4. **Swallowed errors.** About 44 call sites turn a failed resolution into a fallback (`resolve_entity(..).ok()`, `if let Ok(..) = self.layout_struct(..)`, `let _ = self.type_info_global(..)`; found by grep). A swallowed `Retry` would let a body continue on a wrong assumption. So `Retry` is not only an `Err`: the poison flag is checked at the body root (the result is thrown away) and at every publish (a poisoned worker publishes nothing: `Done` cell stores, `funcs[id]` stores, queue pushes). Inserts of content-addressed values stay allowed, since they are true regardless.
5. **Orphans.** Aborted attempts leave only content-addressed inserts, and a rerun makes the same ones, so the final set of types and globals does not depend on aborts. This needs rule 3 of the design (claim before build) wherever a type or procedure has identity: `Types::new_struct` and `new_proc` (8 sites) take a claim keyed by declaration.
6. **No cell is waited on across a serial operation.** A body that must do one (next section) releases its cells by unwinding first.

### 3. What must stay serial

Reuses the classification of the speculative prototype: the `Wrote(table)` reasons of `speculative.rs` `Reason`, `first_use`/`FirstUse::{Entity, Proc, Signature}` (`resolve_ahead`), and the `Abandoned` unwind. In the new design only the rows below abort; everything the speculative code abandoned for `ty.types`, `out.program`, `pr.procs`, `ns.entities`, `meta.perf` and `ns.thunk_scopes` no longer does.

| Operation | Detected by (`W` entry or call) | Hand-off |
| --- | --- | --- |
| `#run`, a constant that is not folded (`eval_const` falls through to `run_thunk`, `decls.rs:935`, `consteval.rs:101`), `#assert`, global initializers that run code, frozen values | `ct.interp`, `ct.executions`, `ct.top_level_runs`, `ct.asserts`, `ct.frozen`, `ct.twin` | Inside a first use (an entity resolution, a signature, a struct layout, a descriptor): **step hand-off**. The body unwinds, the main thread performs that step at the barrier in `ProcKey` order of the requesting bodies (what `resolve_ahead` does), and the body goes back into the pool. Inline in the body (`#run` as an expression): the **whole body** reruns in the serial tail |
| `#insert`, `#if` and imports pending in a scope the lookup falls through, `#placeholder`, `#add_context`, `#poke_name`, module parameters, body-level `#import` (`stmt/block.rs:698`) | `epochs.namespace_generation`, `ns.unsettled`, `ns.deferred_pending`, `ns.pokes`, `ns.add_contexts`, `src.hoisted_imports`, `epochs.placeholder_misses`, `diag.dependencies` | Whole body in the tail (step hand-off when it sits in a first use) |
| Module and file loads | `ns.modules`, `ns.module_cache`, `src.files`, `src.sources`, `src.file_by_path`, `src.provided_texts`, `src.insert_depth`, `src.insert_sites` (0 bodies in all five workloads) | Same |
| `Code` values, `compiler_get_code`, `code_of(proc)` | `pr.codes`, `pr.code_scopes`, `pr.code_procs`, `ct.interp` | Whole body in the tail |
| Metaprogram loops (`compiler_wait_for_message`, `build/drive.rs` steps), `drain_bodies_lenient`, `lower_with_callees`, `export_code_typed` | They run on the main thread from the interpreter, outside a region | A region is one `drain_bodies` call and ends with an empty queue. These never run during a region |
| Errors | Any `Err` out of a body | The failed bodies are discarded and rerun in the tail (retry loop of `drain_bodies` unchanged) |
| Editor live, host twin, `-dev_body_fifo` | `ed.is_live()`, `meta.twin` | Region runs with one worker |

**Loop.** `drain_bodies` becomes: run the pool until every deque is empty and nothing is in flight (a **region**); then run the serial requests (step hand-offs and whole-body reruns) one at a time on the main thread in `ProcKey` order; anything they queue starts another region; stop when both are empty. The number of regions is bounded by the chain length of serial bodies (30 on game2d: the queue tree depth).

### 4. Measurements

Release build (`-j 4`, `body-probe` feature, no LLVM), `-check`, one thread, the probe from `-dev_body_probe` extended in this change with read counts, growth totals and lookup hit rates (`body_probe.rs`). The machine ran another agent's builds: load average 2.1 to 2.7 throughout, so wall times are best of 5 and rough. Reads are `Deref` plus `Index` (include `len`/`iter`), writes are mutable borrows, both only while a body is being lowered, so lookups and inserts here are upper bounds except where a hit rate is given (counted at the keyed lookup itself).

Bodies that need something serial (the probe's set A: `ct_exec`, `namespace_gen`, modules, files):

| Workload | Bodies | Wall | Body time | Serial-set bodies | Their share of body time | Pure bodies | `ct_exec` / `namespace_gen` bodies | Critical path of queue tree |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `examples/game2d/game` | 1,940 | 104 ms | 57 ms (55%) | 10.4% | 42% | 27.4% | 9.2% / 2.9% | 3.4 ms |
| `examples/ui-gallery` | 735 | 52 ms | 24 ms (47%) | 17.0% | 58% | 28.7% | 15.1% / 1.9% | 5.3 ms |
| `benchmarks/compile/stdlib-imports.jai` | 224 | 28 ms | 6.9 ms (25%) | 21.4% | 61% | 30.8% | 18.8% / 9.8% | 1.1 ms |
| same, `-no_dce` | 1,209 | 54 ms | 28 ms (52%) | 12.5% | 50% | 47.6% | 11.8% / 3.6% | 1.3 ms |
| 600k generated lines | 30,562 | 813 ms | 633 ms (78%) | 0.5% | 5% | 70.3% | 0.1% / 0.4% | 24 ms |

Modules, files and warnings were 0 for every workload. Interpreter time itself is small (game2d: 4 ms in 721 interpreter calls), so a serial-set body is mostly ordinary checking that happens to contain one compile-time operation; that is why step hand-off matters more than it looks.

Reads, growth and keyed lookups (game2d, then the 600k program; per body in brackets):

| Table | Reads | Growth by bodies (inserts) | Keyed lookups, hit rate |
| --- | --- | --- | --- |
| `ty.types` kinds by id | 1.13M (583) / 24.5M (802) | 2,754 / 24,404 new types | intern: 13,158 lookups, 82.3% / 80,259, 77.5% |
| `ns.scopes` | 2.44M (1,260) / 40.5M (1,327) | 15,797 / 195,720 (body-local) | n/a |
| `ns.entities` | 663k (342) / 13.0M (426) | 13,203 / 209,958 | n/a |
| `pr.procs` | 298k (154) / 2.67M (87) | 2,655 / 32,408 | `ProcInfo.instances`: 2,322 lookups, 75.3% / 14,628, 73.9% |
| `out.program` | 6k / 37k | 2,562 funcs, 5,838 globals, 291 foreigns / 30,943, 7,940, 7 | n/a |
| `out.strings` | 7.6k / 33k | 3,026 / 453 | 4,545 lookups, 33.4% / 32,534, 98.6% |
| `ty.infos` | 3.8k / 21.7k | 236 / 70 | 3,051, 92.3% / 21,429, 99.7% |
| `out.packs`, `out.const_addr_globals` | 0 | 0 | not reached |
| name caches (`caches.*`) | 222k / 2.56M | 70k / 662k writes | memo, per thread |
| `ns.reexport_visiting`, `diag.dependencies` | 223k, 153k / 2.6M, 2.6M | 51k, 4k writes / 450k, 61k writes | per-thread stacks |
| `meta.perf` | 544 / 366 | 8.2k / 80k writes | per-thread buffer |
| `ed` (idle recorder) | 578k / 11.3M | 336k / 7.3M writes | no-op handle |

Reading it: id-to-data reads outnumber inserts from 50:1 (entities) to 400:1 and more (type kinds, 1,000:1 on the 600k program), which is why the stores are chunked arrays with two-load reads and the maps are consulted far less (13k intern lookups against 1.1M type reads on game2d). Keyed hit rates are high on big programs and low on small ones (strings 33%), so a lost CAS race is rare: 24k new types in 633 ms of body time is one insert per 26 us across all threads. New ids per body (all tables): about 22 on game2d, 16 on the 600k program, so per-thread id blocks are enough. Each worker will touch `ns.scopes` about 1,300 times a body, almost all of it on its own scopes plus the shared module scopes, which are read-only during a region.

Not measured: the single-thread slowdown from atomics (stage 1 gate), contention (needs the implementation), and the share of serial-set bodies whose serial operation sits inside a first use (stage 3 measures it with the `first_use` counters from the speculative prototype).

### 5. Data structures

All candidates were looked up on crates.io on 2026-11-01. `tools/ci/check_dependency_age.py` (14 days, not yanked) must be run on the real `Cargo.lock` when a crate is added; every version below is far enough back.

| Need | Pick | Why | Not picked |
| --- | --- | --- | --- |
| id to data store (append-only, stable addresses, wait-free reads) | `boxcar` 0.2.14 (MIT, 2025-08-13, no dependencies) to start; a hand-rolled `Chunked<T>` after, if the stage 1 gate says the per-slot flag costs more than 2% | Safe API, `push` returns the index, `get` is wait-free, iteration skips unfilled slots (gaps). The hand-rolled version follows `jaic-syntax/src/intern.rs` (fixed array of `AtomicPtr` segments, `AtomicU32` next id) and needs no slot flag because an id only reaches another thread through the map or cell that published it (Release/Acquire) | `Vec` behind a lock (every read locks), `scc::TreeIndex` |
| insert-only content map | `papaya` 0.2.5 (MIT, 2026-08-21, depends on `seize` 0.5.1 and `equivalent`) | Lock-free reads, CAS inserts (`try_insert`, `get_or_insert_with`: the closure can run twice on a race, which is the loser-drops rule), cooperative resize. Inserts never remove, so reclamation only frees old table versions; a worker holds one pinned guard per body | `dashmap` 6.2.1: sharded `RwLock`, a read takes a shared lock. `scc::HashMap` 3.8.8 (Apache-2.0, 2026-09-01, 2 dependencies): lock-free reads but bucket latch on insert and heavier. `std::sync::RwLock<HashMap>`: reads lock, 1.1M of them per game2d compile |
| once-cell with a `Building` state | Hand-rolled `AtomicU64` word (phase, owner) beside `std::sync::OnceLock<T>` for the value | The owner id is what cycle detection needs; `OnceLock::get_or_init` blocks without telling who holds it | `once_cell` (no owner) |
| wait/wake | `std::sync::{Mutex, Condvar}` per worker | At most `T` waiters; no new crate | `parking_lot` |
| per-thread buffers, work stealing | `jaic::pool` (rayon) with `thread_local!` and an ordered merge | Already the one pool | |
| id counters | `AtomicU32` handed out in per-thread blocks of 64 to 256, as `AstId::fresh` does | Keeps the counter line cold | |

`sema/sync_world.rs` already proves the frozen tables are `Send + Sync` as plain data. It keeps its job: `world_is_sync`/`state_is_sync` get one `assert_sync` per new type, so a `Cell` or `Rc` inside a cell fails to compile. The workspace denies `unsafe_code` with scoped allows, as in `intern.rs`; the new types are the only unsafe in sema, behind loom tests and a nightly ThreadSanitizer job.

### 6. Implementation plan

Each stage ships behind a switch that is off until it wins. "Size" is changed lines including test churn, one developer.

| Stage | What | Switch | Verified by | Size |
| --- | --- | --- | --- | --- |
| 0. Order independence on the serial compiler | `ProcKey`/`TypeKey`/`FuncKey`/`GlobalKey`; the `S` sorts in section 1; `canonicalize_program` before stack-trace instrumentation and codegen with content-derived global names; sema id scramble (`JAIC_ID_SCRAMBLE` also gives sema counters random gaps and block order); `-dev_body_order=fifo\|shuffle:<seed>` (grows `-dev_body_fifo`) | `-dev_stable_order`, default on once goldens are updated (it changes which error shows when several bodies fail, and the order of warnings) | `-emit_ir` byte-identical for every body order and id scramble; `determinism.rs` and `symbol_order.rs` run over seeds x orders; `-serial_parse` + `JAIC_SYM_SCRAMBLE` as today | 1.5k lines, golden updates |
| 1. Shared storage, one thread | Replace `Probed<T>` tables by the stores of section 5 with the same call shapes (`push_new`, `insert`, `get`); per-thread scratch gathered into a `Worker` struct; fields of non-table state moved out of the shared groups | cargo feature `shared-tables` (off) | All tests with the feature on; one-thread `-check` within 5% of the baseline on game2d, ui-gallery and the 600k program; output unchanged | 3k lines |
| 2. Threads for pure bodies | Worker handles, once-cells, `waiting[]` cycle detection and the poison flag, deques and region termination, serial tail with whole-body reruns, warning buffers; extend `sync_world.rs` | `-dev_threads <n>` (needs the feature) | `-emit_ir` byte-identical for 1, 2, 4, 8 threads over 20 scheduling seeds (a seeded yield injected at every CAS, wait and queue pop = the thread/scheduling scramble); loom models of the map insert, the chunked store and the cell; TSAN job | 2.5k lines |
| 3. Step hand-off and the rest | Step-level hand-off from first uses (`resolve_ahead` generalised), per-thread `meta.perf` and dependency buffers, instance and anonymous-procedure claims, first-use span minimum, `check_unreferenced` worklists | same | Serial-tail share of bodies at most the set A share plus 3 points; same determinism checks | 1.5k lines |
| 4. On by default | Switch on above a size threshold (about 2,000 queued bodies), CI matrix over thread counts and scramble seeds, docs for `docs/compiler/*`, remove the speculative prototype | `-threads <n>` / automatic | Whole suite at 1 and 4 threads; benchmark gate in CI | 1k lines |
| 5. Optional: smaller serial set | Pure constant thunks on a per-thread interpreter | | Needs an interpreter purity analysis | not estimated |

About 9k lines in all, roughly ten to fourteen weeks. Stage 0 stands alone: it is hardening the project wants anyway, it makes the serial output independent of body order, and it is the only stage that changes visible output.

### 7. Go or no-go

**Conditional go: stages 0 and 1, then a gate.** Stage 0 is cheap and worth having whatever happens. Stage 1 is the measurement of the real risk (single-thread cost of atomics and shared stores). Start stage 2 only if stage 1 meets its gate.

Expected speedup of `-check` from sema threads alone. Formula: wall' = wall - P + 1.25 P / T, with P = body time x (1 - serial-set share) (25% extra CPU for shared tables, serial tail unchanged, parse-ahead already in wall). It gives 2.0x and 2.7x for the 600k program at 4 and 8 threads; the ranges below discount ramp-up (critical path 3% to 22% of body time) and contention:

| Workload | 4 threads | 8 threads |
| --- | --- | --- |
| 600k generated lines (and similar large, plain programs) | 1.7x to 2.0x | 2.2x to 2.6x |
| `examples/game2d`, `-no_dce` stdlib program | 1.2x to 1.3x | 1.25x to 1.4x |
| `examples/ui-gallery` | 1.1x to 1.15x | 1.15x to 1.2x |
| `stdlib-imports.jai` (224 bodies) | under 1.1x: leave serial | under 1.1x |

Real programs gain little because 42% to 61% of their body time is spent in bodies that run compile-time code or change a namespace, and Amdahl does the rest. The levers beyond threads are step hand-off (stage 3) and pure-thunk evaluation (stage 5), neither of which is in these numbers. Whole-compile speedups are smaller by the share of work outside bodies (parse is already parallel; codegen is already in units).

**Stop if:**
- one-thread overhead of the shared stores exceeds 8% on any of the three workloads after tuning, or 5% is not reachable by stage 1's end;
- the stage 2 prototype is below 1.3x at 4 threads on the 600k program, or the serial tail on real programs stays above the set A share by more than 10 points;
- more than ten output-order leaks beyond section 1 turn up under the scramble in stage 2 (the audit is incomplete and the surface is larger than a table);
- a cycle or poison path produces a deadlock or a wrong cell that the loom models and TSAN cannot reproduce deterministically.

**Top risks.** (1) Lazy-resolution cycles and swallowed errors: about 44 call sites turn a failed resolution into a fallback, and monotone reads must hold for all of them. (2) Amdahl on real programs: the 1.1x to 1.3x range is the realistic outcome for typical code, and only stage 3/5 improve it. (3) Determinism leaks through a 65k-line `&mut self` surface (stack-trace hashes from FuncId, pointer tags, winner-dependent spans, hash order over id keys were found by reading; others will be found only by the scramble), plus the first unsafe code in sema.

## Design (speculative bodies, superseded)

> Superseded by the [shared tables design](#shared-tables-design). Reversed there: the "Rejected" and "Rules" paragraphs below (shared concurrent tables are now the design, global ids may come out in any order, no commit in queue order, diagnostics from per-body buffers merged by key). Kept for the prototype and its measurements.

Speculative bodies, deterministic commit.

1. `drain_bodies` takes the next N queued bodies as a batch. During the batch the `Compiler` is immutable and shared.
2. Each worker checks a body through a view holding `&Compiler` plus a delta (new types, procs, strings, diagnostics) with local id ranges.
3. A body that would run compile-time code, change a namespace, load a module, hit an error or warning, touch a `#placeholder` or emit a message is abandoned and re-run serially. Never guess.
4. The driver commits in queue order: remap local ids to global ones, rewrite the finished IR, merge diagnostics.
5. Settle points (`drain_bodies_lenient`, `expand_all`, metaprogram loops, `#run`) stay serial between batches.

Rejected: locks or concurrent arenas (ids in completion order change IR names run to run), pre-resolving declarations (sema is demand-driven), per-thread `Compiler` copies (memory), splitting check from lowering (one pass).

Rules: batch membership and commit order are the serial order; no thread allocates a global id; diagnostics and IDE facts come from committed bodies only; containers iterated at commit are ordered.

## Expected gain (superseded)

See [Go or no-go](#7-go-or-no-go) for the figures from the shared tables design, which are lower on real programs than the target below.

Realistic target: 1.5-1.8x from sema alone and 2-2.4x with parallel parse, at 1.3-1.6x the CPU. The editor keeps the serial path.

## Plan (speculative design; stage 2 part 3 onwards superseded)

The staged plan of the shared tables design is [section 6](#6-implementation-plan). Stages 3 and 4 below are not going to be built.

0. (done) Audit in-place writes during `lower_body`; make the AST, values and signatures `Send + Sync`.
1. (done) One-thread prototype of the view/delta/commit protocol; go or no-go on commit cost and abort rate. Result: go, see [stage 1](#stage-1-one-thread-prototype).
2. Fewer abandoned bodies, then threads for a restricted subset. Part 1 (done): first uses, see [below](#stage-2-part-1-first-uses). Part 2 (functions done, types and procedures not): provisional ids, see [below](#stage-2-part-2-provisional-functions). Part 3: threads.
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

### What stage 2 had to widen first

The stage 1 blockers by share of abandoned bodies: first-use entity resolution (22-40%), resolution bookkeeping (`epochs.resolving_depth`, `ns.thunk_scopes`, 7-17% on small programs), new procedures, signatures and types (up to 14%), and `out.program` / `meta.perf`. Compile-time execution stays serial.

### What is left

Threads; a read-only view type in place of write interception on the whole `Compiler` (the per-thread caches and the non-`Sync` fields listed below go with it); batches sized by the work and not a fixed 64; and the three tables above.

## Stage 2, part 1: first uses

A first use is a demand-driven step that does work the first time something asks for it: resolving a global entity, working out a signature, giving a procedure its function and queueing its body (`proc_func`). A body that was first to ask used to be abandoned. Three changes, all in `sema/speculative.rs` and `sema/speculative/journal.rs`, with output unchanged:

- **Into the delta.** The memo fields a first use fills in are changed in place for the rest of the body, put back when it ends, and replayed at its commit: the entity state (`set_entity_state`), a procedure's `poly_checked` / `is_poly` / `sig` (`proc_memo_mut`), and the thunk scope of a scope (`note_thunk_scope`). An entity resolved this way that asked for a provisional global, or a signature naming a scope the body made, still abandons the body. A thunk scope made by two bodies of a batch invalidates the later one (`thunk_scope_taken`), since the serial path would have made one. `epochs.resolving_depth` is scratch that the body restores.
- **Run ahead at the body's turn.** Steps that write ordered tables cannot be replayed (a function id, a queued body, a new type). When a body is abandoned inside one (`first_use`), its turn runs that step serially, where the serial path would have run it a moment later, and checks the body again (`resolve_ahead`; at most `MAX_AHEAD` = 2 steps per body, and not when the same step comes back). A body that had already asked for a global before it stopped is not resumed: the step would number its globals first. The steps are the first run of `resolve_entity_inner`, `make_proc_target` (plain procedures with a body only) and `make_signature`.
- **Counters.** Bodies abandoned inside a first use are listed as "wrote X while resolving an entity", and the summary says how many bodies committed on the first try and how many only after steps ran ahead.

### Measured

Same workloads and build as stage 1 (release, `-check`, one thread; load average 3-6 from other work, so times are rough). "Committed" is the share of batched bodies committed, with "first try" the part that needed no step run ahead.

| Workload | Committed before | Committed after | First try after | Top remaining blockers (share of batched) |
| --- | --- | --- | --- | --- |
| `examples/game2d/game` | 25.2% | 35.5% | 26.6% | `ty.types` 10%, `out.program` 9% (+9% inside a first use), `meta.perf` 9%, invalidated 7%, `pr.procs` 9% |
| `examples/ui-gallery` | 22.0% | 33.0% | 24.4% | `out.program` 15% (+8%), `ty.types` 15%, `meta.perf` 8%, invalidated 8% |
| ~48 stdlib modules (`-no_dce`) | 46.5% | 59.9% | 52.6% | `out.program` 11%, `ty.types` 9%, `meta.perf` 8%, `ty.default_images` 4% |
| 600k generated lines | 73.3% | 82.2% | 73.3% | `pr.procs` 7%, `meta.perf` 3%, `out.program` 3%, invalidated 2% |

Per cause, as the share of batched bodies abandoned first at that table, before and after (game2d / ui-gallery / stdlib48 / 600k):

| Cause | Before | After |
| --- | --- | --- |
| `ns.entities` (first-use entity) | 40 / 37 / 2 / 22% | 1.4 / 0.6 / 0 / 0% |
| `epochs.resolving_depth` | 3 / 9 / 9 / 0% | 0 |
| `ns.thunk_scopes` | 7 / 4 / 7 / 0.2% | 0 |
| `pr.procs` (callee state, signatures) | 0 / 0 / 14 / 0% | 9 / 5 / 0.4 / 7% |

Most first-use entities are procedures, so clearing `ns.entities` mostly moved the blocker to the next table: the first call of a procedure reserves its function (`out.program`) and queues its body, and its signature interns types (`ty.types`). Those cannot be replayed without provisional ids, so on the first try only 1-6 points more of the bodies commit; the rest are recovered at their turn, which on threads is serial work. `-check` wall time with the flag on is slower than stage 1 (600k lines: about 1.2s then, about 2s now, load-dependent) because a recovered body is checked up to three times; with the flag off it is unchanged (0.70s both, and `-emit_ir` of `game2d` and `ui-gallery` identical to the build before).

The few percent of `ns.default_scopes` / `ns.overload_sets_checked` / `ty.default_images` blockers are untouched.

### Identical output

`-emit_ir` with and without the flag, built (not `-check`, which writes no IR), `@frozen.*` globals excluded: byte-identical on `game2d`, `ui-gallery`, the 600k-line program (350 MB), and the `basics`, `polymorphism`, `kvdb`, `threads` and `using-records` examples. A program importing about 48 stdlib modules and `examples/metaprogramming` differ only in the order of `type_info` globals, which also differs between two runs with the flag off; the sorted lines match for the stdlib program. `cargo test -p jaic` passes by default, with the feature, and with `JAIC_SPECULATIVE_BODIES=1`; the full `stdlib` suite has the same 22 local failures (missing probe libraries) with and without the flag.

### What is next

Part 2 made function ids replayable, see [below](#stage-2-part-2-provisional-functions); types and procedures are left. Until then threads would speed up only the first-try share. Part 3 adds the threads.

## Stage 2, part 2: provisional functions

The first call of a procedure gives it an IR function and queues its body (`make_proc_target`). A speculative body now does that with a provisional id and leaves the real work for its commit. Same output, `sema/speculative/funcs.rs` plus a few lines in `procs/func.rs`.

### How it works

- **Provisional ids.** `proc_func_in_code` (used by `emit_call` for a call and `lower.rs` for a procedure's address, the two places that only put the id in IR) lets a speculative body reserve a function: the id is the program's function count when the body began plus the body's own count, the target and `Queued` state are set in place and undone when the body ends. The name and signature wait in a list. Foreign symbols (`#foreign`, `#elsewhere`, llvm intrinsics) get the same treatment (`ForeignId`).
- **Anywhere else a provisional id would leak.** `proc_func` (constants and relocations, `check_failed`, imports) still has to be the first to reserve: a body that finds a provisional target through it is abandoned (`refuse_provisional`), and one that is first to ask stops and is resumed at its turn as in part 1.
- **Commit.** Before the IR is rewritten, the commit walks the body's list in order. A procedure that has no target gets a real function (`reserve_func_with_sig`, so ids come out in serial order); one that has a target by now (an earlier body of the batch, or a serial body, called it first) keeps it and the body's call is pointed there. Then `remap_func_ids` rewrites `Call`, `FuncAddr` and `ForeignAddr`, and the deferred op for each fresh function queues the body, sets the target and fills `func_procs`/`func_targets`, in the order the serial path would have.
- **Run ahead.** A body that already reserved a function is not resumed at a first use: the step would number its function before it.

### Measured

Same workloads and build as part 1 (release, `-check`, one thread, load average 3-5 from other work). Committed / first try, before and after:

| Workload | Committed | First try |
| --- | --- | --- |
| `examples/game2d/game` | 35.5% -> 40.9% | 26.6% -> 27.3% |
| `examples/ui-gallery` | 33.0% -> 38.5% | 24.4% -> 24.8% |
| ~48 stdlib modules (`-no_dce`) | 59.9% -> 63.3% | 52.6% -> 58.6% |
| 600k generated lines | 82.2% -> 83.8% | 73.3% -> 73.3% |

Share of batched bodies abandoned first at each table (game2d / ui-gallery / stdlib48 / 600k; "in a first use" included):

| Table | Before | After |
| --- | --- | --- |
| `out.program` | 18.0 / 23.1 / 11.9 / 3.2% | 6.1 / 12.7 / 6.1 / 0.1% |
| `pr.procs` | 9.4 / 4.6 / 0.4 / 6.8% | 14.3 / 7.4 / 0.7 / 8.1% |
| `ty.types` | 12.5 / 16.8 / 9.9 / 1.9% | 13.0 / 18.1 / 11.3 / 2.1% |
| `meta.perf` | 8.5 / 8.4 / 8.0 / 3.1% | 8.6 / 8.8 / 8.3 / 3.2% |
| `ty.default_images` | 4.0 / 3.3 / 4.3 / 0% | 4.0 / 3.5 / 4.4 / 0% |
| invalidated by a serial body | 7.0 / 7.9 / 2.1 / 2.3% | 7.2 / 7.9 / 2.4 / 2.3% |

First-try commits barely moved because the bodies that cleared `out.program` stop at the next table: a callee's first call also needs its signature (`ty.types`, the procedure type) and often its `ProcInfo` (`pr.procs`, first-use declarations and `meta.perf` instances). What is left under `out.program` is `#modify` blocks, `#library` declarations and a few address-of constants.

`-check` wall time, best of 5, load average about 3 (ms; flag off / flag on, part 1 then now):

| Workload | Flag off | Flag on |
| --- | --- | --- |
| game2d | 111 / 111 | 246 -> 232 |
| ui-gallery | 66 / 67 | 117 -> 115 |
| stdlib48 | 108 / 109 | 194 -> 173 |
| 600k | 709 / 718 | 1873 -> 1570 |

Flag off is unchanged. Flag on is still about twice as slow on one thread, for the reason in part 1: a recovered body is checked up to three times.

### Run ahead

Dropping it (`MAX_AHEAD` 0) gives back the committed share but not the first try (game2d 41% -> 27%, 600k 84% -> 73%) and saves a fifth of the flag-on wall time (game2d 232 -> 187 ms, 600k 1570 -> 1250 ms). Limiting it to one step per body saves 3-6%. It stays at 2 for now, because the point of the prototype is to count, but note what it is worth with threads: a recovered body is checked again at its own turn, which is serial work, so it saves nothing over the serial path. Only the first-try share can speed up. Before threads, either drop run-ahead or put a recovered body back into a later batch instead of checking it at its turn.

### Identical output

`-emit_ir` with and without the flag, `@frozen.*` globals excluded: byte-identical on `game2d`, `ui-gallery`, the 600k-line program (350 MB) and the `basics`, `polymorphism`, `kvdb`, `threads`, `using-records`, `compile-time-record`, `type-variants` and `memory` examples. The ~48 stdlib module program has identical function IR in the same order; `type_info` globals and the strings and debug ids numbered after them differ, as they do between two runs with the flag off. `examples/metaprogramming` differs in the order of some `__init_*` functions and of `__command_line_arguments` against `__runtime_support_disable_stack_trace`, and that also varies between runs with the flag off. `cargo test -p jaic` passes by default, with the feature and with `JAIC_SPECULATIVE_BODIES=1`; the full `stdlib` suite has the same 22 local failures with and without the flag.

### What is next: types and procedures (superseded)

The shared tables design removes this work: types and procedures are inserted into shared tables instead of being remapped at commit. The analysis below of why they were hard is the reason for that change.

The remaining top blockers need ids in tables other than the program's, and that is wider than the speculative module:

- **Types** (`ty.types`, `ty.default_images`, ~13-18%): about half the writes are `finish_signature` interning a callee's procedure type, the rest pointer, array and slice types from type expressions and `&`. A body would append to `Types` in place and cut it back as it does the scope arena, and the commit would intern the same kinds in order. The cost is the remap: a `TypeId` can sit in anything the body leaves (the expression log, entity kinds and states, `Signature` memos, `ProcTypeParams`, deferred ops that capture a type, `Value`s) and, worse, in the bytes of a constant (a `Type` field). That needs a `TypeKind`/`Value` visitor in `types.rs` and `value.rs`, and a rule that a provisional type never reaches constant bytes or IR immediates (it would abandon at `type_info_global`, the one place a type becomes data).
- **Procedures** (`pr.procs`, `meta.perf`, ~15-23%): a first use of a procedure declaration (`new_proc` in `resolve_entity_inner`) and each polymorphic instantiation push a `ProcInfo`. The same arena approach works, but a `ProcId` is stored in entity states (`Value::Proc`), `instances` maps of older procedures and the perf lists, and two bodies that resolve the same entity or ask for the same instance both create a `ProcInfo`: the commit has to keep the first and map the other's id to it, so ids no longer shift by one constant. The function ids above already do that mapping; `Shift` would carry it for procedures and types too.
- **Invalidated** (7%): a serial body between check and commit changed the stamp (namespace generation, files, compile-time runs). A batch that stops at such a body would not waste the bodies behind it.

Part 3 (threads) as described here is superseded by the shared tables design.

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

For the shared tables design, new shared state needs an entry in [section 2](#2-shared-mutable-state-touched-while-checking-a-body) (its treatment) and, if its order can reach output, a row in [section 1](#1-where-id-order-reaches-visible-output). The paragraph below describes the speculative prototype.

The probe hooks are `lower_body_uncached`, the `body_queue.push` sites, `resolve_entity`, `signature`, `layout_struct_with_runtime_dependencies` and `finish_program`. New shared tables need a delta/commit story before workers can touch them. A memo that a first use fills in goes through `journal` (undo at the end of the body, replay at commit) in `speculative/journal.rs`; a new step that does ordered work gets wrapped in `first_use` so a body stopped inside it can be resumed. An id the body hands out (function, foreign symbol, global) is provisional, starts after the table's end, and is remapped by `Shift` at commit (`speculative/funcs.rs`); anything that would keep such an id past the body must go through `refuse_provisional` or be remapped there.

## Configuration

- `speculative-bodies` cargo feature (off): build the stage 1 prototype.
- `-dev_speculative_bodies` / `JAIC_SPECULATIVE_BODIES=1`: check bodies speculatively (needs the feature; off by default; the environment variable is also how the test suites force it on).
- `body-probe` cargo feature (off): also count in-place writes.
- `-dev_body_probe`: print the mutation histogram and queue-tree critical path at the end of `finish_program`. With the `body-probe` feature it also prints, per shared table, the reads and mutable borrows made while bodies are lowered, how much each id table grew, and the hit rate of the keyed lookups (types, strings, packs, constants, descriptors, polymorphic instances).
- `-dev_body_fifo`: lower queued bodies first-in-first-out (study only; output differs).
