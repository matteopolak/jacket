# Element transfer analysis

This domain proves an unordered-removal relation over logical array entries. Logical identity is separate from payload equality and allocation relocation: two equal values can be different entries, and the last entry can move without moving the backing allocation.

`Analysis::transfer` returns a conditional exact relation for a checked procedure. The receiver and selected-index positions refer to checked formal parameters. Structural `(block, event)` witnesses identify the count and element writes. Its separate preconditions are `0 <= index < incoming_count` and disjoint storage for the descriptor header and its element data range; on such calls the output count is `incoming_count - 1`, output position `index` receives the old last entry when it survives, and every other surviving position retains its old identity.

For example, removing position one from identities `[A, B, C, D]` produces `[A, D, C]`, even if all four payloads compare equal. Removing the last position only shortens the sequence. Removing the sole entry leaves no surviving output position.

Storage ownership comes only from retained actual stack-root certificates: canonical physical slots and stack allocations, with their original byte extent. A derived pointer is not a new allocation. Selected projections and constant offsets stay relative to that root; every load, store and initialization must fit its original extent. Unknown extents, dynamic offsets and oversized type reinterpretations yield unknown. Source locals, default initializers and the attribution-only `stack_places` list cannot create private storage.

## Checked proof

The analysis accepts successfully checked bodies whose syntax `Arc` still matches the current procedure. It selects pointer-to-array descriptor and signed-index formals by type. It uses compiler-derived inline field projections and selected pointer subscripts, rather than recognizing procedure or field names.

Descriptor count and data slots come from the language's checked array layout. A descriptor type does not prove that its header and data ranges are disjoint: raw views can point into their own header. The relation exports this separation requirement explicitly, and its consumer must prove it for an actual occurrence or keep the result unknown. Copied descriptors sharing one backing range are a different alias case. Scalar values and pointer identities are captured at their actual load events. Integer operations retain their checked type: constant folding respects machine width/sign, and overflow or unsupported arithmetic yields unknown. Count-relative signed 64-bit arithmetic is retained only when its whole range is nonoverflowing under the valid-index precondition. Comparison literals use the recorded primitive width and signed comparison semantics. The count is tracked relative to its incoming value. A proof requires exactly one write of `old_count - 1`, an unchanged backing pointer, and one full typed element copy from `old_count - 1` to the selected formal index. Other writes through unknown aliases reject the proof.

The valid-index precondition can settle checked boolean branches in the assertion expansion. This does not assume an assertion failure cannot return: failure paths are excluded only when the explicitly stated precondition makes their condition false.

Retained body selection requires the map key, recorded procedure, completed checker state and original syntax identity to agree. A mismatched sidecar is unknown, even if its syntax still matches.

## Extending the domain

Keep the CFG driver separate from computed-value transfer: `computed_value` handles checked arithmetic, pointer offsets and Boolean relations, with failures propagated through the enclosing proof. Both comparison refreshes remain at their original event boundaries.

The public relation lives in `element_transfers.rs`; the checked CFG driver is in `element_transfers/checked.rs`, and symbolic numbers, addresses and predicates are in `element_transfers/domain.rs`. Extend the symbolic transfer algebra before recognizing a larger protocol. Keep captured address values and read epochs distinct from mutable descriptor slots.

A partial field copy cannot become a full logical-entry copy. A `Copy` must preserve the actual address; a nonzero `#as` pointer adjustment is a checked field projection instead, so it cannot inherit the original backing identity. Zero-offset casts can preserve it. Buffer replacement is not removal. A second write, an opaque call, an unsupported alias, a repeated CFG block or exhausted work bound produces unknown. Missing summaries never establish that a procedure leaves entries unchanged.

The initial slice does not infer arbitrary permutations, ordered shift loops, keyed containers, copied-descriptor alias protocols or interprocedural wrappers. Those require separate relations and regression evidence. Consumers must preserve the valid-index precondition when using the result; a relation alone does not authorize removing a source contract or changing a diagnostic.

## Configuration and dependencies

There are no user-facing options or persistent cache. The analysis is local to one compiler generation and depends on the selected target, checked lifetime events, typed descriptor layouts, selected field byte offsets, selected subscript operands and semantic boolean/comparison facts. Cancellation discards partial analysis. Finite block/work and boolean-tree node/depth limits are analysis cutoffs, not evidence of non-mutation. Boolean parent cost is checked while borrowing operands before copying.

Regression fixtures include the actual checked `array_unordered_remove_by_index` carrier, renamed synthetic formals, first/middle/last/only positions and duplicate payloads. Negative fixtures cover lookalike user structs, wrong source/count, machine overflow, shifted `#as` pointers, unexpected and aliased writes, partial copies, loops and opaque index computation. Native programs need not execute; the proof uses the compiler sidecar and is checked again after executable IR release.
