# Content relations

This domain relates a returned string's bytes to a checked byte-copy operation. It tracks content correspondence separately from allocation ownership, resource identity and formatting grammar. A copied result is not automatically fresh storage or a formatted string.

`Analysis::returned_copy` returns alternatives covering every normal return for each supported result position. A relation records the incoming string formal, typed descriptor projections, complete returned range and structural read/write epoch. All alternatives must satisfy a consumer's query; selecting one alternative is not an unconditional source guarantee. `incomplete` identifies unsupported string result positions. `None` is unknown, never purity or absence of mutation.

Storage ownership comes only from retained actual stack-root certificates: canonical physical slots and stack allocations, with their original byte extent. A derived pointer is not a new allocation. Selected projections and constant offsets stay relative to that root; every load, store and initialization must fit its original extent. Unknown extents, dynamic offsets and oversized type reinterpretations yield unknown. Source locals, default initializers and the attribution-only `stack_places` list cannot create private storage.

## Read and write epochs

A copy captures the source bytes at its actual read epoch. For example, `copy_string` allocates before copying; an allocator callback may have changed source memory. The relation does not relabel those bytes as immutable procedure-entry content.

The selected intrinsic currently has move semantics and supports overlap. Its read snapshot describes bytes before the copy's writes, including overlapping moves. Later writes to the original source do not retroactively change that snapshot, but can change the destination if their actual byte ranges overlap it.

Later destination writes invalidate a snapshot. A resolved local write is harmless only when its containing storage is positively private and unexposed. Writes to exposed locals, descendants, whole objects or partial overlays discard snapshots because an opaque returned pointer may alias that storage. Recognized string-header writes can instead export a header/destination separation obligation at the actual store epoch; caller-visible return headers are never relabeled private. A supported source write can retain it only when its exact store range is proved disjoint from the destination or an explicit separation requirement is exported. A different unknown pointer write or opaque call discards the content fact. Read/write masks and a zero count do not certify resource or allocation-generation preservation. Only a selected closed operation contract can justify that exemption; a selected bounded write with count zero has no accesses or hidden effects and preserves the snapshot. Negative or out-of-extent local-slot addresses follow that same unknown-write path; resolving a slot root does not establish a harmless write. A source write outside the copied source prefix cannot be declared safe merely because the original source and destination ranges were disjoint.

## Checked flow and frame conditions

Only current successfully checked bodies with matching syntax `Arc` identities participate. Selected copy metadata identifies destination, source, count and overlap rules; read/write masks, procedure names and semantic notes cannot establish byte equality.

Typed selected field projections describe string descriptor slots. Private descriptor copies capture count and pointer values; formal substitution uses checked argument positions and immutable scalar operands. A by-value aggregate with multiple actual arguments remains unknown without an evaluation-time copy certificate. Pointer-valued opaque call results are captured identities, not freshness or disjointness proofs.

Frame requirements identify relevant descriptor headers by procedure, private/output place and bounded call context. They pair those headers with the actual copied destination or later written range. Raw string data may overlap a descriptor header, so a descriptor type alone proves no separation. Consumers must establish these obligations for an actual occurrence before using the relation.

The initial slice covers full input-string copies and zero-byte copies. Byte-offset multiplication, accumulated offsets and call substitution must fit a signed 64-bit displacement. Interval separation also requires a representable relative displacement and supported nonnegative spans, so wrapped pointer arithmetic cannot become mathematical disjointness. Negative or unsupported copy counts cannot justify interval disjointness or preserve an earlier snapshot. Partial copies, borrowed returns, overlapping destination overwrites, unknown effects and unsupported conditional exits do not become full returned-content guarantees. Builder segments, general append relations, formatter interpretation, ownership and allocation freshness need separate proofs.

Retained body selection requires the map key, recorded procedure, completed checker state and original syntax identity to agree. A mismatched sidecar is unknown, even if its syntax still matches.

## Extending the analysis

`checked.rs` prepares entry state and runs the bounded CFG; `calls.rs` composes selected copies and callee descriptors, and `events.rs` handles typed values and writes.

The public contract is in `content_relations.rs`. `domain.rs` owns captured values and slots, `bindings.rs` owns formal/call-frame substitution, and `ranges.rs` owns snapshot invalidation and frame requirements.

For example, after `memcpy(dst, s.data, s.count); return .{s.count, dst};`, a consumer can query a complete copy of incoming string zero at that operation's read epoch. It must still discharge the returned range's validity and separation requirements. Returning `s` without a selected copy operation establishes no copied-content relation.

Extend byte ranges and interval composition before adding segmented copies. Keep source and destination epochs distinct from mutable descriptors. Extend selected-operation facts at their checked compiler producer; do not reconstruct executable IR or recognize a convenient wrapper spelling. Add a positive proof and a matched alias, partial-write or unknown counterexample with each extension.

## Configuration and dependencies

There is no persistent cache or user-facing switch. A build belongs to one compiler generation and depends on checked lifetime events, selected memory-copy semantics, typed layouts, checked scalar arithmetic and boolean facts. Unknown calls after a copy invalidate it. Recursive/cyclic queries, unsupported operations and exhausted state, field, relation, condition, call-depth or expression budgets yield unknown. Boolean trees are checked before copying operands into a larger expression. Cancellation discards partial results.

Regressions include actual checked `copy_string` and `copy_temporary_string`, IR-release parity, captured read epochs after callbacks, overlap-safe moves, later source/destination writes, header overlap, named arguments, borrowed/partial returns, opaque calls, expression bounds and cancellation. These are checked compiler proofs; no native program execution is required. Consumer integration and source-note removal require separate review and caller-obligation tests.
