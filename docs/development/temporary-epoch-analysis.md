# Temporary epoch analysis

This domain describes conditional bump-storage lease transitions and typed snapshot observations. Rewinding or resetting a protocol lease does not prove a native free or physical release of its backing bytes.

## How it works

`Analysis::build` reads completed checked lifetime bodies from one compiler generation. The first contiguous slice supports a pointer base and signed 64-bit cursor and compatible signed length captures, with physically separate header slots. Unsupported numeric reinterpretations remain unknown. A contiguous protocol relates a returned base-plus-old-cursor pointer to the exact cursor advance. Backing extent, nonnull address, nonnegative size, alignment, arithmetic and separation between all protocol-maintaining state bytes and writable backing bytes remain explicit caller obligations unless independently proved. A capacity counter is not an allocation extent.

A `SnapshotObservation` records typed fields copied from one captured allocator instance at coherent read epochs. It is not a valid mark by itself. A `SnapshotTemplate` additionally identifies a compatible proved protocol and requires caller ghost state: the same instance, its current full-reset generation and its current rewind lineage. Restoration requires the resulting mark to remain reachable on that lineage. Returned field types and complete copy provenance matter; equal numbers assembled from unrelated marks do not create a mark.

`ForwardedMark` transports an existing opaque argument ghost through a certified complete aggregate copy. It preserves the original instance, frontier, generation, lineage and validity state; copying a stale mark does not validate it. Entry-copy evidence establishes private storage, while the caller must select the ghost from the actual argument evaluation/copy epoch. The entry spill cannot justify rereading a caller variable later.

Whole-copy provenance is captured before clearing the destination, including self-copy. Overlapping scalar writes, unsupported aggregate replacement and uncertain writes kill stored origins. Reconstructed or mixed fields do not create an origin. Initial transport excludes callbacks, object-state transitions and unmatched aggregate outputs. Its output separation obligation resolves the source ghost's whole actual allocator-state extent, not the size of the Mark value or a guessed callee type.

Full reset and rewind have different effects. Reset retires all corresponding leases and marks controlled by the selected frontier of its instance, then advances that frontier's reset generation. Rewind retires descendant marks and post-cut allocation identities. Earlier allocations survive only with a proved position before the cut. Reusing the same bytes cannot revive a discarded mark or allocation. Without ancestry evidence, other marks become unknown rather than being accepted from their numeric frontiers.

An epoch belongs to the captured instance and its particular physical frontier. Protocol keys identify checked proofs. Different procedures using the same instance and cursor share retirement effects; sharing only an object type does not merge independent cursor frontiers. Unknown aliases or partially overlapping cursor ranges must widen potentially affected ghosts rather than being treated as independent.

Instance identity comes from actual captured pointer values, including the selected context slot at its read epoch. Switching context cannot redirect an earlier mark to the new storage instance. Changing the base, capacity or instance pointer, or an uncertain callback, invalidates the corresponding protocol evidence. Structural sites identify observations, never unique runtime acquisitions or marks.

Only actual stack-root certificates establish private storage, with every access checked against the original extent. Aggregate return headers are borrowed typed output views, not owned allocations. Snapshot templates retain a result-header/state separation obligation covering the whole checked state extent, including the data pointer and capacity rather than only copied cursor fields. Typed byte projections handle physical overlap; unsupported access or assembly remains opaque.

`LoopBypass` is a separate conditional header-transition component. It requires the actual receiver address snapshot, readable/writable full receiver extent, zero entry bytes at the checked head slot and a null captured stop argument. The analysis specializes the checked CFG, identifies the real strongly connected loop component and its excluded header-to-body edge, and explores every feasible path under those assumptions. A reached callback, unknown barrier, nonreceiver access, unsupported branch, cycle or exhausted budget rejects the whole component.

Every guarded normal exit retains its ordered header writes and each source's captured read epoch. The real initial-page helper can initialize its original backing fields before restoring current fields, so a bypass is neither identity nor purity. It proves no traversal only under its hypotheses; it does not prove reset, rewind, mark validity, native free or physical release. Nonzero or unknown head/stop values remain outside applicability. Alias and overlay writes invalidate specialized cached bytes rather than reapplying entry assumptions.

## Current boundary and acceptance work

The first slice covers conditional contiguous protocols and snapshot field correspondence. Bump/reset/rewind paths with borrowed aggregate output writes remain unknown. Snapshot certification permits only its own framed aggregate output; extra aggregate outputs are unsupported until their complete state-separation obligations are represented. Its transition templates remain incomplete; absence of a relation never means purity or absence of retirement. Callers must discharge each obligation before using an effect.

The real paged carriers are part of acceptance, not a completed extension: allocation page growth, same-page restore, ordered page-prefix cleanup, callback interference, null temporary storage fallback to caller/default allocator, caller application of mark-forwarding relations across helpers, caller lineage replay and source-note-absent diagnostic parity still require their own checked proofs. A page-looking loop or absence of inferred frees cannot establish cleanup. Null storage yields no mark capability. Both temporary-overwrite settings must remain covered.

## How to change it

The public relation types are in `temporary_epochs.rs`; `domain` handles bounded typed values and physical slots. `entry` prepares actual roots, original extents and certified entry copies; `transfer` applies grouped checked events. `checked` owns CFG coverage, before-event comparison refresh and fuel, then composes compatible schemas under one publication budget. `paged` checks conditional loop bypasses and records structural excluded edges; full page-prefix traversal is separate. Add paired regressions before extending a protocol, a mark-copy operation or a caller obligation. Preserve the distinction between observations and certified templates, and between reset generation and rewind ancestry.

Shared compiler facts need separate review. Overlay selection can be classified only by an exact compiler certificate plus an independently checked physical projection; neither event adjacency nor field spelling identifies a harmless barrier. Assembly and surrounding callbacks remain opaque.

## Configuration and dependencies

There are no user options or persistent caches. The analysis depends on retained lifetime CFG events, actual stack-root extents, typed projections/arithmetic/Boolean relations, checked body identity and context selections. It reuses generic captured-value and structural-site types from allocator-provider analysis, without inheriting allocator ownership or behavior claims.

Transfer work, comparisons and materialization have finite budgets; expression trees are checked before cloning and limited to 128 nodes and depth 16. Cancellation discards publication. `TEMP_ALLOCATOR_OVERWRITE_MEMORY` changes real restore/reset control flow and is tested in both states. Compiler-only tests do not execute native programs.
