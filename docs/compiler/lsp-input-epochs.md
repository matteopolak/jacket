# Language server input epochs

Native sessions capture the filesystem responses used by a semantic transaction. An accepted epoch keeps those responses immutable so warm queries can reuse coherent inputs without rereading every closed file.

## How it works

A new capture records the actual first response for each operation and raw path. It preserves canonical path spelling and directory order separately from the normalized comparison proof. Repeated contradictory responses make the capture unavailable. Publication requires the existing semantic, selection, overlay and cancellation guards, followed by native validation of the actual capture. The sealed provider denies native parse-ahead and never reads through a missing operation or path.

Open-buffer changes remain immediately visible through versioned overlays without discarding accepted closed inputs. Source saves and known file events invalidate the native epoch before reading new native identities or saved configuration. Unsaved settings edits retain the existing saved-settings policy. A silent external change can leave an accepted answer unchanged until reconciliation discovers it; warm answers do not promise a disk check before every request. This is a deliberate change from the former strict silent-change policy.

Known native events retire affected original input histories, including exact build probes; last-good build metadata remains advisory. Captured-history invalidation is conservative, rather than a claim of precise semantic dependencies. A fresh post-event response can preserve an otherwise affected history row only when its raw operation, path and available response are identical; this does not grant answer authority or replace final guards.

A stored client URI already owns its current buffer. Syntax tokens and parser-only edits can remain useful while unrelated native coverage is pending; semantic refinements retain their individual guards. Requests through a different native spelling still validate the observed alias mapping. Exact native navigation and diagnostic authority do not follow from syntax-only output.

Opening or closing a source buffer preserves accepted closed inputs while changing the overlay manifest. A ready epoch starts a preserving capture before alias lookup. If identity coverage is already unavailable, the raw notification still owns its text and version; exact output waits for sealed identity hydration and a unique buffer identity. Raw edits and closes remain available during that wait. Background compile refreshes require an accepted semantic callback, not just a matching document version.

Saved settings retain their accepted client URI, native identity and local input history. Disk text and client-committed text are separate authorities; opening a settings buffer can use its supplied text when native text is unavailable. A watched refresh follows each stored alias, including a binding whose old physical target disappeared. Only a certified missing-key response permits ancestor-config discovery. An unavailable refresh retains no stale settings text and rejects dependent semantic publication, including virtual environments. Saved text, identity and retained proof storage count against the workspace byte budget. Provisional event captures own their input history separately; rejected snapshots do not enter the parent epoch or its reconciliation work.

The native loop services reconciliation between bounded request batches, including continuous request traffic, and while idle. Each slice advances a persistent cursor over reads, existence checks, directory listings, canonical identities, stamps and definite-missing keys. A difference invalidates the epoch before another answer can publish. Push diagnostics are scheduled separately; negotiated diagnostic, token, hint and lens refreshes use the existing coalescing queue. Shutdown and untrusted sessions do not reconcile.

Reconciliation limits operations, expected response bytes and cooperative elapsed time. One filesystem call can block beyond that time, and this does not establish a freshness deadline. A previously successful read becoming unavailable invalidates its epoch. The filesystem's `None` and boolean results do not distinguish every I/O failure from absence; only the explicit missing-key capability certifies resolving absence.

## How to change it

`semantic/input_epoch.rs` owns actual responses, exact consistency, sealed coverage and reconciliation. `Session` invalidation owns event transitions; `protocol/handling.rs` schedules client updates; `main.rs` grants recurring service opportunities. Preserve original actual histories when reusing compiler facts, and keep event, owner, buffer, fact-token, coverage and cancellation checks at acceptance. Do not enable direct native reads merely to satisfy a sealed coverage miss.

A genuinely cold program or alternate target can start a new native capture before its first compiler selection. This retains and validates the accepted original history together with new inputs. Owned accepted source/context presence suppresses that transition, so warmed document and target switches keep the sealed view. A failed selection never enables a same-request reset. Before selecting an old cache entry, a conflicting raw response already captured by the new transaction makes that entry a cache miss. It is retired before its old history can contaminate the fresh capture; accepted candidates still pass all existing guards.

A coverage miss queues its exact operation and raw path. Between-request work captures a successor and validates its original and added responses in bounded steps. Queries remain unavailable during that transaction; interrupted or changed inputs cannot promote it. Successful coverage expansion schedules negotiated client updates.

Read proofs retain shared byte owners; the epoch can keep a weak reference to a small owner without retaining its separate byte allocation after proof eviction. Expired owners are unavailable, never cached read failures. Discovery memos and retained compiler facts restore their original actual histories before a new capture can reuse them.

Diagnostic help owns a separate optional input carrier tied to the accepted semantic epoch. Missing hint inputs hydrate between requests; hint-only changes refresh help without retiring a valid compiler. Shared input changes revoke semantic authority. Active render transactions defer optional service, and required semantic inputs retire optional storage before reporting budget pressure. An oversized optional input with no durable proof owner stays unavailable until its parent resets instead of repeatedly rereading it. Rejected help never publishes a clean replacement diagnostic report.

The auto-import catalogue remains a separate, notification-governed advisory cache. Its explicit source view uses the original provider with current buffer and saved-settings snapshots, so suggestions may be newer or older than a semantic epoch. Catalogue scans stage their index changes and publish only complete, uncancelled results for the same buffer generation. They never authorize compiler facts, exact import resolution or diagnostic help. Certified missing discovery candidates are valid negatives; an unreadable existing candidate or unknown I/O rejects the catalogue commit. Existing traversal, item and shared-entry limits apply; the catalogue does not retain a second exact filesystem proof map.

Unavailable saved native settings use a separately budgeted recovery candidate. Service rotates through failed inputs and alternates with ordinary reconciliation; commit requires the same native-event revision and saved owner. Ordinary typing does not starve recovery. While saved-input recovery is pending, its alternating ordinary replay lane also receives one operation per slice, including during recovery backoff. Budget refusal waits for workspace capacity instead of repeatedly reading a known oversized file.

Broader context recovery remains an acceptance gate in the current integration. An unavailable path must not mix live data with a sealed answer.

## Configuration

There is no user setting. Carrier limits cap operation/path entries at 32,768 and directly retained capture storage at 16 MiB. Larger reads may borrow an existing accounted proof owner. An oversized coverage miss requires a later strict native transaction with a durable proof owner; hydration does not repeatedly retain and discard the same large payload. Reconciliation slices allow 32 operations and 256 KiB of expected response payload, with a cooperative 1 ms check; the native loop polls ordinary reconciliation every 25 ms. Active parent coverage receives consecutive bounded opportunities between input batches or idle receives, returning to ordinary polling after completion or unavailable input. These are work limits, not I/O deadlines or total process memory limits.

## Dependencies

The implementation uses the compiler `FileSystem` interface, existing semantic comparison proofs, versioned document overlays, client file events and negotiated LSP refresh requests. It creates no watcher, compiler worker or reconciliation thread.

Retained probe and hint input owners charge shared read bytes once only when their own
comparison proof holds the exact same raw-key `ReadBuffer` allocation. Equal bytes from
a separate read still count separately; actual-response metadata and weak-owner metadata
remain charged. This keeps the existing probe budget without changing input authority.

Comparison proofs and dependency sets share immutable nodes between snapshots. A changed
record detaches its map; identical shared histories need no repeated union. This only avoids
copying proof storage: first-response conflicts, raw read owners, native replay and final
epoch/overlay/cancellation checks retain their existing meaning. Arc root metadata remains
charged, and payload accounting stays conservative per retained owner.

Workspace polymorph discovery opens its first transaction before traversing project roots,
preserving the accepted input history. After full discovery and final epoch acceptance, the
workspace index suppresses repeated initial transactions even when summaries are deferred.
That lifecycle bit grants no rows or completeness: every query still validates its complete
workspace context, and later missing inputs use normal bounded hydration. Clearing the index
allows initial admission again.

Entry discovery collects configured and default workspace entries before computing loader
fallbacks. It resolves open-file fallback programs only when no entry was found, avoiding
unneeded library-tree scans whose roots would be discarded. Keep every open file's config
scan before that decision so a later configured file still supplies its entry programs.

Under metadata pressure, an epoch may borrow earlier read payloads from the exact raw-key
`ReadBuffer` already retained by its discovery proof. The proof still charges those bytes;
the epoch charges weak-owner metadata, and replay still counts the full read length. A
different equal-byte owner or an actively borrowed proof cannot supply that lease. Old
actual-input snapshots stay unchanged, and losing the last real owner makes reuse unavailable.
Workspace discovery captures its own proof so accepted compiler and summary owners retain
the required payloads after the request ends.

Each open document admits its first exact link-discovery context before resolving imports.
The accepted marker only suppresses another initial capture; link history, projection and
final epoch checks still authorize every result. Text replacement creates a new context.
Pending or failed inputs cannot reset themselves through a link request.

The first whole-program compile-time cost command similarly admits its output-location
context before projecting results. Accepted repeats use sealed inputs; an unavailable or
rejected command cannot mark that document context accepted.

Definition targets retain whether the selected compiler source is a file or virtual input.
File targets reuse their compiler-owned file identity while still observing canonical
responses and every open-buffer alias. Parsed paths and virtual origins retain general
filesystem projection; all results still pass the enclosing semantic acceptance checks.
