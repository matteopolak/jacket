# Language server architecture plan

**What this is:** the target architecture for `jailsp` and the order to get there. Each claimed gain is tied to a measurement below. Nothing here is built yet.

## 1. Today

| Part | What it does | Where |
|---|---|---|
| Reader thread | Decodes frames. Sets the process-global `jaic::interrupt` for every message except `workspace/diagnostic`. | `main.rs`, `protocol/framing.rs` |
| Worker thread `jailsp` | 1 GiB stack. Owns a `JsonSession` (`Session` is `Rc`/`RefCell`). Its loop runs, in order: indexing progress, debounced publication (wait = last check / 4, clamped to 15–200 ms), completion warm-up, background warm-up, then a 25 ms service poll. | `main.rs`, `session/` |
| Requests | Never compile (`defer_compiles(true)`). They answer from the newest finished compile, shifted to the current text. | `semantic/cache.rs` |
| Check | Type-checks the whole program from scratch: `LintCompile::start` → `compile` → `check_rest` → release. Only parses are reused (`ParseReuse`, 4 compiles). | `semantic/compile.rs` |
| Cache | Keeps 2 compiles (`CACHED`), 2 more under 1 MiB, 3 alternates. Found by `Rc` address, then content hash, then `unaffected`. | `semantic/cache.rs` |
| Several entries | Checked one after another (branch `worktree-agent-a30bd01747413685e`, not yet in `main`). | |
| Membership | `programs.rs::program()` takes the first entry whose closure has the file. Closures use recorded `BuildFacts` when they exist, else the static `add_build_file` literals in `FileScan::builds`. | `auto_import/` |
| Protocol | About 3,200 hand-written lines: wire shapes ~790, dispatch and cancel ~1,500, framing ~290. | `protocol/` |
| wasm | `jai-wasm` drives `JsonSession` directly with `handle_json`. No threads. | `crates/jai-wasm` |

### Measured costs

All runs on the 10-core, 16 GiB dev machine while other agents were building. Absolute times are inflated by the load (shown), so the shares are the reliable part. Medians over 7 warm checks. A one-character edit is made in the entry file.

**Where a warm check goes.** These numbers come from a temporary exclusive-time profiler, now removed.

| Program | Load | Warm check, median [min–max] | Lints | Stdlib sema (bodies + decls + loads) | User sema | Rest of compile | Stdlib / user bodies lowered |
|---|---|---|---|---|---|---|---|
| `examples/editor` | ~40 | 4144 ms [2831–5467] | 48.8% | 34% | 5% | 3.6% | 1953 / 135 |
| `examples/game3d` | ~40 | 1423 ms [1215–2594] | 28.6% | 35% | 8% | 15% | 845 / 133 |
| `examples/seek` | ~40 | 1550 ms [1247–2146] | 36% | 40% | 7% | 12% | 665 / 83 |
| `examples/traceview` | ~40 | 1337 ms [715–2460] | 33% | 37% | 4% | 16% | 1004 / 51 |
| `examples/editor`, quieter run | ~33 | 670 ms | 41–53% | 30–38% | 4.5% | 6–14% | 1953 / 135 |

Other results from the same runs:
- Release of build state and lowered code takes about 3%.
- Diagnostics assembly takes 2–4%.
- Parsing is all hits on a warm check.
- On a cold check, parse misses take 75–235 ms, about 8% of the check.
- A warm `editor` check makes 4.6 M allocations. A cold one makes 5.3 M.

The user's own code is about 5% of a check. Lints and the stdlib account for 65–85%.

**Memory per kept compile.** This is the live heap freed when the cache drops one compile. The base is what stays: parse reuse, indexes and the stdlib scan.

| Program | Per kept compile | Base after drop |
|---|---|---|
| editor | 77 MiB | 84 MiB |
| traceview | 45 MiB | 67 MiB |
| game3d | 48 MiB | 60 MiB |
| seek | 34 MiB | 38 MiB |

**Requests and typing against the real `jailsp`, `examples/editor/main.jai`, load 33.** Measured with a scratch driver over `tools/bench/lsp_client.py`.

| Measure | Idle | While a check runs |
|---|---|---|
| hover, median [range] | 37 ms [16–59] | 32 ms [10–112] |
| completion | 33 ms [28–102] | 29 ms [23–169] |
| semanticTokens | 5.2 ms [3.4–43] | 4.8 ms [3.2–23] |
| edit → publish, undisturbed | 1316 ms | |
| edit → publish, with requests sent mid-check | | 3169 ms [1412–7084] |

| Typing, 21 edits | Publishes | Last key → publish |
|---|---|---|
| 80 ms apart | 1 | 1377 ms |
| 150 ms apart | 1 | 2833 ms |
| 300 ms apart | 1 | 2035 ms |
| Whole session | 23 checks finished (45.9 s) | 27 checks stopped (9.2 s thrown away, 17% of check time) |

What the numbers say:
- **Requests are fast because they never compile.** Even mid-check they answer in milliseconds.
- **The cost moves to diagnostics.** Every request interrupts the running check, which restarts later. Requests mid-check push publish latency from 1.3 s to 3.2 s (2.4×).
- **Typing at any realistic speed gets one publish, at the end.** Stopped checks waste about a sixth of all check time.

**Serialization**, native release build, load 16–26:

| Payload | Today's shape | Typed serde structs (lsp-types) |
|---|---|---|
| 200k semantic tokens | 11.1 ms (raw `Vec<u32>`) | 8.9 ms (same bytes) |
| 5000 completion items | 29.1 ms (`json!` tree per item) | 5.7 ms (same bytes) |

The completion gain comes from typed serialization, not from lsp-types in particular.

## 2. Target architecture

### Threads and ownership

```
          ┌──────────── main thread (owns Session) ─────────────┐
frames ─▶ │ documents + revisions, membership facts, job queue, │ ─▶ replies, publishes
(reader)  │ syntax answers, queries on finished compiles         │
          └───────▲─────────────────────────────┬────────────────┘
                  │ results (Send)              │ jobs (Send inputs + token)
          ┌───────┴─────────────────────────────▼───────┐
          │ check pool: N threads, 1 GiB stacks          │
          │ build-facts / types / lints jobs, per program │
          └───────────────────────────────────────────────┘
```

**Main thread.** It owns everything that is `Rc` today:
- documents and their revisions;
- the auto-import index;
- membership facts;
- the kept compiles;
- the job queue.

It answers two kinds of request itself:
- **Syntax-only requests**: folding, selection ranges, formatting, document symbols, and syntax semantic tokens.
- **Every query on a finished compile.** Measured query costs are 0.1–6 ms in-process (hover 0.1–1.3 ms, semantic tokens 2–6 ms), so the main thread never waits on sema.

**Reader thread.** It stays as it is (native only). It only frames and forwards. It no longer touches any interrupt.

**Check pool.**
- Size: `N = min(cores − 1, memory budget / 80 MiB)`. A program costs 34–77 MiB per compile plus a 38–84 MiB base, so a 1 GiB default budget gives 4–6 workers on this machine.
- Each job carries a program key, a revision, a kind and a cancel token.
- A job builds its own `Compiler` on the worker thread from `Send` inputs:
  - texts as `Arc<[u8]>`;
  - a file-system snapshot;
  - options.

**Ownership of a compile.** It moves exactly once:
1. The worker builds the compile.
2. The worker publishes type diagnostics as plain data.
3. The worker runs the program's lints.
4. The worker sends the compile to the main thread, which keeps it and answers queries from it.

So `Compiler` must be `Send`, not `Sync`. That means replacing about 15 `Rc`/`RefCell` sites with owned or `Arc` values:
- the file-system handle;
- the host;
- the shared workspaces;
- `ParseReuse`;
- the name caches' `StdlibView`;
- the `Rc` in `source_state` and the span index;
- the schema and plan;
- `LoaderGraph`;
- on the LSP side, `OverlayFs` reads and `SharedHost`.

`sema/sync_world.rs` already proves the data tables are `Send + Sync`. The thread-local `ARGUMENT_NAMES` (`sema/calls/slots.rs:13`) must move into `Compiler`, because it leaks state between compiles on one thread.

**Why not keep each program on its own thread** (the original proposal)? Because then queries must run on that thread. While it is checking, a query either waits for the check (1.3–4 s here) or interrupts it, which is today's 2.4× publish penalty. Moving the finished compile to the main thread removes both. The worker keeps nothing between jobs except the shared parse cache and the stdlib snapshot (below), both of which are `Send + Sync` and read-only.

### Message flow

1. **`didChange`.** Main applies the edit and bumps the document revision. It cancels the tokens of queued and running jobs for every program that owns the file and has an older revision. It then enqueues a `types` job for each owner, debounced as today.
2. **A request.** Syntax requests are answered at once. Semantic requests are answered from the owner's newest finished compile, with a `Shift` for edits since. A request never cancels a check.
   - Completion is the exception: it may want a compile of the current text. It enqueues a high-priority `types` job and answers from the older compile if that job has not finished within the client's wait.
3. **A `types` job finishes.** The worker sends `Diagnostics(program, rev)`. Main publishes it if `rev` is still current for that file. The worker then runs `lints` under the same token, sends `Lints(program, rev)`, and finally sends `Compile(program, rev, Box<Compiler>)`.
4. **`$/cancelRequest`.** Only requests can be cancelled. Checks are cancelled only by a newer revision.

### Job queue

There is one queue on the main thread. Jobs are ordered by `(priority, revision age, program LRU)`:

| Priority | Job | Why |
|---|---|---|
| 0 | `build-facts` for an entry with no facts | Membership is pending until it runs (§ Membership) |
| 1 | `types` for a program that owns a visible document | Diagnostics the user is looking at |
| 2 | completion compile | Explicitly asked for |
| 3 | `lints` (runs on the same worker after `types`) | 29–49% of a check, can publish later |
| 4 | `types` for other open programs, workspace diagnostics | Background |
| 5 | warm-up, indexing | Idle work |

When a newer revision for the same program and kind is enqueued, it replaces the queued one. Rules for running jobs:
- A running job of priority 0–3 is not preempted. Its worker is simply not reused until it ends.
- A running background job (priority 4–5) is cancelled when a job of priority 0–2 is queued and no worker is free. It goes back on the queue. Without this, background checks hold visible diagnostics for a whole check, and with `N = 1` (wasm, small budgets) they always would.

So checks are cancelled only by a newer revision or to free a worker from background work; requests never cancel them.

### Cancellation

- `jaic::interrupt` is replaced by a `CancelToken(Arc<AtomicBool>)` passed in the compile options.
- It is polled at the same five points as today:
  - `lower_body`;
  - `resolve_entity`;
  - `load_file_inner`;
  - the two module points.
- A token belongs to one job, so cancelling one program never stops another.
- The interpreter doesn't poll the interrupt today; compile-time code stops only at the check time limit. Stage 4 adds a token poll where the interpreter checks that limit.

Gain: with requests no longer cancelling checks, publish latency under requests should match the undisturbed latency. That is 1316 ms instead of 3169 ms (2.4×) on `editor`. Stale-revision cancellation still stops checks during typing. That waste is unavoidable, and it shrinks with the stdlib reuse below, because a restarted check redoes less.

### Lints as their own phase

`Session::diagnostics` today waits for lints before publishing anything. In the target, type diagnostics publish first and lints follow in a second publish of the same version. Gain: check-to-first-publish falls by the lint share, 29–49% measured.

Lints still need the compile. They run on the worker that holds it, before the hand-off, so no extra copy is needed.

Until the new lints arrive, the first publish of a version carries the previous version's lint diagnostics, shifted through the edits since and dropped where an edit touched them. Without that, every edit would clear the lints and redraw them a second later.

### Stdlib reuse

Stdlib sema is 34–40% of a warm check. It is redone every time because each check starts a fresh `Compiler`. The state after loading the stdlib is shared by every check with the same:
- options;
- target;
- import set;
- module parameters.

That holds only until user code mutates it. The known mutators are:
- `#add_context`;
- `#poke_name`;
- `#placeholder`;
- module parameters;
- target options set by a build;
- `#run` with side effects;
- polymorph instances added to generic `ProcInfo.instances`.

Target design: a **frozen stdlib snapshot** per `(options, target, import set)`.
1. Build a `Compiler` that has loaded the imports, resolved every non-generic stdlib declaration and lowered every non-generic stdlib body reachable from them. Generic bodies stay lazy.
2. Freeze it as a read-only `Arc`.
3. A check clones it. Better, it layers user state over it, as the speculative-body arena cut-back and undo journal already do.

A check falls back to a full build when it sees a mutator touch snapshot state. Each fallback is counted, so we can see how often it happens.

Snapshot blockers, ranked:
1. Interpreter state bound to addresses, which needs a reset or relocation.
2. User mutation of stdlib state, handled by the fallback above.
3. `Clone` on all of sema, about 60k lines, mechanical.
4. Copy-on-write of lazily resolved state.
5. Trait objects.
6. Thread-locals.
7. Id order, which is fine for the IDE.

Because lowering is lazy, a snapshot taken right after loading would save only the load share (~3%). The gain needs the eager pass in step 1, which costs once per key.

Expected gain: up to 34–40% of a warm check, minus the clone cost. **The clone cost is not measured yet.** That is the first task of the stage, and the stage stops if a clone costs more than half of what it saves.

### Caches and bounds

| Cache | Key | Bound |
|---|---|---|
| Kept compiles | program, revision | Newest per open program, plus 1 alternate for the active document. Evicted LRU by program under the memory budget (default 1 GiB, setting `jai.memoryBudget`). |
| Stdlib snapshots | options, target, import set | 2, LRU. Each is roughly one compile's size. |
| Parses | path, content hash | Today's `ParseReuse` window, made `Arc` and shared by workers |
| Build facts | entry, content stamps | In memory for all entries; persisted (§ Membership) |
| Scans | path, content hash | As today |

The budget replaces `CACHED`, `SMALL_KEPT` and `SMALL_COST`. Those count compiles, but a compile here ranges from 34 to 77 MiB.

### wasm

`jai-wasm` gets an inline executor with the same queue and the same job code:
- after each incoming message, it runs one job to completion;
- `N = 1`;
- no threads.

Tokens still work, because an incoming message is handled between jobs. Two things change:
- an in-flight wasm job can't be cancelled mid-way. That is today's behaviour too, since the browser can't set an interrupt while wasm runs.
- lints still publish separately.

Native tests and the fuzzer get the inline executor as well, so runs are deterministic.

### Multiple entries and shared files

- **Checking.** Each entry is its own program and its own job, so programs check in parallel up to `N`. Serial checking (the branch above) becomes parallel at no extra design cost.
- **A file owned by several programs.** Main asks every owner whose compile exists and merges:

| Feature | Merge |
|---|---|
| Diagnostics | Union. A diagnostic found in only some owners says which, e.g. `(only in build for windows)`. |
| Hover | Deduplicated. When types differ, one section per program, headed by its entry. |
| Definition, references, rename | Union of locations. Rename refuses if owners disagree on the target. |
| Completion | Union by label. |
| Semantic tokens, inlay hints | From the owner listed first in the build facts (stable). These can't be merged. |

This replaces the first-match `break` in `programs.rs::program()`.

### Membership: facts only

**Rule.** Program membership comes only from actually running an entry's build metaprogram. That covers:
- which program a file is in;
- which files a build adds;
- the not-in-program note.

Static scans never decide it, even provisionally.

**Facts.** `BuildFacts` / `BuiltProgram` (`semantic.rs`) grow each workspace's full effective build options, not only files, strings and import dirs:
- target OS and CPU;
- output type;
- the import path;
- module parameters;
- every other `set_build_options` field.

Hover, navigation and the target used for checking then follow what the build really set. That retires `only_os` guesses.

**States per entry:**
- **Pending** (no facts yet). Its `build-facts` job runs at priority 0. Files it might own get syntax features only:
  - no membership-dependent answers;
  - no not-in-program note.
- **Known**. Facts from the last good run.
- **Failed / timed out**. Shows one note naming the entry and the reason, and keeps the last good facts. If there never were any, the entry stays pending; nothing falls back to a scan.

**Untrusted workspace.** No metaprogram runs, so no membership claims are made. Each file is checked on its own with compile-time code off, as today's untrusted compile does. One note says build scripts were not run because the workspace is not trusted. The per-file not-in-program note is not shown.

**Cold start.** The first build run costs about a cold check:
- 0.9 s quiet;
- 2.9 s for `editor` at load 33.

Two things keep that small:
1. **Entry builds run first**, during indexing (priority 0), before warm-up or workspace diagnostics.
2. **Facts persist.** At the end of a session, each entry's `BuildFacts` with its `sources` content stamps is written to the cache dir, keyed by entry path and options hash. At start the stamps are rehashed. That reads only the listed files, which takes milliseconds next to a check. Matching facts are used at once and the build reruns in the background to confirm. A mismatch leaves the entry pending.

Facts already carry `sources: BTreeMap<PathBuf, u64>` stamps, so no new invalidation logic is needed.

**`FileScan` after the change.** Static fields that stood in for facts are removed. The scanner stays for syntax facts.

| Field / use | Fate | Why |
|---|---|---|
| `builds` (`index_fs.rs::edges`, `items.rs::is_build_metaprogram`, test in `exports.rs`) | **Remove** | A guess at what a metaprogram adds. Facts or pending replace it. Untrusted mode no longer needs to know which file is a metaprogram. |
| `import_paths` (`modules.rs:110,167`) | **Remove** | Facts' import path replaces it. Before facts, only the default import path applies. |
| `only_os` (`module_index.rs:111,199`, `items.rs:325`) | **Remove from target choice.** Keep only as a filter on auto-import suggestions of stdlib modules. | Target comes from facts. A module's own `#assert OS` is a syntax fact about that module, not about membership. |
| `declares_main` (`items.rs:278`) | **Keep** | Auto-import uses it to avoid suggesting `#load` of another program's entry. That is a fact about the file's text, not about membership. |
| `loads`, `imports` | **Keep** | Exact lexical `#load`/`#import` edges. Only used inside closures that facts already rooted. |
| `symbols`, `decls`, `local_decls` | **Keep** | The auto-import and workspace-symbol index. |
| `first_code` | **Keep** | Where to insert an import. |

## 3. Expected gains and risks

| Change | Measured basis | Expected gain | Risk |
|---|---|---|---|
| Lints as a second publish | Lints are 29–49% of a check | First publish 29–49% sooner | Two publishes per version; clients must accept a replaced set (all do) |
| Workers + per-job tokens + queries on main | Requests mid-check: 3169 vs 1316 ms publish | Publish under requests ≈ undisturbed (−58%) | `Send` conversion touches ~15 sites; a missed thread-local leaks state (fuzzer + oracle catch it) |
| Parallel programs | Serial today; 10 cores | Multi-entry wall time ≈ slowest program, not the sum | Memory: 34–77 MiB per compile, bounded by budget |
| Stdlib snapshot | Stdlib sema 34–40% of a check | Up to 34–40% per check, also shrinking wasted typing work (17%) | Clone cost unknown; mutation detection must be complete or answers go wrong |
| Facts-only membership | First build run ≈ cold check, 0.9–2.9 s | Correct membership always; persisted facts make it ~instant on restart | Pending period on a first-ever open shows fewer features |
| Typed reply structs | Completion 29.1 → 5.7 ms for 5000 items | 5× faster completion serialization | None worth noting |

## 4. `lsp-server` and `lsp-types`

**Decision: adopt neither.** Convert the remaining `json!` hot paths to our own serde-derived structs.

**wasm test.** A scratch crate pinned to `lsp-server =0.10.0` and `lsp-types =0.97.0` was built for `wasm32-unknown-unknown` (release, 363 KB) and run in node.

| Part | Builds | Runs |
|---|---|---|
| `Message` (de)serialization, lsp-types structs | yes | yes |
| `Message::read` from a framed buffer | yes | yes |
| `Connection::memory()` + `try_recv` / `recv` with a queued message | yes | yes |
| `recv_timeout` on an empty channel (a waiting server loop) | yes | **traps `unreachable`** |
| `Connection::stdio()` | yes | **traps `unreachable`** |

So in wasm, only the message types work. The push-driven `JsonSession` does not need lsp-server's transport anyway.

| Criterion | lsp-server | lsp-types |
|---|---|---|
| Lines replaced | 150–250 of framing and dispatch | 400–650 of wire shapes |
| wasm | Message types only; the transport traps | Fine |
| Custom `jai/*` methods | Fine (raw `Request`) | Not covered; we keep our own types for `jai/expansion`, `jai/source`, `jai/setTrusted`, `jai/compileTimeLimit` |
| Message size limits | **None.** `read_msg_text` resizes to the claimed `Content-Length`, so one bad header aborts the process. Ours caps at 64 MiB native and 16 MiB in the browser, and recovers the id. | n/a |
| Ids | `i32` or string. We accept `i64` and floats. A null-id parse-error reply can't be expressed. | n/a |
| Cancellation | `ReqQueue` only cancels registered requests. It doesn't know our job tokens. | n/a |
| Uris | n/a | `fluent-uri` 0.1 validates but doesn't normalise, which conflicts with our normalised, file-only `DocumentUri` |
| Semantic token edits | n/a | `SemanticTokensEdit` can't express our u32-granular edits; `Vec<SemanticToken>` needs a second vector |
| Serialization cost | n/a | Same bytes. Completion 5× faster than our `json!` trees, but our own derived structs give the same gain. |
| Spec coverage | n/a | All 37 standard requests and 10 notifications we use exist in 0.97 without `proposed`. `CompletionList.itemDefaults` is missing (unused today). |
| Dependencies | crossbeam-channel, log | bitflags 1.x, fluent-uri 0.1, serde_repr |

If lsp-types is reconsidered, pin it exactly (`=0.97.0`) and use it only for request params, not for hot replies.

## 5. Rejected alternatives

- **Make sema `Send + Sync` with `Arc`/`Mutex` and query it from any thread.**
  - Lazy resolution mutates on read, so every query would lock.
  - A check makes 4.6 M allocations, which would pay for atomic refcounts.
  - Queries are 0.1–6 ms, so concurrent reads buy nothing that moving one finished compile doesn't.
- **Salsa-style incremental queries.**
  - Sema is a whole-program checker with `#run`, `#insert`, build metaprograms and stdlib mutation. Those side effects don't memoize.
  - It would be a rewrite of ~60k lines.
  - The measured hot spots (lints 29–49%, stdlib 34–40%) are addressed directly above.
- **A process per program.**
  - Every query result would cross IPC.
  - Each process repeats the 38–84 MiB base.
  - wasm can't do it.
  - Its one benefit, crash isolation, is mostly covered by catching a worker panic and dropping that job.
- **One thread per program, compile never leaves it** (the original proposal). Rejected for the query-wait reason in § Threads.

## 6. Migration

Stages are ordered by value per effort. A split is made only where it costs nothing: no shim and no temporary interface. Each stage ships with tests, the typing fuzzer (`docs/tools/lsp-fuzzing.md`) and the oracle (`docs/tools/lsp-oracle.md`) green.

| # | Stage | Lands together because | Size | Files | Test or benchmark |
|---|---|---|---|---|---|
| 1 | **Lints as a second publish** | Self-contained: splits `Session::diagnostics` into type and lint phases; works on today's single worker | S (~300 lines) | `session/documents.rs`, `semantic/queries.rs`, `protocol/notifications.rs` | Test: type diagnostics publish before lints for the same version. Bench: `typing_latency.py` first-publish time |
| 2 | **Facts-only membership** with full build options in `BuildFacts`, pending state, priority build runs, untrusted note; remove `FileScan::builds`, `import_paths`, `only_os` target use | One rule: any leftover scan use would mean two sources of truth | M (~1.2k lines) | `semantic.rs`, `auto_import/{programs,index_fs,items,module_index}.rs`, `modules.rs`, `exports.rs`, `session/{compile,indexing}.rs` | Tests: pending file gets no unreached note; failed build keeps last facts; untrusted shows one note; target follows `set_build_options` |
| 3 | **Persisted build facts** | Needs stage 2's facts as the only source; free split after it | S (~400 lines) | `session/indexing.rs`, new `session/facts_store.rs` | Test: restart with unchanged stamps gives membership before any build run; changed stamp gives pending |
| 4 | **Workers, per-job tokens, `Send` compile, queries on main, job queue, parallel entries, shared-file merge, inline executor for wasm and tests** | These only work together. Tokens need several workers to mean anything. Without the queue, workers have no order. Queries on main need the `Send` hand-off. The merge needs per-program results. Any subset would need a shim around today's single loop and global interrupt. | L (~4–5k lines) | `jaic/src/interrupt.rs` → token, ~15 `Rc` sites in `jaic` sema and `source_state`, `calls/slots.rs`, `main.rs`, `session/`, `semantic/{cache,compile,overlay}.rs`, `polymorphs.rs`, `jai-wasm/src/language_server.rs` | Bench: `request_latency.py` publish-under-requests ≈ undisturbed; typing bench stopped-check count. Tests: two programs check concurrently; shared-file hover merges; wasm executor matches native replies |
| 5 | **Memory budget replacing compile counts** | Needs per-program compiles from stage 4 | S (~300 lines) | `semantic/cache.rs`, `session/config.rs` | Test: budget evicts LRU program; bench: peak RSS on the 4 examples |
| 6 | **Stdlib snapshot** | Internal to the compile; free split | L (~3–6k lines, mostly mechanical `Clone`) | `jaic/src/sema/**`, interpreter reset, `semantic/compile.rs` | First: measure clone cost (go/no-go). Then bench: warm-check share of stdlib sema → ~0; oracle equality with snapshot on and off; fallback counter |
| 7 | **Typed reply structs** for completion and other `json!` paths | Free split | S (~400 lines) | `protocol/`, `completion_wire.rs` | Bench: completion serialization (5000 items) |

Stage 7 is independent and cheap. It can go anywhere.

Stage 6 is the largest gain per check but has the most unknowns, so it follows the structural work. It is the one stage that may stop at its first measurement.

## 7. Open questions

- **Clone cost of a stdlib snapshot.** Not measured. It decides stage 6.
- How often real projects mutate stdlib state (`#add_context`, `#poke_name`, module parameters). This sets the fallback rate. Measure it on the bench corpus inside `tools/bench`; the corpus wasn't present here.
- Whether compile-time `#run` inside stdlib modules runs before the snapshot point on any target.
- Worker count versus the 1 GiB stack each: virtual only, but confirm on Windows, where stacks commit differently.
- Where persisted facts live (workspace `.jacket/` vs the OS cache dir), and whether they are trusted-workspace only. They should be, because they encode a metaprogram's output.
- Rename across programs that disagree: refuse, or rename per program?

## Configuration

- `jai.memoryBudget` (new, stage 5): bytes for kept compiles and snapshots. Default 1 GiB.
- Worker count is derived from the budget and cores. There is no separate setting.
- Existing check limits (`CheckLimits`) apply per job unchanged.

## Dependencies

- `jaic` sema and interpreter: becomes `Send` in stage 4.
- `jailint`: lint phase.
- `tools/bench/typing_latency.py` and `request_latency.py`: the benchmarks above.
- No new crates.
