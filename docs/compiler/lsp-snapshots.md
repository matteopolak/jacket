# Checked reference snapshots

The language server retains checked reference facts separately from full compilers. References and linked editing can answer after checks of other open programs evict the original compiler. Rename still uses its exact compiler path.

## Ownership and queries

`jaic::ide::meta::ReferenceIndex` owns selection spans, declaration flags, member links and local target numbers. Extraction resolves compiler entities, procedures and types once; those compiler identities never become keys outside the exported value. Queries preserve member precedence, smallest overlapping selection, overload-set closure, dollar-prefix trimming, declaration inclusion, ordering and deduplication. Compiler tests compare every byte position in representative sources with `Compiler::ide_references`, then repeat after dropping the compiler.

`semantic::Analysis` and each metaprogram `Program` cache an immutable index within one semantic query. Every mutable query boundary invalidates those views before and after the callback: completion can resolve an existing alias, and definition can add a missing callee fact, without changing collection lengths. All record construction and source identity reads precede the final native freshness guard. A successful guard commits the staged owned records without further filesystem access; failure leaves prior records untouched. Exact live references take precedence over retained records; rename uses its exact compiler path. Focus changes swap the index and an opaque accepted-context token with the compiler.

Immutable call-target syntax descriptions may be shared across refreshed graphs only when the callee path, declaration offset and complete source bytes match. Call edges and their mutable-fact acceptance token are rebuilt.

Snapshots retain that token without retaining the compiler or its IDs. A query of an older compiler can enrich its own prior accepted record while the user is typing different text; the enriched facts become available if the exact inputs are restored. It cannot replace a record from a different context unless all relevant current overlay inputs match. Query callers establish the program root, ordered import directories and native target before publication; workspace/configuration transitions clear incompatible ownership. A cancelled query retires the mutable compiler from reuse and publishes nothing, leaving previously accepted immutable records available only under their existing validation rules.

`reference_records::Store` owns the server's records. Documents checked in the same selected program share a snapshot. It keeps the source text needed to convert byte spans to UTF-16 ranges and maps those sources through the session's client URI handling. It does not retain a compiler, interpreter or service child.

A record is exact only when its program root, ordered import directories, relevant open overlays and queried source still match. Comparisons use content equality, not a hash as identity. Relevant overlays include failed lookups and directory searches recorded by `OverlayFs`; changes to unrelated open programs leave the record usable. These checks apply to the entire program, so an unchanged document cannot offer linked edits while an imported declaration has a pending change.

Workspace, trust, check-limit, service-result and panic-reset transitions retire incompatible exact authority. Saved configuration has its own overlay lifecycle: opening, saving, closing or receiving a disk notification retires exact contexts; unfinished config edits leave the saved snapshot in force. Environment option callbacks receive `(filesystem, root)`. All dynamic stdlib/prelude discovery must use that supplied filesystem, so its actual first responses join the accepted proof. Producer registry selection and compilation of root, module-parameter and generated-child identities use the same observer. Inconsistent precompile observations reject the attempt before compiler work; observations acquired during compilation must agree before acceptance. Other callback state must remain stable until the corresponding configuration notification or reset.

## Native input proof

Notifications are proactive invalidation hints. Only the selected compiler context is validated; unrelated bounded entries remain candidates until selected. `Cache::query` issues a private borrow-scoped grant after merging current discovery proof and validating the selected context. The semantic transaction consumes it once, checks interruption and sticky consistency before the callback, then fully replays the accumulated proof before accepting any answer. This removes an immediately duplicated precheck without caching validation across requests. Before a native cache, snapshot or negative result answers, the server replays the filesystem operations its context observed: exact read bytes (including unavailable reads), file and directory tests, directory listings, path identities and stamps. A metadata-preserving rewrite therefore invalidates a context. Validation visits only observed dependencies; it does not crawl the workspace. The first response to each operation remains authoritative for the transaction. Contradictory observations or failed pre/post validation make that request unavailable; the next request can start a fresh transaction.

Project discovery and `jacket.toml` reads use the same observation representation. Actual discovery responses are captured before compile-input memo publication. An input memo miss discards unproven helper scans and recomputes discovery; decoded configurations are read afresh. An exact discovery scan cannot consume an advisory hash-only open-file scan. The latest session discovery proof is retained separately and reported by `Session::discovery_proof_bytes()`.

`semantic::CheckedInputs` stages current overlay and ignored-path identities through an observer before its final replay. It combines those actual responses with the immutable retained history; identity mapping cannot change the filesystem after validation and still authorize an answer.

`semantic::CheckedInputs` freezes observed dependencies and relevant exact overlays without retaining compiler IDs. Its opaque environment authority cannot survive incompatible settings or reset transitions. Build probes, retained call graphs, generic-instance summaries and skipped alternate-target spans use this proof. Call graphs also capture an Analysis-wide mutable-fact acceptance token: a later mutable query retires its previous view, and cancellation cannot accept a replacement. Unrelated Analysis owners and evicted compilers leave their accepted tokens stable. Generic rows are frozen by compiler `release_lowered_code`/`ide_freeze_polymorphs`, so later exploratory instantiation does not change those summaries. Build discovery distinguishes a completed producer, positive child membership, each child's check status and whole-workspace success. A checked present child remains useful when another child fails; failure never proves that an absent child is standalone. Exact probe retirement removes its membership and input memo together.

Directory enumeration merges disk entries with unsaved overlays and inferred directories. On proven compiler reuse, the compiler filesystem and its input manifest move together, so later queries read current bytes of previously unrelated buffers. Generated module stubs remain separate. Native validation also checks raw overlay spellings, including new aliases admitted by an unrelated-input cache hit.

The native editor forwards root watcher events and the protocol schedules push diagnostics after watched-file notifications. Unnotified changes are detected on demand; they do not themselves schedule proactive diagnostics. The VS Code watch pattern is not a universal dependency watcher. Browser filesystems retain their declared immutable-environment/reset lifecycle and validate open overlays separately.

## Diagnostic hint authority

Render-time unknown-name/member hints use `Compiler::with_name_suggestion_in` with an independent `HintContext` and an isolated observer over the selected compiler's overlays, generated sources and base directory. All error and warning enrichments in a batch complete before its proof is frozen; the existing first-50-error limit remains for unknown-name/member enrichment. Operator-import help remains available beyond that limit, as it was before deferral. Contexts, indexes and rendered hint caches are discarded after the query. Generic hover/reference cache selection therefore does not replay unrelated hint scans.

The semantic transaction unions every hint batch used by its callback, including across Program focus changes. It accepts the owned hinted result only after graph refresh, full semantic validation, exact hint validation and interruption checks. Hint-only invalidity rejects that result while leaving a semantically current compiler reusable. Nested queries share the hint transaction, and scope exit clears it on success, failure, cancellation or panic. A callback that uses no hints performs no hint replay.

Acceptance is per semantic query. Later lint/refactor queries do not retroactively invalidate already accepted immutable rows; this does not promise one atomic snapshot across every document in a publication batch. Missing-operator checks retain owned normalized operand/home-module names and defer only the external import search; actual namespace/type lookup remains semantic. Their import ranking uses the validated render context, consistently with name hints. Discarded dead-branch diagnostics never trigger this search. Build-time rendering and module-not-found hints already embedded during checking retain semantic input proof. The LSP forces `HintIndexMode::PerCompiler` before compiler construction; cloned options preserve it in workspace children and host twins. Their raw hints therefore record actual index reads instead of borrowing a process-wide stamp-only index. The render split does not remove those semantic reads or weaken their authority.

Checker findings, unreachable ranges and lints for one diagnostic report are gathered from one selected Analysis. Lint configuration is captured before selection; parse-invalid or excluded files keep the existing lint skip. The full semantic and hint guards still run after collection. A failed selection, validation or interrupted lint pass returns `DiagnosticsUnavailable`; it cannot replace a prior error with clean diagnostics. Normal native parser errors remain diagnostic results, while a session without an Environment is explicitly syntax-only.

Push omits unavailable documents; pull returns the standard cancellation described in [the protocol boundary](lsp-protocol.md#diagnostic-availability). Hint-only failure does not retire a semantically valid compiler. This is per-document acceptance, not one atomic snapshot across a workspace response.

## Exact and advisory caches

| Owner | Authority and lifetime |
| --- | --- |
| Full semantic analyses and reference records | Exact context, captured native responses, relevant overlay equality and lifecycle transitions |
| Build probe registry | Exact checked input proof; failed attempts distinct from completed no-child results |
| Call graphs, generic summaries, alternate-target negative spans | Shared checked input proof before reuse; stored source ranges use exact source text |
| Syntax parse results and document analyses | Exact source bytes; hashes only select candidates |
| Last-good build hints | Explicit advisory metadata; detached completion/definition/hover analyses cannot publish exact references, rename or current build facts |
| Workspace symbols, auto-import suggestions, shared stdlib/module indexes | Notification-driven advisory indexes; native silent edits may leave suggestions stale until invalidated |

Advisory indexes do not establish current compile-input or accepted semantic authority. The server does not promise that every suggestion reflects an unnotified filesystem change.

## Memory and fallback

The store has a separate `16 MiB` accounting budget, defined by `reference_records::BUDGET`. Shared snapshots count once within the store. Accounting includes index vector capacities, referenced source bytes, relevant overlay bytes, captured filesystem response bytes, path storage and conservative tree metadata. Compiler retention charges its observed input proof too. Text shared with compiler/parser state is charged to the snapshot as well; this is a retention budget, not a bound on process RSS or allocator overhead.

The oldest checked document records go first when the budget is exceeded. An individually oversized snapshot is not retained. References first inspect an existing exact compiler, then a retained record. A missing or invalid record falls through to the existing compile/check scheduling; under native deferred scheduling, an absent compile queues a check and the request is temporarily unavailable. Compiler eviction preserves records within this budget; deliberate record eviction does not promise persistence. Linked editing additionally requires the existing single local binding, same document and exact spelling checks.

Build probes and their last-good advisory metadata share a separate `16 MiB` budget. Oldest exact probe records retire first, then advisory metadata. One accepted oversized probe may be consumed transiently by its current request, then its exact views and input memo retire at request exit; a budget limit never manufactures a no-child result. `Session::build_probe_counts()` reports retained count, accounted retained bytes and transient bytes separately.

Call graphs, generic summaries and alternate-target negative spans each enforce a conservative `16 MiB` retention cap including proof bytes and owned ranges/names. Call graph extraction can return the current graph without retaining an oversized record; generic rows are merged for the current answer before retained summaries are trimmed. Dropped skipped spans merely require another target check. These budgets do not bound transient compilation or process RSS.

## Changing the contract

Change symbol resolution in `crates/jaic/src/ide/meta/references.rs`, keeping the live compiler query and owned index equivalent. Add oracle fixtures for new aliases, generated syntax or member behavior. Change snapshot ownership and pressure behavior in `crates/jai-language-server/src/reference_records.rs`. Filesystem operation capture lives in `semantic/observations.rs` and `semantic/overlay.rs`; shared accepted input authority lives in `semantic/checked_inputs.rs`, discovery transactions in `discovery_inputs.rs`, and build outcome retention in `semantic/probes.rs`. Preserve first-response consistency and operation semantics when adding filesystem methods. Keep disk hints in `session/config.rs` distinct from overlay lifecycle in `session/documents.rs`.

Native fixtures must own the ancestors consulted by nearby-name help. `test_support::lsp::project` provides that private ancestry while retaining cleanup of the outer scratch tree; placing a failing program directly under shared temporary storage makes unrelated tests legitimate observed dependencies.

Run focused checks with an isolated target directory and the checkout's stdlib:

```sh
cargo nextest run -p jaic --test ide_reference_index
cargo nextest run -p jai-language-server --test linked_editing --test disk_freshness
```

The store uses the compiler's IDE facts and source map, the session's discovery and URI identity handling, and `OverlayFs::Reads`. It adds no external dependency or user setting. Native and synchronous browser sessions use the same implementation; deferred request scheduling determines when retained records are needed.

Use `Session::reference_record_counts()` to inspect retained document records and accounted bytes independently of `compile_counts()`. See [request measurements](../tools/lsp-latency.md) for useful-answer checks and comparable process runs.

Reference answers stage client URI and range mapping before the final native guard. The mapper records actual canonical responses for the target and open spellings; retained records validate their original input history together with those mapping observations before returning owned locations. A changed alias cannot reinterpret accepted compiler spans after validation. An unavailable reference lookup may return no locations; a later stable request must still recover the useful answer, including after compiler eviction.

Type-definition and related diagnostic/lint locations are projected inside their selected semantic callback. Exact reference, definition, type-definition and diagnostic entry points also record request URI to open-buffer identity before selecting the stored buffer.

Definition projection runs inside the selected semantic callback, including the offset shift for an older accepted compile. A rejected final guard is distinct from an accepted empty answer: alternate-target lookup may continue only from the latter. An unavailable transaction never triggers an immediate recheck to obtain an answer. Cache construction also combines already captured discovery and raw input identities before starting compiler work; contradictory first responses reject construction without an extra filesystem replay.

Call-hierarchy projection keeps each scanned graph's selected input context, including its first URI/overlay mapping responses. Incoming-call answers include graphs that establish no matching call. After mapping targets, the server combines their original histories with current projection and discovery observations, rejects disagreement, and performs one final native replay. Per-graph root, import directories, environment authority, overlays and accepted fact ownership still apply. Target syntax uses retained compiler text, or equal current open-buffer text; it does not reread an external file after acceptance. These request-local contexts are discarded with the answer.

Preparing a call-hierarchy item resolves the definition and parses its target declaration within the same guarded semantic callback. A resolved nonprocedure returns an accepted empty item set. Declaration selection and fallback lookup can use current client syntax without a semantic provider; their alias traversal is observed and validated, and they read only stored client buffers. External target files are never reread after a semantic definition has been accepted.

Documentation-link definitions and compile-time cost locations also project their targets inside the semantic callback. A selected live reference transaction is terminal even if its final guard fails; retained facts or a first fresh compile are considered only when no live context was selected. This preserves the difference between a stale cache entry discovered before selection and inputs changing during an accepted query attempt.

Lint quick fixes and fix-all actions gather lint payloads and project secondary diagnostic locations inside the same semantic callback. Their accepted actions own the resulting URI and UTF-16 ranges. The action entry point also observes request-alias selection; syntax-only actions validate that mapping before returning.

Literal `#load`, `#import,file` and `#import,dir` links resolve open-file identity, existence and target URI in a local observed transaction without compiling. Named module imports use the central exact compile-input discovery result and observed options. Deferred or failed discovery yields no link; it does not substitute advisory import directories or a default stdlib. Recognized links remain terminal definition/hover providers even when unavailable, so another provider cannot immediately retry their failed transaction.

## Historical limit notices

Opt-in limit notices describe an accepted build that reached its compile-time limit. They are historical events, separate from current diagnostics. Acceptance freezes the original compiler source URI and UTF-16 range; draining the queue performs no filesystem lookup and cannot remap an old event through a newly retargeted alias. Source edits or later budget changes do not rewrite queued events.

`semantic::notices` retains only located messages, with a `16 MiB` budget including owned URI/message capacities and queue storage. Oldest events retire first under pressure; an oversized event is discarded. `Session::limit_notice_counts()` reports queued events, accounted allocation bytes and pressure retirements, independently of RSS. Disabled subscriptions collect nothing. Resetting the compiler cache clears events while preserving the subscription; trust changes preserve configured check limits. Interrupted or inconsistent builds publish no events, including interruption during the final native replay.

Change event retention in `semantic/notices.rs` and accepted-build publication in `semantic/cache.rs`. Keep identity capture before the final guard and queue mutation after it; do not retain compiler sources or input proofs merely to locate historical notices later.

Last-good build metadata can still support advisory completion and ordinary symbol definition when a producer later fails. A module import string is different: its target claims the current import context and therefore requires successful exact discovery. A failed producer leaves that directive link unavailable even when the ordinary symbol provider can use an explicitly advisory last-good context.

Exact semantic request consumers normalize a physical request URI to the stored client key before reading text or offsets. `projected_document_query` retains and replays that request mapping around use; native retargeting rejects the output, while ordinary request errors stay errors. Highlights and rename compare against the stored key, so opening through a symlink does not suppress valid results requested through the physical path. Private helpers that receive an already normalized key avoid repeating the entry transaction.

Compile-input ownership also observes open-loader alias traversal and the configured entry's actual `is_file` predicate. A transient absence cannot select another root while the proof contains only restored-file responses. Graph owner grouping and generic-summary scheduling use the same observed identity boundary; advisory path completion keeps its separate best-effort root selection.

Signature help copies procedure and parameter documentation alongside signatures in the original compiler callback. Later rendering consumes owned path, declaration offset and comment strings. It never resolves an earlier signature's compiler-local file number in a newly selected Analysis.

Ordinary hover captures macro expansion facts alongside its selected semantic facts when the current own-target Program exactly matches the client view and documentation rendering needs no link-resolution query. Older, alternate-target and advisory hovers, and documentation that resolves links, keep lookup after documentation enrichment. This avoids selecting the same compiler twice without guessing whether syntax can contain a macro.

Warm references can use an already published immutable index from the cache's first exact quick candidate. The current focused Program must own the same index and accepted context; matching another Program is insufficient when sources are shared. The answer freezes current live input authority, maps locations, and validates its union with the original record and discovery history once. Success changes no compiler, index or record. A failed admitted guard is terminal for that request and retires only that candidate's reuse eligibility; a later request can rebuild, while restored independently retained facts remain available under their own proof.

Rename preparation and edits share one checked selection. The selected compiler first checks member eligibility, then exports references, verifies a declaration with the requested spelling and projects every location before the final semantic guard. Preparation returns its owned range; rename builds edits from the same accepted locations and keeps the direct open-buffer spelling filter. No rename plan is retained across requests.
