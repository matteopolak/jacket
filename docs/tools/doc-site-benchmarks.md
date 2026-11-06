# Documentation site measurements

This procedure compares the legacy documentation renderer and the Astro builder on the same library and publication settings. It separates build cost, browser responsiveness and actual network transfer; a successful build or layout check does not establish a performance improvement.

## Record the comparison inputs

Pin the baseline and candidate source commits, executable hashes and build profiles, compiler/runtime hashes, export manifest digest, dependency lock digest, Node/npm versions and publication configuration. Use production output for both sites. Keep the library, guides, categories, private-item policy, base path, examples and runtime identical. A legacy build using an interpreted compiler cannot establish a fair production comparison with a native candidate.

Before timing, validate the full output and record route, declaration, example and asset counts. Investigate missing or changed content independently. Include static HTML, local links/fragments, accessibility, layout, no-JavaScript navigation and runnable-example checks. Preserve failures instead of timing a reduced successful subset. The retired renderer publishes `assets/nav/api/<module>/<group>.html` as list rows, not standalone documents. `node tools/bench/doc_validate.mjs LEGACY_SITE /` validates these fragments inside their actual `ul.nav-list` context in a temporary shadow tree, then applies the same whole-tree standards and link checker. It records every fragment hash and verifies the served originals remain unchanged. All other legacy HTML is checked unchanged; malformed rows or missing links still fail.

Record the machine, OS, power mode, browser version, font availability, viewport, device scale, zoom, motion preference and theme. Use the same browser binary and font environment for both outputs. Record CPU and network emulation explicitly, including an unthrottled profile. System contention and interactive scrolling can add noise even when a benchmark owns the build slot.

## Build measurements

`tools/bench/doc_site.py` accepts a reviewed JSON plan with schema `jacket-doc-benchmark/1`. Commands are argument arrays, with an explicit working directory and cache-state label for every case. Validation alone runs no workload:

```sh
python3 tools/bench/doc_site.py --plan /tmp/doc-benchmark-plan.json
```

Execution requires an absent owned output directory. Prefer exclusive allocation and avoid overlapping owned builds; the host need not be completely idle. Record background load and contention, use matched case order, and report the spread rather than hiding slow samples. On a shared host, place the entire execution inside its heavy-job gate:

```sh
python3 tools/bench/doc_site.py --plan /tmp/doc-benchmark-plan.json \
  --out /tmp/owned-doc-benchmark --execute
```

The harness records wall time, waited-child CPU accounting and process-tree RSS sampled every 20 ms. The aggregate RSS peak is a sampled lower bound; the separately reported kernel largest-process peak is not the aggregate peak. Unavailable sampling and command failures fail the run and retain their receipts. Report all repetitions, median and range; use at least three repetitions and place paired baseline/candidate cases next to each other in the reviewed plan. The runner reverses case order on odd repetitions and records each sample's execution ordinal, while receipts remain grouped by case. Every case must prepare its own stated cache state; a warm case cannot depend on an earlier cold case running first. Do not silently discard slow or failed runs.

Measure complete cold and warm builds, then CSS-only, guide, API and configuration edits. State exactly which caches and outputs were removed for each cold case. An empty compiler cache, empty Astro cache and cold filesystem page cache are different states; do not call all three simply “cold.” Do not purge shared caches or delete another process's files. Reversible prepared edits must restore the same source after each case. Separate one-time dependency installation and compiler compilation from a site's rendering cost.

Keep the direct validated-shard loader with its four-shard LRU. The measured rendering stage was close to the legacy renderer; producer work and full HTML validation dominated the remaining build cost. This does not justify a custom-collection prototype. If measurements later identify shard materialization as a bottleneck, compare a real Astro collection on the same frozen export, components and validation, with separate caches. Count collection materialization, preserve equivalent content and retain compact results outside the delivery source; do not add a permanent unused architecture switch. Record raw and compressed output sizes, export sizes, file counts and cache sizes separately from elapsed time.

## Browser measurements

Use isolated owned browser profiles and production servers with matched response headers and compression. Record `Cache-Control`, validators, content encoding and service-worker state. A development server, unmatched file timestamps or browser heuristic caching can change the comparison. An uncompressed local server cannot establish hosted-CDN transfer savings.

Use the same controlled server for both frozen production trees:

```sh
PYTHONPATH=tools python3 tools/bench/doc_server.py --directory /tmp/legacy-site --port 8100
PYTHONPATH=tools python3 tools/bench/doc_server.py --directory /tmp/astro-site --port 8101
```

This loopback HTTP/1.1 server uses deterministic gzip level 6 for negotiated text/JSON/SVG/Wasm, SHA-256 ETags for the actual encoded representation, and `Vary: Accept-Encoding`. Page HTML, missing pages and the root mutable `versions.json` require revalidation; published assets have a one-hour freshness lifetime. The reserved `assets/` and `_astro/` namespaces remain assets regardless of extension, including the legacy sidebar HTML fragments. This gives legacy fragment data and candidate JSON data the same freshness policy. It resolves the same slashless routes as the preview server. A per-server cache retains at most 32 MiB of encoded bodies and 256 recently requested representations; warm the planned response set before browser sessions when reporting a server-warm comparison. Empty browser profiles remain browser-cold even with a warm server. Record gzip/identity negotiation and cache policy in every result. This policy excludes file timestamps from validators and does not establish HTTP/2, Brotli or a hosted CDN result. Change `tools/bench/doc_server.py` and its real HTTP regressions together when changing that policy.

For each representative landing page, large API module, declaration and guide, measure repeated matched sessions in these states:

| State | Required preparation |
| --- | --- |
| Cold first visit | Empty HTTP cache and storage; fresh page; no preceding speculative requests |
| Warm first new page | Same profile after a recorded initial visit; destination never visited or prefetched |
| Prefetched navigation | Record the hover/focus request before clicking; include it in the session total |
| Revisit and Back/Forward | Record visited routes, history actions and whether a document, SPA swap or BFCache restoration occurred |

Record first paint, FCP and LCP for initial document loads where the browser reports them. For navigation, define click-to-visible/useful-content with a destination-specific content predicate and separately verify the URL, anchor, focus and history result. Do not relabel initial-load LCP as an SPA navigation metric. Measure prefetched and unprefetched navigation separately. Measure keyboard search input to usable results after distinguishing index loading from repeated queries. Keep runtime/Wasm first use separate from a warm search or page navigation.

For a large API page at wide and narrow widths, record frame intervals, long tasks and layout shifts during a fixed scroll input sequence. Include DOM size. Report dropped-frame or long-task definitions, observation window, repetitions and spread. Preserve content and accessibility checks during these runs; hiding a sidebar or deferring required content changes the workload. Mark scroll noise and UI contention explicitly. Existing layout-test deadlines are correctness checks, not latency samples.

### Transfer ledger

Collect CDP request/response/loading-finished events together with Resource Timing. The recorder attaches to owned dedicated workers before they run, enables their network domain; worker imports and Wasm requests retain separate CDP session identities. The ledger classifies the retired sidebar HTML fetches as export data; actual document loads and page-HTML prefetches remain document bytes. Document Resource Timing remains a separate document-only view. The tested Chrome worker target rejects network emulation, so first-use runtime sessions require the explicit unthrottled CPU/network profile; the harness fails rather than silently measuring a differently throttled worker. Navigation-only profiles can still use page emulation. Pausing worker startup adds instrumentation overhead, so compare both variants with this same recorder and keep first-use runtime results separate. Each action also retains raw CDP `Performance.getMetrics` snapshots before and after it. For same-document search, the script/task/layout duration and heap deltas bound the combined decode, validation, filtering and rendering work; they do not isolate JSON parsing or establish process CPU time. Preserve request IDs, initiators, URLs, status, cache flags, response headers, encoded transfer sizes and decoded body sizes. Account for redirects, conditional responses and failed/cancelled requests explicitly. CDP encoded bytes and Resource Timing transfer sizes have different accounting boundaries; label each rather than adding them together. Cross-origin timing restrictions and cache-hit zero sizes need explicit treatment.

Classify documents, JavaScript, CSS, fonts, search, runtime and exported data. Include prefetch traffic even if no click follows. Compare sessions of the same length and actions. Inspect actual bodies or body hashes to distinguish new destination content from repeated global navigation, catalogue data or unchanged hashed assets. Look for eager search/runtime loading, duplicate requests, unconsumed prefetch and refetches of identical payloads. A smaller individual click does not imply a smaller session if its prefetch moved the bytes earlier.

## Recorded comparison

These measurements used the native legacy renderer at `0803e84`, shared library/native candidate inputs at `7b0194b`, checked Astro builder `6bcca53`, and browser harness `d948713`. Astro is pinned to 7.4.1, Node to 24.12.0, npm to 11.6.2 and LLVM to 23.1.2. The host was a 16 GiB arm64 Mac; both variants used the same production inputs and runtime. The [machine-readable samples](../benchmarks/astro-docs.json) retain exact source identities, three browser samples per case, phase boundaries and limitations.

Generation optimization is deferred. Each generation row below is one paired sample, not a median or a variability estimate. Cold means empty owned application caches/output, not a purged OS filesystem cache. The loaded-file case primes and verifies both complete baseline publications, changes an authored `#load`-included value from 17 to 19, and times the full changed CLI. Reset, baseline priming and semantic postchecks are outside that interval. It has one additional declaration and is not the same workload as the unmodified cold build.

| Generation workload | Legacy wall | Astro wall | Legacy sampled tree peak | Astro sampled tree peak |
| --- | ---: | ---: | ---: | ---: |
| Cold full build (one pair) | 46.275s | 152.543s | 881 MiB | 3021 MiB |
| Loaded-file edit (one pair) | 93.079s | 196.435s | 714 MiB | 2104 MiB |

Astro includes graph admission, rendering, full HTML/link/fragment checks and atomic publication; the legacy path uses its original native renderer and link checker. The candidate is slower in these samples. The sampled process-tree peak is a lower bound, distinct from the kernel largest-process peak stored in the JSON. Further generation/iteration repetitions were not run.

### Navigation before the live-main correction

The completed navigation comparison has 48 sessions: three alternating-order repetitions of four scenarios, two variants and two viewports (1280×900 and 375×812, scale 1). Chrome 156.0.8078.80 used the same system fonts, light theme, normal motion, unthrottled CPU/network and fresh browser profiles against warmed loopback gzip servers. All sessions passed, with no runtime exceptions, network failures or unknown encoded transfer sizes. These are historical measurements of the captured-main implementation; they do not establish the final corrected navigation latency.

Values are median (minimum–maximum), in milliseconds from trusted input. DOM arrival is separate from settled useful content, which also waits for fonts, hit testing, animations and two frames.

| Viewport / navigation | Legacy DOM | Astro DOM | Legacy settled | Astro settled |
| --- | ---: | ---: | ---: | ---: |
| desktop: first new guide, no hover dwell | 23.9 (23.3–32.8) | 77.2 (76.8–78.4) | 57.3 (54.6–75.6) | 306.9 (306.8–307.2) |
| desktop: first API leaf | 13.9 (13.4–16.6) | 50.4 (50.3–56.8) | 38.2 (33.0–43.1) | 274.6 (273.8–275.8) |
| desktop: hover-prefetched guide | 16.6 (13.8–18.5) | 60.8 (59.8–61.5) | 48.7 (32.4–49.5) | 282.0 (282.0–282.8) |
| narrow: first new guide, no hover dwell | 22.3 (20.2–26.0) | 80.8 (79.0–87.4) | 51.4 (45.1–55.5) | 321.7 (301.9–323.1) |
| narrow: first API leaf | 16.2 (15.8–18.2) | 54.2 (54.2–56.0) | 43.1 (42.2–45.9) | 277.9 (276.8–279.1) |
| narrow: hover-prefetched guide | 12.9 (11.1–13.0) | 62.2 (61.3–64.7) | 32.2 (31.6–33.3) | 282.1 (282.0–282.9) |

The legacy SPA did not invoke native view transitions. The measured Astro implementation added a 170ms main fade/4px slide plus native capture preparation/completion. Its captured main could not receive input during the fade, even with pointer events disabled on every transition pseudo-element. The correction skips the native visual transition through Astro’s public lifecycle hook and animates the live main with the same duration/easing/motion, after scroll restoration and outline initialization. A trusted incoming-page link now works during the visible fade. Settled animation time remains a conservative endpoint; it must not be relabeled as first paint or immediate input readiness.

### Transferred bytes

The following totals include the initial Basic visit, explicit prefetch preparation and Back/Forward actions. Values are medians; ranges appear where repetitions differ. Request/cache cells show request count and explicit CDP disk/prefetch/served-cache flags, not all conditional reuse.

| Session | Legacy requests / flags | Astro requests / flags | Legacy encoded bytes | Astro encoded bytes |
| --- | ---: | ---: | ---: | ---: |
| Desktop first guide + history | 10 / 1 | 14 (14–15) / 1 | 75504 | 90149 (90149–90371) |
| Narrow first guide + history | 10 / 1 | 15 (14–15) / 1 | 75504 | 90371 (90149–90371) |
| API leaf + history (both viewports) | 12 / 2 | 17 / 1 | 64852 | 85394 |
| Hover guide + history (both viewports) | 10 / 1 | 15 / 1 | 75504 | 90371 |
| Unconsumed hover (both viewports) | 9 / 1 | 12 / 1 | 75282 | 89705 |

On the initial Basic visit, legacy transferred 59,123 bytes in eight requests and Astro 74,716 bytes in eleven. External JavaScript was 10,430 versus 25,678 bytes; CSS was 7,782 versus 7,732. The JavaScript category includes application code and extracted bootstrap as well as framework code; legacy also embeds JavaScript in HTML. The destination Testing HTML itself transferred 16,159 versus 14,989 bytes. No unchanged hashed assets were refetched, and neither variant eagerly fetched search/runtime data in these scenarios. Unconsumed means no matching navigation within the recorded session, not proof the response can never be reused.

Encoded totals are observed CDP loading-finished bytes. A normalized status 200 can conceal HTTP 304, so a small response is not inferred to be a cache hit. Retrieved decoded-body totals in the JSON mean browser-exposed bodies: Chrome exposed empty bodies for some speculative fetches despite real encoded transfer. They are not complete decompressed network volume or decompression CPU. No HTTP/2, Brotli or CDN saving is established.

### Search reuse usability check

A separate bounded desktop check used the same search, navigation, slash-focus and query actions before and after the admission fix, with three alternating-order pairs under ordinary shared load. All twelve sessions passed without runtime or network errors. Before the fix, the candidate fetched the same digest-bearing index twice: the second request transferred zero bytes from disk cache but decoded and admitted the 14,995,697-byte payload again. A document now retains one successfully validated row set for its exact URL; failed or aborted pending requests remain page-owned and are not retained. The compiled check confirms one search fetch/admission per document and working results after navigation.

| Candidate post-navigation observation | Before reuse | After reuse |
| --- | ---: | ---: |
| Search requests across first and second query | 2 | 1 |
| Decoded data-event bytes across those requests | 29,991,394 | 14,995,697 |
| Slash focus through visible query results | 110.9–125.0ms | 30.2–42.5ms |
| Combined browser task-time proxy | 77.0–89.1ms | 8.1–9.1ms |

These are diagnostic ranges under shared load, not a controlled population speedup or isolated parsing CPU measurement. Slash focus starts admission before query input, so the combined window is reported. The first search transfer remains 1,096,341 bytes versus legacy 558,444 bytes; reuse removes repeated decode/admission, not that initial transfer. No wire-format redesign or eager loading was added.

FP/FCP/LCP, scrolling smoothness and runtime first-use performance are not established by these completed samples. Functional layout, accessibility, runtime and lifecycle checks are separate from performance measurements. No website speedup is claimed for these unmeasured workloads.

## Evidence and current limits

Keep commands, sanitized identities, raw per-run records, summaries and failures together. Publish relative evidence paths and hashes without private machine paths or credentials. Explain content differences, unsupported browser metrics, cache/header limitations and noise before drawing a conclusion. Do not invent budgets, wins or confidence intervals.

The build-plan runner implements wall/CPU/RSS recording and failure retention. The SPA correctness probe checks targeted request behavior, navigation and lifecycle cleanup. `tools/bench/doc_browser.mjs` implements a separate production-browser recorder: PerformanceObserver paint/LCP/CLS/long-task observations, scroll frame intervals, trusted input timestamps, useful-content predicates, CDP request/cache/encoded-byte events, Resource Timing and post-session decoded-body hashes. Its accounting and CLS helpers have focused regressions. The recorded comparison above is limited to its stated workloads and source identities; source implementation and mock tests alone do not establish a real-browser performance improvement.

Validate a reviewed browser plan without launching a browser:

```sh
node tools/bench/doc_browser.mjs --plan /tmp/doc-browser-plan.json
```

The plan uses schema `jacket-doc-browser/1`, exact `identities` and `equivalence`, an explicit Chrome executable, two named variants with production base URLs, at least three repetitions, and named profiles/scenarios. Each profile specifies width, height, device scale, CPU slowdown, network latency/throughput and a stated frame-time budget. Each scenario names its cache state and lists explicit preparation and measurement actions. Timed `goto`, `click`, `search`, `back` and `forward` actions require an expected path/hash and a visible useful-content selector; optional text, focus and scroll expectations check correctness. `hover`, `focus`, `key`, `wait` and fixed wheel `scroll` actions make prefetch, search setup and observation windows explicit. A fresh isolated profile starts each session, so a warm case must include its warm-up actions. Runtime clicks use `freshCompletion: true`: an observer armed before each trusted click must see its own output change to a pending state and then to the required successful text. The previous Run result cannot satisfy the next invocation; per-invocation tokens and observed mutations are retained in the receipt. This matched instrumentation adds overhead to both variants.

After source/plan review and the host’s resource admission, add `--execute --out /tmp/owned-doc-browser` with an absent output directory. Record whether the allocation was exclusive or shared; shared-load observations do not isolate contention from application cost. Variant order alternates between repetitions. Each action preserves a partial receipt; failures and browser exceptions fail the session. Body hashes and decoded sizes are requested after the actions, while network event byte totals include the entire session's preparation and prefetch traffic. Resource Timing, CDP data-event decoded sizes and retrieved decoded-body sizes remain separately labeled. Unknown or evicted bodies remain unknown. Repeated-body and refetched-hashed-asset receipts identify candidates for investigation; identical content alone does not prove an avoidable request.

Pointer preparation requests ordinary centered scrolling once, then waits for a connected, exposed target with stationary geometry across three frames before dispatching trusted CDP input. Native smooth scrolling and fold behavior remain enabled. Host action-to-ready intervals include this preparation; trusted input-to-ready intervals start at the actual input event.

Browser polling, observers and CDP messages add matched instrumentation overhead. Trusted input-to-visible browser time is available across a retained SPA document; a full document navigation uses the separately labeled host input-receipt measurement. Double animation-frame visibility is a presentation proxy, not proof of a specific compositor paint timestamp. A native view-transition snapshot can intercept input after the destination DOM arrives. The recorder therefore keeps DOM-ready observations separate and waits for native capture completion, loaded fonts and a live destination hit target before reporting content readiness. The SPA correctness gate independently checks native gesture readiness. Firefox headless BiDi captures were independently shown to omit the native transition compositor in the owned control fixture, so those images cannot establish GPU paint timing or pixel parity. Initial LCP observations are not reset or relabeled as SPA LCP. Scroll intervals use the declared budget rather than assuming every monitor runs at 60 Hz. Validate these limits with actual runs before interpreting the output.

## How to change it

Extend `tools/bench/doc_site.py` and its focused tests for build accounting or plan validation. Browser accounting belongs in a separate production-browser probe sharing the CDP transport with the correctness tools; keep correctness assertions and measurement samples distinct. Change the reviewed workload plan when adding a route or edit scenario, and preserve both variants' equivalent content. Dependencies are Python on POSIX (`wait4` and `ps`), the pinned builder tools, an owned browser and the production output being compared.

The browser harness records DOM arrival separately from content readiness. Content readiness requires the destination predicate, first contentful paint, loaded fonts, an uncovered live hit target, and no active native capture or finite ancestor animation, followed by two frames that still satisfy those conditions. This is a conservative interaction endpoint, not the first visible pixel of a fade. The native transition observer reads `Document.activeViewTransition` or `:active-view-transition`; it does not handle application promises or change routing. Transition state changes are sampled on animation frames, so their times have frame-level granularity. See the [CSS view transitions specification](https://drafts.csswg.org/css-view-transitions-2/) for that readonly state. Initial FP/FCP/LCP remain separate browser-reported measurements.
