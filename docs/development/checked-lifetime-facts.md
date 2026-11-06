# Checked lifetime facts

Lifetime analyses share body authentication, structural event identities and original stack byte ranges. These facts do not prove allocation ownership, resource liveness, array capacity or a complete procedure effect.

The shared layer accepts a retained body only when its map key, recorded procedure, completed compiler body and syntax identity agree. Event identities identify checked instructions, not distinct dynamic loop executions. Stack roots come only from retained physical allocations and canonical slot aliases. Unknown original extents cannot authorize accesses. Inline projection checks signed displacement, overflow and the full access width against that original extent; a cast's destination type never enlarges it.

Scalar resource analysis and logical element transfers use these same primitives. Their values, exposure, unknown-call interference, guards, summary scheduling and joins remain separate. Borrowed aggregate output regions are not original stack roots. Exact equality copies preserve a captured address; unsupported pointer arithmetic remains unknown.

Results must retain their domain's meaning:

- A complete result covers every applicable normal-return alternative under its explicit conditions. It does not assert termination or absence of abnormal exits.
- Partial MAY observations describe witnessed reachable operations before an unsupported suffix; omitted operations and final outputs remain unknown.
- Cancellation, stale facts or exhausted proof work cannot publish a partial result as complete. An absent result is unknown, never purity.

Change `rules/lifetime_facts.rs` when adjusting these shared certificates. Preserve the scalar production consumer and element carrier regressions, including alias exposure, unknown extents, dynamic/out-of-bounds projections, zero-offset SSA reuse, stale bodies and executable-code release. Domain-specific operations belong in their owning analyses rather than flags on the shared layer.

There is no new configuration or persistent cache. Each caller supplies its existing work bound; facts depend on compiler-retained lifetime sidecars and target-checked types. Retained-size accounting reports owned collection capacity, excluding shared syntax and allocator/hash-table overhead.

`DefaultImage.zeroed` records the exact selected zero operation over its frozen byte width. It is independent of typed-leaf coverage, so a builtin descriptor can have known zero bytes without a complete leaf decomposition. A copied default image leaves this bit false, even if its bytes happen to be zero. Consumers must prove that the queried integer slot is fully covered, bound the write by the original destination extent and invalidate overlapping aliases. The bit grants no pointer capability, ownership or provider behavior.

Declared growth markers retain their exact following call index. For a checked ordinary call, a consumer can take the invocation prestate before that marker after validating its argument and type binding. The marker still updates ordinary poststate. Expanded calls are excluded: their effects have already occurred before the marker. Missing or inconsistent anchors retain conservative growth behavior. This association proves a call boundary, not no-growth or purity.


Typed immutable aggregate sources retain their exact selected constant-global address and byte extent. The producer checks the actual committed global's read-only flag, size and image length; unresolved speculative IDs contribute no certificate. Consumers can validate a full typed source read. At the actual aggregate-constant selection, the existing bounded default-image decoder also retains exact supported semantic leaves from that selected immutable image. A later full typed copy may use a matching leaf; missing or unknown leaves remain unknown. Generic read-only regions, including runtime configuration values finalized later by the compiler, remain extent-only. Neither form establishes writable storage, ownership, pointer validity or separation. The retained vector survives executable release and contributes its capacity to sidecar accounting.


File-indexed body selection validates the recorded procedure against its original map key before discarding that key. Lint consumers then share the same safe key/Done/syntax-identity check; CFG shape, normal-return coverage, budgets and rejection reporting remain consumer-specific. Stale file-index entries do not select a missing or mismatched body.


Pointer/integer bit-conversion facts retain the integer side's exact byte width from the selected IR conversion. Both 32-bit and 64-bit records are possible; a reader requiring lossless 64-bit address bits must check that width in both directions together with the selected target pointer width. For example, a 32-bit integer conversion cannot establish a 64-bit round trip. Width describes the bit operation only, not pointer provenance, a usable byte range or allocation ownership. Extend `ide/lifetimes/addresses.rs` when changing this recording, and keep raw IR width pairs and checked-body executable-release tests together.


Runtime cast checks retain their actual selected branch, and runtime check/trap reporting records an opaque barrier in the reporting block. Warning handlers can return; an empty source-event block cannot establish that their code has no effects. Disabled, unchecked and truncating conversions do not fabricate a callback. Closed normal-return proofs must either account for the failure operation or prove the exact cast-fit premise selecting its passing edge; the default handler's spelling is not a noninterference certificate. The central recording is in `emit_check_failed` and `emit_trap`, so noncast report paths share this boundary.

Formal-relative slot separation uses the same checked interval primitive as original-root ranges. Both ends must be representable; an overflowing interval cannot establish separation through the other interval's valid end. Different formal identities remain potentially aliased.

Selected integer addition, subtraction and bitwise AND retain their actual checked type, operands and operation. These are machine arithmetic facts; they do not assert nonwrapping arithmetic, allocation range or pointer provenance. Type-only expressions record no executed operation.

Arithmetic overflow reports retain the actual checked branch and a callback barrier in its failure block. Warning and fatal modes preserve the emitted continuation; absence of a runtime module, type-only evaluation and `#no_aoc` record no fictitious report. This prevents a memory footprint from treating a generated reporting callback as a closed operation.

Runtime context initialization writes its selected constant payload and type descriptor immediately before the first payload read, after the fallible temporary-storage acquisitions. Native calls, failure cleanup and command-line publication retain their order. This keeps the unpublished payload's initialization close to its use without assuming a default allocator or changing custom context defaults. Change `stdlib/internal/runtime/module.jai` and the selected-constant recorder together if extending this path; arbitrary runtime initialization callbacks cannot be reordered this way.

The private runtime storage-size getter returns `u32`, matching the supported nonnegative count domain. The compiler validates the existing `0..i32::MAX` option range before creating its four-byte transport global and before final refresh. Invalid interim getter reads now reject before compile-time execution can observe them, even with target runtime initialization disabled. Valid late workspace changes still refresh the same interpreted global. Runtime code widens this value losslessly to signed storage counts and unsigned allocation sizes; the sidecar remains extent-only until finalization, never an early numeric constant. Update `sema/runtime_config.rs` and `internal/compiler-api/runtime-config.jai` together when changing this private protocol.
