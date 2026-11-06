# Allocator provider analysis

This domain records which captured procedure/data pair supplied an observed allocator callback result. It does not establish fresh storage, live ownership, temporary reset epochs or allocator behavior from capability bits.

## How it works

`Analysis::build` reads completed checked lifetime bodies from one compiler generation. `provenance(id)` returns possible callback uses, pointer return sources and observed inline receiver stores. A callback result can retain a checked signed byte displacement, such as the Pool block allocator's returned pointer plus eight bytes. Builder buffer stores and the bucket allocator's nullable fallback are other checked carriers.

The selected compiler allocator-protocol certificate supplies actual argument roles, result position, nominal operation type and enum tags. Neither procedure names nor structural callback types authenticate this protocol. Procedure and data values are captured independently: a mixed pair stays mixed, and overwriting a receiver after invocation cannot relabel the result. Constants are normalized using the selected operation type before comparison with the certified tags.

Each field observation keeps its typed byte slot and read epoch. Aggregate argument snapshots require an actual checked copy into a sufficiently large private stack root; the analysis preserves the original copy/read epoch across later argument evaluation. Context fallback records the selected context and guarded alternatives. Sites are bounded structural call chains, not unique runtime instances.

Only actual stack-root certificates establish private storage, and every access must fit the original extent. Copies and projections preserve canonical address exposure. Unknown callbacks invalidate potentially shared storage and exposed private slots; immutable SSA captures remain separate. Unsupported derived addresses cannot become new private roots, including after storing and reloading a pointer.

`uses` are possible observations, not guaranteed calls. `stores` describe writes at their recorded sites, not final field contents. `returns` retain guarded callback, null and unknown alternatives. `incomplete == false` covers normal pointer returns within the supported representation; it never promises absence of other effects or allocation freshness. Loops and later unsupported traversal may leave a bounded acyclic prefix of possible uses while outputs remain unknown and `incomplete` is set. `None` is unknown, never purity.

## How to change it

The public relation types are in `allocator_providers.rs`. `checked` prepares entry state and runs the bounded CFG; `calls` composes observed prefixes and callee exits, while `events` handles typed values and storage. `domain` owns storage and bounded values, and `bindings` substitutes actual captured arguments across checked calls. Add regression pairs before extending storage, control flow or callback roles. Keep protocol authentication in the compiler producer rather than guessing roles in this adapter.

Unsupported aggregate returns, pointer-hop output traversal, recursive/repeated callback instances and uncertain address arithmetic remain unknown. Extending them requires an explicit representation and caller obligations. Consumer integration and source-note removal need separate validation.

## Configuration and dependencies

There is no persistent cache or user-facing option. The analysis depends on checked body/AST identity, typed layouts, actual stack-root extents, semantic Boolean and arithmetic events, context selection facts and selected allocator-protocol metadata. It polls compiler cancellation and discards cancelled publication.

Internal limits include 8,192 transfer steps, 64 alternatives/observations, call depth 16, and Boolean expressions of at most 128 nodes and depth 16. Storage maps and candidate discovery also have finite bounds. Exceeding a limit widens to incomplete/unknown instead of claiming coverage. Compiler-only tests check real carriers, body-release parity, capture timing, aliasing, selected tags, arithmetic, loops, budgets and cancellation; they do not execute native programs.
