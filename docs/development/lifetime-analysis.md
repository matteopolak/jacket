# Lifetime analysis

Jailint derives allocation origins, reads, writes and invalidation from checked lifetime events. Explicit contracts describe API intent where checked bodies do not establish the required semantics, including native boundaries and logical container identity. A contract is not itself proof that a checked body satisfies it.

## Ownership and flow

`rules/run_state.rs` coordinates state shared by the files and rules of one lint invocation. It holds lazy whole-program summaries and per-file resource reports, plus token and filesystem caches. Sharing orchestration does not make the identities in those analyses interchangeable. A new invocation creates new state; procedure IDs, source spans and type IDs are meaningful only inside that invocation's compiler.

`lifetime_summary::Summaries::build_for` rejects incomplete or obsolete checked bodies, selects the required graph, and builds allocator facts and the indirect-call target universe. It solves strongly connected components from callees upward. Normal-return reachability settles before body preparation; recursive guarantees start unproven. Write and read passes follow the effect solve.

`lifetime_summary/prepared.rs` shares borrowed body definitions inside the summary solve. Preparation depends on the target universe and normal-return blocks. Each resolver keeps its own places, allocator context and summary-dependent queries. CFG consumers build separate preparations after summaries settle. Do not retain a prepared entry across compiler generations or treat source text alone as its validity key.

## Checked body and query ownership

`CheckedSession` selects bodies from its compiler's keyed retained registry. `Prepared` owns their immutable definitions; `Resolver` borrows those definitions without exposing a mutable ownership handle. Captured load maps retain that same borrowed body. A graph reader rejects a load map or definition index from another preparation, even if the numeric SSA identifiers match. This binding does not prove branch applicability: each guarded query must still use captures produced under its own exact edge cuts.

The lazy captured-value index replaces repeated definition scans in frame, bit-return, native-backing and exact-store queries. It records either a complete index or unavailability, including a failed construction attempt. Optional index failure leaves ordinary diagnostics available. Legacy uncounted address queries remain separate and are not a fallback for these migrated readers.

Index construction runs once per preparation. Every top-level proof reserves that actual construction receipt within its existing 8192-work limit; recursive queries charge their visited definitions and sources. Selector/polarity continuations carry the charged view together with cumulative work, while an independent proof reserves its own receipt. Construction statistics distinguish physical visits from proof reservations; shared ownership alone is not a runtime or memory improvement claim.

To extend this seam, update `captured_values.rs` and the existing storage transfer together. Unsupported or conflicting producers must remain unknown in either source order. Add cross-body ownership, exact-load epoch and budget-fallback regressions. The index depends on retained checked events and the selected compiler/type registry, has no persistent cache or environment configuration, and does not replace detached summary identity or trusted-contract validation.

## Storage versions

Formal pointer-slot prestate queries now use the same ordered storage transfer as captured loads. Their slot inventory, current normal-return environment and outputs belong to the query, so evolving call summaries never enter immutable preparation. Physical same-byte pointer stores replace a requested source; numeric overwrites and partial overlaps remain unknown. The legacy MAY projection retains entry/call alternatives and is not a complete or guaranteed poststate. Query work includes nested place cloning, typed layout searches, revisited transfer helpers and joins; failure retains conservative sources rather than discarding ordinary body analysis.


A query can collect private scalar loads through that same transfer and combine them with the body's ordinary pointer captures. This composition uses all paths on both sides, checks actual load types and ownership, and charges cloned sources to the same query budget. It does not establish a correlated return poststate. Writes may be omitted from this scalar projection only when retained original stack allocations prove that their entire selected byte range is separate from every requested slot; calls and barriers retain their ordinary behavior. Extend the existing request selector and transfer when adding demanded bytes, rather than replaying stores in another domain.

A receiver field names a slot, not the allocation stored in it. `storage_versions.rs` tracks reaching definitions for pointer fields that a body replaces. For deallocation and resize queries, each load selects either the field's initial allocation, a checked stored value, or an unknown origin. Capturing a load preserves that selection when the field changes later. Private pointer variables use their captured SSA definitions too. Only positively owned, unexposed local storage is protected from unrelated receiver writes or calls; globals and exposed locals may alias an incoming receiver. Local overlays, partial writes and whole-object initialization use their checked byte extents. Ordinary descriptor and read identities remain separate. Caller releases are evaluated before overwritten fields are forgotten.

```jai
old := holder.storage;
holder.storage = replacement;
free(old); // releases the initial allocation
```

Reversing the store and load releases the replacement instead. Branch joins retain all possible sources; a guaranteed origin requires agreement. Calls and potentially aliasing writes retain uncertainty. `value_writes.rs` adds checked may relations for pointer values installed in receiver slots. Sources distinguish an incoming pointer value, an entry-state slot read and unknown. A helper's slot read is evaluated in the caller's state immediately before the call, so a later overwrite cannot change the captured input. Structural block/event indices identify calls; spans can repeat in expansions. Whole-aggregate stores capture their pointer fields at the store site too; later source replacement cannot change the copied image. Exact cleanup requires an agreed pre-copy source. Allocator cleanup keeps supported may origins when that exact proof is unavailable. Missing write relations never prove purity.

Retention destinations are captured at the store, not reread from an owner variable at cleanup. Replacing `ctx.owner` does not overwrite the old object's `borrowed` field. An exact later store may end the earlier slot retention only when its captured pointer and full byte range agree; unknown captures retain uncertainty. The bounded capture query reuses SSA pointer equality and selected inline layout, with no allocation or disjointness claim. This first query rejects merged or repeated invocation sites and shares one body-local work budget.

An ordinary store is a memory write, not an opaque resource callback. The resource solver invalidates potentially aliased mutable contents while preserving already captured SSA values. A pointer load retains its original slot relationship and can also use one complete stored SSA capture from the shared ordered transfer. Equality is applied only at that load epoch; conflicting known states stay uncertain, and an unknown peer cannot revive a dead pointer. Opaque calls, assembly and runtime report paths retain their separate barrier behavior.

A direct authenticated C `free` has no deallocation effect when its actual pointer argument is proven null. Exact typed scalar defaults and full-width pointer stores can establish this constant; partial writes, replacement and exposed aliases remove the proof. An adjacent selected zero write can also establish the numeric zero bits of a fully covered requested eight-byte pointer slot in original private storage. This uses the target null encoding and supplies no pointer provenance or valid pointee; pointer-origin readers reject the numeric source. This specialization omits only the covered deallocation, preserving independent declared effects such as temporary-storage reset. Extend the existing typed constant transfer and native ABI authentication when adding cases; do not infer a no-op from a wrapper's name or from missing may effects.

An owner release alone does not prove that its former contents stopped escaping. For example, `KEEP = owner.borrowed; free(owner);` leaves the copied pointer alive elsewhere. Helper cleanup therefore still requires a complete publication/effect proof; may-write summaries and empty escape sets cannot supply one. Extend the existing frame observer and its copy-out/retarget controls when admitting cleanup helpers, rather than treating a release annotation as purity.

`physical_slots.rs` certifies inline byte ranges using each checked projection's actual layout. This preserves identity through pointer reinterpretations whose field names have different offsets. Certificates start at immutable incoming pointer values, follow inline fields and bounded fixed-array indexes, and stop at pointer or descriptor hops. By-value aggregate arguments are snapshots and cannot become writable caller slots. Unknown roots, partial writes and overlapping ranges retain uncertainty.

`pointer_returns.rs` uses the same finite source relation for checked scalar pointer results. A return distinguishes an immutable input value, the address of certified inline bytes and a pointer read from certified entry bytes. A structural call-result query instantiates a callee's entry read immediately before that call. A helper that returns a captured pointer cannot silently reread a field after its own replacement, and pointer reinterpretation retains byte identity through getters. Unsupported global slot reads remain unknown. Recursive forwarded write targets and their physical read/address sources widen to unknown, while direct writes and supported may values remain. Recursive physical slot returns also widen to unknown to bound shifted slot growth; layout-independent value origins still converge in their finite domain. Aggregate descriptor/read structure continues to use its separate relation. Logical borrow metadata can retain an explicit `CapturedRead` root: the checked read site, its typed input slot and a bounded chain of caller sites. It says which read template supplied a derived address, not that two dynamic reads are the same runtime pointer. Cleanup and ordinary entry-slot substitution cannot consume it; a consumer needs its own epoch and separation proof. Pointer loads through further indirection remain unknown.

A `ReadFootprint` records the actual selected load's width and inline offset from an immutable input pointer, including a checked view through `*void`. It carries no allocation extent or readable-storage guarantee. Physical cleanup and write certificates retain their stricter extent bounds.

The borrow CFG consumer can use a direct captured read when a separate bounded frame query proves its selector unchanged between the read and return. Calls and uncertain writes reject that proof. The caller resolves its evaluated actual address and maps the captured byte range through the caller's actual layout, requiring one pointer leaf within the original object extent. This preserves a pure getter's borrow across pointer reinterpretation without confusing a returned old pointer with a replacement field. Forwarded read templates remain conditional until their outer frame is proved; cleanup substitution still cannot use them.

The source-query worklist expands each SSA value, call result, call write and pre-call slot read once. Ranges must fit the compiler-known formal pointee extent, and each query graph is limited to 4096 nodes. Exceeding either bound, cancellation or an unsupported projection yields explicit unknown, never absence-as-purity. Extent rejection does not prove an address lies outside its actual allocation. No query result cache crosses an SCC iteration or compiler generation.

This finite domain tracks body-local SSA values and converges across loops without a path-count cutoff. It does not prove that a traversal releases every owned node, that a container remains reusable, or that replacement succeeded.

Extend the transfer rules and matching safe/unknown regressions together. A may effect must never become a guaranteed effect merely because the exact origin is unavailable. Preserve captured aliases, conditional stores, recursion and unknown-call behavior when changing joins or substitution.

## Checked boolean facts

The compiler records semantic `Not`, `And` and `Or` relations as `LifetimeEvent::Boolean`, alongside the actual short-circuit branch. These facts survive executable-body release and let protocol domains relate copied or inverted success results to the same acquisition. Integer XOR is not a boolean proof. For `And` and `Or`, the right SSA value exists only on its evaluation edge; a consumer must not demand it when the left operand already decides the result.

Extend semantic lowering in `sema/expr/unary.rs` and `sema/binary/logical.rs` together with the retained fact and release regression in `jaic/tests/ide_lifetimes.rs`. The facts are part of the optional lint sidecar and do not change executable lowering or require a new flag.

## Checked address and copy facts

`stack_roots` records actual stack allocations and their original byte extents, canonicalizing repeated addresses of the same physical slot. It excludes derived addresses; `stack_places` remains an ancestry list for attribution and cannot prove ownership. A consumer must check each access against the original extent, propagate exposure through aliases, and reject dynamic or out-of-bounds derivations. Unknown allocation sizes permit no positive-width private access.

`allocator_protocol` optionally records the checked canonical allocator invocation roles (operation, requested size, previous size, previous memory, provider data and result), nominal mode type and selected operation tag bits. The producer validates both the canonical declaration and the actual callable's lexical roles; equal pointer parameter types cannot identify swapped old-memory and data arguments. Changed representable enum tags are retained, while ambiguous or unsupported ABIs stay unknown. This records the callback invocation, not allocation, freshness or provider behavior. Sparse boxed certificates avoid adding the full payload to every event.

`FieldProjection` records a selected inline member path using the captured base address, result address, checked types and byte offset. Accesses and runtime literals emit it while the checker still has the selected path. Pointer-following `using` paths cannot establish an inline projection. A nonzero `#as` pointer adjustment emits a projection instead of an equality `Copy`.

`DefaultImage` retains bounded typed leaf values from the exact default image selected at an `Initialize` event, including unambiguous procedure relocations with their original function identity. Consumers must verify that a retained procedure still selects that function before using its behavior. Values describe the immutable global actually selected for the emitted copy, or the selected zero operation; recomputing a default image does not prove that a cached global contains it. The event freezes its emitted byte width independently of later type-layout changes. It covers semantic leaves rather than padding; omitted, unsupported or ambiguous values stay unknown. The producer checks original initialized-type ranges and bounds traversal to 128 leaves, 512 visits and depth 16. Consumers still enforce actual destination extents, overlapping writes and later clobbers. A default image for a freshly initialized context does not establish the provider in an incoming context or prove allocation behavior.

`overlay_accesses` ties a selection barrier to its exact selected inline projection. The original barrier remains for existing consumers. A domain can refine this barrier only after checking that projection and its physical byte overlaps; assembly and unsupported pointer-following selections remain opaque. Surrounding calls and writes retain their own effects.

Runtime `ifx` expressions retain the actual selected branch and the scalar read of their shared result slot. Both arm writes and that final load refer to the same original physical slot even when lowering uses distinct address SSAs. Constant-folded and type-only expressions add no executed branch or load. Extend the existing load recorder when adding another lowering path; reconstructing a missing branch from block shape or guessing a later store loses the selected epoch.

`Offset.subscript` retains a dynamic logical element delta before stride scaling. A pointer subtraction keeps this unknown until its signed delta has a checked relation. `IntegerBinary.ty` preserves operation width and signedness; reasoning with unbounded integers must not prune a branch after machine overflow.

`LifetimeMemoryEffects.copy()` comes from the selected intrinsic and records actual runtime formal positions. Read/write masks alone do not prove byte correspondence. The current copy intrinsic has overlap-safe snapshot semantics in both native lowering and the interpreter. It does not establish allocation freshness or ownership.

The mutually exclusive operation contract also exposes `bounded_write()` for the actual selected `memset` intrinsic. It identifies destination and byte-count formals and promises no accesses outside that write range or hidden callbacks on a valid invocation. It supplies no fill-byte, content, ownership or extent proof. Consumers must validate a nonnegative count, original extent and no-wrap, then invalidate overlapping stored facts. A zero count is vacuous. Copy, comparison, byte lookup, atomics, fiber release and bodyless lookalikes receive no write-only certificate; their masks alone cannot establish it. The enum representation avoids storing two independent optional operation records. Its measured layout is not a stable ABI or a consumer encoding.

Extend these producers in semantic member access, literal construction, pointer indexing, coercion and intrinsic selection together with the body-release regressions. Consumers must preserve unknown for missing facts; source text and similar signatures cannot replace the checked relation.

## Contracts and extension points

The selected runtime-configuration intrinsic records its actual readonly scalar address and load. Its global extent stays fixed while final build settings may replace the initial bytes, so these facts certify the read footprint rather than a numeric value. Only successful fully recorded lowering without declaration notes marks the call's runtime effects expanded. Annotated calls retain their ordinary contract boundary; proof readers also reject independent trusted constraints. A zero-argument, context-free integer-return helper can retain its effect frame for an actual constant or exact typed readonly load, while all preceding events remain checked.


Keep contracts on foreign or intrinsic operations. A checked wrapper may inherit their effects, but a procedure's display name or a field spelling is not evidence of ownership. Allocator provenance, pointer storage, scalar handles, lock protocols, content correspondence and logical element identity are different domains.

Storage and lock work does not complete the seven semantic contract families. Remaining contracts require independent proofs: ordered growth and quantified cleanup; guarded and recursive lock ownership; successful scalar handle acquisition; exact content copy/append; allocator-relative marks and rewind; element permutations; and allocator-bearing receiver fields. An ordinary boolean store, integer equality, byte copy or counter reset cannot establish those protocols. Compiler-read `PrintLike`, `Interpolates` and `Cold` notes remain API intent.

The bounded content, element, allocator-provider and temporary-epoch modules currently join the crate's test build only. Their checked-carrier and adversarial regressions run in CI, but production consumers must still discharge their range, separation, epoch and liveness obligations before those modules can supply effects. Test registration adds no production analysis pass.

Relevant consumers live in `lifetime_cfg`, `unpaired_lock`, `unclosed_resource`, `format_push_string`, `useless_format` and `lifetime_movement`. Extend a domain before removing a carrier note, compare inferred facts with the contract, and test diagnostics with the source note actually absent. Post-check note stripping alone does not validate compiler behavior.

## Measurement and configuration

`JAILINT_LIFETIME_STATS=1` prints `lifetime-phase` records for selection, allocator construction, contracts and targets, graph construction, inference, write/read passes, body preparation and resolver context. `items` counts bodies, events, blocks or iterations as appropriate to the phase; `proc=all` denotes an aggregate phase. Records overlap: preparation and context are included in the surrounding solve's time. Group per-body records to count repeated preparations. `scalar_openings` includes the nested `scalar_relations` and `scalar_callers` records; do not add them together. Their item counts respectively describe indexed source calls, ordinary relation-cache entries (including unknown results), and caller bodies with nonempty probe plans. A missing phase means it was not invoked, not that the work is free. Tracing reads clocks and writes stderr, so disable it for latency comparisons.

```sh
JAILINT_LIFETIME_STATS=1 jacket lint path/to/root.jai 2>lifetime-stats.txt
```

For retained-allocation profiling, use `JAILINT_LIFETIME_MEMORY=1 cargo run --release -p jailint --example lifetime_memory -- --manifest=coverage.json path/to/root.jai`. The profiling example uses one worker and a counting allocator; run it separately from normal latency measurements. `--strict`, `--os=windows` and `--cpu=x64` select stricter pointer analysis and target bodies. Check the coverage manifest before comparing measurements.

Use fixed repository-owned roots and compare the same inputs, target, selected files, entry root and enabled rules. Record revisions, wall/CPU time, RSS, completeness, diagnostics and secondary evidence. Build before timing; alternate paired runs on a quiet machine. A cross-request cache additionally needs identities and invalidation for bindings, contracts, target choices, indirect targets, global escape, allocator context, root selection, SCCs and return reachability. No persistent or cross-thread summary cache is provided.

## Dependencies

The analysis consumes `jaic::ide::lifetimes` checked events and syntax identities, compiler types and symbols, `lifetime_targets` call targets and `lifetime_places` projections. `jacket-config` controls tracing and pointer lifetime mode. No external service is required. Cancellation regressions run in isolated child processes using the internal registered `JAILINT_LIFETIME_CANCELLATION_CHILD` marker.

## Lock protocols

`lifetime_summary/locks.rs` represents recursive lock effects with both a required entry depth and a net depth change. Acquiring then releasing has no entry requirement; releasing then acquiring requires an already held level even though both have net change zero. Native `EnterCriticalSection` and `LeaveCriticalSection` facts require a Windows target, a function symbol from a checked system `kernel32` library and a matching C ABI signature with a void return, one fixed pointer parameter and no variadic tail. See Microsoft's [acquisition](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-entercriticalsection) and [release](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-leavecriticalsection) contracts.

Checked wrappers compose these relations through formal-relative receiver paths. Syntax consumers use `call_bindings.rs` to resolve checked runtime formals to written actuals, including reordered named arguments. Defaults, spreads and disagreeing checked instances remain unknown. Plain `@Locks`/`@Unlocks` select formal zero; indexed forms select the validated runtime formal. Different paths may alias, unknown calls remain unknown, and differing control-flow or recursive depth cannot establish an exact exit relation. The initial domain models one receiver and bounds exact depth at 32; exceeding it yields uncertainty. Status-returning acquisitions are not unconditional acquisition primitives.

`native_locks.rs` consumes checked calls and CFG edges. Each acquisition captures an immutable native object: an incoming pointer, a positively owned inline stack address, or an incoming pointer-slot read, with checked inline byte offsets. By-value formal snapshots do not identify caller pointer addresses. `address_objects.rs` instantiates wrapper receiver certificates from evaluated actual SSA values. Local inline addresses must remain inside the original certified stack allocation, including after a cast or a callee-relative projection; a larger view cannot enlarge that allocation. A later assignment cannot change the captured object; a later load must resolve again at its own epoch. Deferred calls run on their checked exit edges. Unknown calls and unresolved pointer aliases keep the path uncertain. Different incoming formals do not prove physical disjointness; a warning says a path can retain an obligation, not that all actual inputs differ.

Native receiver tracking bounds exact depth at 32, path states at 256, candidate sites at 256 and total event work per body at 8192. Receiver queries have a 256-node budget. Temporary call transfers are materialized once at checked event epochs after summaries settle. Exhaustion discards the body report. Cyclic CFGs remain unknown until repeated acquisitions have a recency domain. Missing or obsolete bodies, disagreement across checked instances and cancellation publish no native finding. The syntax scan remains for explicit contracts on unmodeled protocols; it does not pair native effects by written expressions.

`unpaired_lock` uses proven call effects separately from wrapper intent. `@Locks` declares that exporting a held obligation is intentional. An inferred positive delta describes an effect, but cannot grant that exemption: it would also exempt ordinary functions that forgot to unlock. `@Unlocks` remains a boundary for unmodeled protocols. Inferred release effects do not authorize guessing an unlock name for an editor suggestion.

## Checked carrier inventory

`lifetime_carriers` inventories notes by procedure instance, with full contract text,
source spans and checked/bodyless/template status. Run each relevant target separately:

```sh
cargo run -p jailint --example lifetime_carriers -- examples/editor/main.jai > carriers-host.jsonl
cargo run -p jailint --example lifetime_carriers -- examples/editor/main.jai --os=windows --cpu=x64 > carriers-windows.jsonl
```

The command fails on incomplete checking or empty carrier coverage. Its checked rows
prove coverage, not inference equivalence; unchecked generic templates and bodyless
boundaries are explicitly different records. A carrier-removal test must still identify
nonempty checked instances and compare the inferred relation and actual diagnostics.
The selected source roots determine which templates are instantiated.

### Generated entry calls

A generated `#entry_point` wrapper has no ordinary source CFG. Its sparse retained edge records the actual selected target and whether the generated call forwards the hidden context, passes null, or has no context operand. Null is a null pointer, and forwarding preserves the pointer value only; neither establishes a default provider or unchanged context memory.

The producer records an edge only after successful generated-body installation in lint mode. Regeneration clears the old edge before fallible resolution. Consumers use `Compiler::lifetime_generated_entry`, which checks current procedure/function/syntax identities, context flags and the target's completed retained body. Executable IR release preserves these facts; `clear_lifetimes` releases them. The record is not a source span, dynamic acquisition identity, return-origin fact or purity certificate. Extend `ide/lifetimes/generated.rs` and `sema/procs/func.rs` together when changing this generated call. `lifetime_generated_entry_bytes` reports reserved record payload without counting shared syntax twice.


### Diagnostic derivation labels

Resource warnings keep their CFG proof unchanged. A separate bounded metadata query can replace a later pointer-load label with its proven captured origin. It follows only actual equality copies and complete reaching SSA values captured at that load event, and requires every alternative to reach the same actual resource origin. It never rereads a later variable value or guesses an allocation from a source name. Replacements keep separate SSA origins; mixed branches, exposure, unsupported producers, cycles and exhausted work retain the original load label.

`lifetime_cfg/resources/origins.rs` owns this query; `lifetime_summary/storage_versions.rs` captures the alternatives and `Prepared` exposes them without rebuilding a summary-dependent resolver. Construction is capped at 8192 block/event/source/edge work units, including canonical edge insertion, each query at 256 steps, and total query work at 65536 steps per body. Incomplete construction publishes no index. The graph is built only after a new finding needs an origin label; quiet bodies do not construct it, and a failed construction is not retried. The temporary graph and memo belong to that checked body analysis only. These facts describe diagnostic derivation sites, not runtime token identity, resource liveness or new free effects.

The retained fact types and capacity accounting live in `ide/lifetimes.rs`. Checked lowering and finalization live in `ide/lifetimes/recording.rs`; changing the recorder must preserve event order, successful-body publication and executable-release parity. Compiler regressions separate general recording from selected retained facts in `tests/ide_lifetimes/retained_facts.rs`.


### Closed helper footprints

A no-growth borrow proof needs the actual initialization helper's effects, even when that helper has no lifetime note. `lifetime_summary/frames.rs` observes the existing summary traversal and composes complete checked scalar helper footprints bottom-up. It follows structural call dependencies of typed descriptor-to-element return candidates; helper names do not select semantics.

A present frame certifies complete possible caller-visible reads and writes and excludes unmodeled resource or external effects. Independent semantic declarations are frozen separately after all contract seed passes. Their bounded snapshot is not the evolving body-derived may-effect set: an excluded growth branch does not invalidate an otherwise closed guarded path. Frames reject owner or transitive declaration constraints at publication and use; missing summaries and over-budget constraints remain unknown. An append witness may cover only its own exact unguarded receiver-storage resize, never an additional free, reset, movement or resource effect. For example, a scalar write helper declared `@FreesArgument` cannot become a pure initialization frame, and an append declared `@ResetsTemporaryStorage` retains its reset operation. Its guaranteed full-write subset is separate: a conditional or partial write cannot stand for unconditional initialization of the entire element. A missing frame means unknown. Unsupported signatures, globals, escaped memory, opaque or native calls, pointer hops, recursion, cancellation and exhausted work publish no frame. Compiler default writes use their retained selected byte width; this does not infer a value from a write footprint.

Extend the exhaustive event observer and its negative controls when admitting more operations. Formal-relative byte slots remain bounded by the original formal pointee extent; actual local stores are omitted only with the existing original-root and exposure certificate. The domain caps bodies at 128 events, footprints at 32 and proof work at 8192, and stores no cross-request cache. `JAILINT_LIFETIME_STATS=1` reports visited events, composed slots, charged work and retained vector payload separately from latency measurements. A frame alone does not establish array bounds, backing ownership, lifetime or header separation at the caller.


A closed frame may instead carry a signed64 entry guard. The initial conditional slice selects the entry block's actual comparison and both typed load sites, checks the entire selected prefix and every feasible suffix, and rejects external writes, unknown effects, repeated joins or cycles. Comparison identity is an index in the current retained comparison vector, paired with the actual branch event; it is not a synthesized source span. Both slot values must be proven at the caller's actual invocation epoch before the condition can be discharged. A matching procedure identity alone does not grant an unconditional read-only effect.

This proves the growth helper's early return from its checked count/capacity comparison and supports composition with the checked initializer. The same `flow::states` driver handles excluded edges; other users exclude no edges. No new CFG walker, persistent cache or helper-name rule is introduced.

### Stack-backed append calls

The first caller slice combines these frames with a checked descriptor-to-element return body. At the actual call epoch, it requires exact count/capacity values, a captured data pointer into an original fixed-array stack allocation, sufficient byte extent, and a separate original stack allocation for the full descriptor header. The selected callee path must initialize exactly one scalar element, increment count once and preserve data/capacity without reaching growth or an opaque effect. Unknown bounds, aliasing headers, unsupported argument forwarding and exhausted proof work retain ordinary conservative behavior.

The returned pointer copies the captured backing lifetime; it is not a new live allocation. A separate logical association keeps later real growth and release connected to that pointer. Exact writes clear only mutable place contents that might overlap the element or count bytes. Previously captured SSA values and unrelated private pointer slots retain their values.

Ordinary CFG lowering always completes before optional refinements are installed. Proposed operands enter the final cell inventory, then bounded content-overlap queries and every replacement interval are validated without graph changes. Only single-entry linear event intervals can be replaced, together with their matched growth marker. An optional failure leaves all ordinary transfers intact and discards the optional certificates; it cannot suppress an unrelated diagnostic. Source spans remain diagnostic locations, while checked event indexes identify calls and growth-marker pairing.

Extend the existing summary observer, call-slot transfer and final-cell overlap query to admit another case. Do not add a parallel caller interpreter or infer separation from distinct borrowed roots. The first slice does not prove default/custom provider behavior, allocation extent after growth or general repeated append poststates.


Ordinary record elements use the same stack-backed caller proof. A whole aggregate Store can establish a full-byte initializer write only when its actual source is either original private unexposed storage of sufficient extent or an exact typed immutable constant-image region. Partial fields are not combined into a full-write promise; unsupported defaults, callbacks, pointer publication and unknown sources reject the closed frame. This source-read certificate does not establish provider behavior or permit growth-backed callers to bypass header separation.

### Conditional pointer footprints

A `Frame` distinguishes bounded typed accesses from conditional raw pointer accesses. The latter records a captured formal pointer, a checked signed byte displacement, the selected access width/type and event, and pointer values published by stores. It does not enlarge a `*void` pointee extent. A caller must separately establish original-object extent, liveness and noninterference at that access epoch; pointer publication remains visible even after those access obligations are discharged.

The initial raw slice uses the same event observer for straight-line void-return helpers. Unknown offsets, callbacks, globals, pointer/integer conversions and incomplete captured loads reject the whole frame. Ordinary helper and array-element composition explicitly accepts only the typed variant, so an empty bounded write list cannot hide unresolved raw writes. Extend `frames/raw.rs` and its prefix tests when adding a raw operation; do not convert a negative displacement into an unsigned formal slot. Existing frame work, cancellation and dependency checks still apply.

### Native-backed spare-capacity calls

The same caller proof can use an authenticated native `malloc` result for a spare-capacity append. The initial contract supports fixed64-bit C signatures on Linux and macOS, selected system C libraries, a positive constant byte request, and a checked nonnull success edge. The actual call result must be the descriptor DATA snapshot, with sufficient original bytes and alignment for the selected element. It does not turn caller-selected context or an arbitrary allocator callback into the default provider.

The existing flow traversal checks that the successful edge dominates the append. A unique acyclic path from acquisition to that exact call prestate must contain no intervening callback, release, growth or unproved memory access. Certified original stack accesses are separate from this newly allocated live object; this is an object-specific frame proof, not a noescape label. Final mutable-cell invalidation receives the explicit native-range certificate. Later free/grow operations remain in the ordinary CFG, and optional proof exhaustion retains the baseline graph.

Native contracts are selected in `lifetime_summary/primitives.rs`; applicability is in `no_growth/calls/native.rs`. Keep signature, library, target, failure, extent, alignment, alias and later-release controls paired when extending them. In particular, a same-named foreign `free` in another library supplies no must-release contract. The scope is bounded by the existing8192 caller proof-work limit and a128-item native body admission limit, including call-result metadata. There is no persistent range cache.

Contract references: [Apple allocation semantics](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/malloc.3.html), [GNU allocation and alignment](https://sourceware.org/glibc/manual/latest/html_node/Aligned-Memory-Blocks.html), [GNU release semantics](https://sourceware.org/glibc/manual/latest/html_node/Freeing-after-Malloc.html), and [Microsoft C-runtime release semantics](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/free). These justify native primitives, not wrapper names or provider capability hints.

The private aligned-allocation bit helper takes an unsigned byte prefix. Its two signed-prefix callers perform the ordinary checked conversion at the actual call, after allocation succeeds and only when the resize path needs realignment. Failed conversions retain runtime report effects; this factoring moves their report location to the call boundary and does not make an unknown signed prefix valid.


Machine-bit return relations are separate from pointer-origin and allocation-range proofs. The arithmetic reader uses exact typed private-load snapshots under both valuations of one captured Boolean formal. It publishes a bounded U64 expression graph only when every selected normal return agrees and the existing Frame observer proves no unrepresented public effects. Addition and constant subtraction retain their actual operator and modulo 2^64 semantics; a valid graph can wrap and does not establish a usable pointer.

Extend `lifetime_summary/address_arithmetic/bit_returns.rs` for checked expression operations, and the existing storage/Frame layers for new effects. Keep the two arms correlated, exact cast widths and declaration constraints intact. The graph is compilation-local, uses the existing8192 work and32-node bounds, and has no persistent cache or configuration switch. Native range, alignment, caller capture epoch and provider/context obligations must be discharged independently before a caller uses it for lifetime reasoning.


Completed Frame, append-borrow, native-allocation and machine-bit certificates are immutable `Rc` values within one lint run. Summary copies share their buffers; value equality still controls convergence. These references retain no compiler, prepared working state or backreferences. Reference identity is never a runtime allocation identity. Retained payload reports count an owned template once and exclude its Rc control header and allocator rounding; a successful template has one Rc allocation, while an absent certificate allocates none.


Lock transfers keep their small object/balance values inline and share only a present target Path. A targetless default transfer allocates nothing. Call substitution clones the local transfer value and replaces its target, preserving the original template. An actual target with no path steps gains an Rc allocation; this tradeoff avoids repeated nonempty path-vector copies without allocating for every no-lock body.

A conditional native-return witness composes an authenticated allocation with a checked bit transform and the actual prefix accesses. Each accepted invocation must determine every branch on its selected path; other inputs remain unknown. Prefix reads, writes, alignment and stored-pointer relationships are separate obligations. In particular, actual subtraction uses checked object offsets when proving a range; an addition of the corresponding wrapped constant remains only a machine-bit relation. Static enum-member lookup metadata has no runtime memory effect, while runtime field accesses retain their extent and effect checks.


An explicitly declared `PreservesUnpassedStorage` boundary can occur in a native
return witness only as a separate object-relative condition. It is not exported
as an empty Frame. The existing observer rejects public publication, and any
admitted prefix store must target the same allocated object. Private captured
spill storage must remain original and unexposed. The retained condition is
reauthenticated at construction and application with the same work counter,
including note/string and ABI checks. The acquisition call's ordinary global and
other-memory effects remain in the caller CFG; a returned range does not permit
suppressing those effects during a later whole-call refinement.

Returned-object requests use the shared ordered storage transfer for at most three
demanded typed fields of one invocation-bound object. A completed first capture
supplies address coordinates; a fresh replay tracks only those fields. Surviving
constant-image values are decoded together at publication. A guarded caller uses
the same typed constant transport when installing the checked post-call values. Every relative suffix store must
fit the original allocation at every possible returned offset and satisfy the
selected type alignment, including negative header offsets. These are conditional
byte facts: allocation success, lifetime, exposure and caller effects must still
be established before a diagnostic consumer can use them.

Call-context facts retain the exact hidden ABI operand before default arguments
run. They distinguish absent ABI, an evaluated pointer and the indirect-call
lowering's explicit null. Missing or conflicting rows are unavailable. The
selected hidden parameter is recorded separately; it is never inferred from a
numeric SSA ID or an ordinary formal parameter.

Context selection uses the compiler's real private pointer slot. Its checked
loads and stores enter the existing storage transfer after original stack storage
is validated. This gives address identity at the evaluated call epoch, not values
inside the pointed context or permission to preserve it across callbacks.

Context byte requests are collected after normal-return reachability settles and
before value inference. The constructor publishes its own requested return slots;
a caller never writes another procedure's summary. The returned witness keeps
the selected native acquisition, range and checked lifetime dependencies alongside
the typed values. It does not assert that the context remains unexposed or survives
later callbacks.

A guarded caller query installs those values after the ordinary call effects in
the existing storage transfer. It requires the exact current call result and a
nonnull edge that dominates the observation. Later stores, joins and opaque calls
still invalidate the watched bytes. The query leaves ordinary body captures
unchanged. Extend this path through the shared `Capture` transfer and indexed byte
coordinates, rather than adding a context-specific memory walk.

Stored semantic constants carry their checked type and either scalar bits or an
exact procedure identity. Numeric readers accept only the scalar alternative;
pointer-origin readers reject both. Procedure identity must be checked against
the current selected target and signature before it can drive another proof.
