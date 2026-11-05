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

Execution requires an absent owned output directory and a quiet, exclusive machine allocation. On a shared host, place the entire execution inside its exclusive heavy-job gate:

```sh
python3 tools/bench/doc_site.py --plan /tmp/doc-benchmark-plan.json \
  --out /tmp/owned-doc-benchmark --execute
```

The harness records wall time, waited-child CPU accounting and process-tree RSS sampled every 20 ms. The aggregate RSS peak is a sampled lower bound; the separately reported kernel largest-process peak is not the aggregate peak. Unavailable sampling and command failures fail the run and retain their receipts. Report all repetitions, median and range; use at least three repetitions and place paired baseline/candidate cases next to each other in the reviewed plan. The runner reverses case order on odd repetitions and records each sample's execution ordinal, while receipts remain grouped by case. Every case must prepare its own stated cache state; a warm case cannot depend on an earlier cold case running first. Do not silently discard slow or failed runs.

Measure complete cold and warm builds, then CSS-only, guide, API and configuration edits. State exactly which caches and outputs were removed for each cold case. An empty compiler cache, empty Astro cache and cold filesystem page cache are different states; do not call all three simply “cold.” Do not purge shared caches or delete another process's files. Reversible prepared edits must restore the same source after each case. Separate one-time dependency installation and compiler compilation from a site's rendering cost.

For the loader decision, compare direct validated shards with a real Astro custom collection over the same frozen export and production components. Store the experiment outside the delivery source and give each variant its own Astro/Vite caches. Count collection materialization and validation in the appropriate stage. A collection that reads different content or omits validation is not an equivalent comparison. Record raw and compressed output sizes, export sizes, file counts and cache sizes separately from elapsed time. Retain the losing prototype's compact source and receipts; avoid a permanent unused architecture switch.

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

Use a table with actual measured values; leave unavailable cells explicitly pending:

| Profile and action | Variant | Requests / cache hits | Document encoded / decoded | JS + CSS | Fonts | Search + runtime | Export data | Total including prefetch |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Pending measurement | Legacy / Astro | pending | pending | pending | pending | pending | pending | pending |

## Evidence and current limits

Keep commands, sanitized identities, raw per-run records, summaries and failures together. Publish relative evidence paths and hashes without private machine paths or credentials. Explain content differences, unsupported browser metrics, cache/header limitations and noise before drawing a conclusion. Do not invent budgets, wins or confidence intervals.

The build-plan runner implements wall/CPU/RSS recording and failure retention. The SPA correctness probe checks targeted request behavior, navigation and lifecycle cleanup. `tools/bench/doc_browser.mjs` implements a separate production-browser recorder: PerformanceObserver paint/LCP/CLS/long-task observations, scroll frame intervals, trusted input timestamps, useful-content predicates, CDP request/cache/encoded-byte events, Resource Timing and post-session decoded-body hashes. Its accounting and CLS helpers have focused regressions. The recorder has not yet established a matched baseline/candidate result; source implementation and mock tests do not establish real-browser measurement validity or a performance improvement.

Validate a reviewed browser plan without launching a browser:

```sh
node tools/bench/doc_browser.mjs --plan /tmp/doc-browser-plan.json
```

The plan uses schema `jacket-doc-browser/1`, exact `identities` and `equivalence`, an explicit Chrome executable, two named variants with production base URLs, at least three repetitions, and named profiles/scenarios. Each profile specifies width, height, device scale, CPU slowdown, network latency/throughput and a stated frame-time budget. Each scenario names its cache state and lists explicit preparation and measurement actions. Timed `goto`, `click`, `search`, `back` and `forward` actions require an expected path/hash and a visible useful-content selector; optional text, focus and scroll expectations check correctness. `hover`, `focus`, `key`, `wait` and fixed wheel `scroll` actions make prefetch, search setup and observation windows explicit. A fresh isolated profile starts each session, so a warm case must include its warm-up actions. Runtime clicks use `freshCompletion: true`: an observer armed before each trusted click must see its own output change to a pending state and then to the required successful text. The previous Run result cannot satisfy the next invocation; per-invocation tokens and observed mutations are retained in the receipt. This matched instrumentation adds overhead to both variants.

Only after source/plan review and exclusive admission, add `--execute --out /tmp/owned-doc-browser` with an absent output directory. Variant order alternates between repetitions. Each action preserves a partial receipt; failures and browser exceptions fail the session. Body hashes and decoded sizes are requested after the actions, while network event byte totals include the entire session's preparation and prefetch traffic. Resource Timing, CDP data-event decoded sizes and retrieved decoded-body sizes remain separately labeled. Unknown or evicted bodies remain unknown. Repeated-body and refetched-hashed-asset receipts identify candidates for investigation; identical content alone does not prove an avoidable request.

Browser polling, observers and CDP messages add matched instrumentation overhead. Trusted input-to-visible browser time is available across a retained SPA document; a full document navigation uses the separately labeled host input-receipt measurement. Double animation-frame visibility is a presentation proxy, not proof of a specific compositor paint timestamp. A native view-transition snapshot can intercept input after the destination DOM arrives. The recorder therefore keeps DOM-ready observations separate and waits for native capture completion, loaded fonts and a live destination hit target before reporting content readiness. The SPA correctness gate independently checks native gesture readiness. Firefox headless BiDi captures were independently shown to omit the native transition compositor in the owned control fixture, so those images cannot establish GPU paint timing or pixel parity. Initial LCP observations are not reset or relabeled as SPA LCP. Scroll intervals use the declared budget rather than assuming every monitor runs at 60 Hz. Validate these limits with actual runs before interpreting the output.

## How to change it

Extend `tools/bench/doc_site.py` and its focused tests for build accounting or plan validation. Browser accounting belongs in a separate production-browser probe sharing the CDP transport with the correctness tools; keep correctness assertions and measurement samples distinct. Change the reviewed workload plan when adding a route or edit scenario, and preserve both variants' equivalent content. Dependencies are Python on POSIX (`wait4` and `ps`), the pinned builder tools, an owned browser and the production output being compared.

The browser harness records DOM arrival separately from content readiness. Content readiness requires the destination predicate, first contentful paint, loaded fonts, an uncovered live hit target, and no active native capture or finite ancestor animation, followed by two frames that still satisfy those conditions. This is a conservative interaction endpoint, not the first visible pixel of a fade. The native transition observer reads `Document.activeViewTransition` or `:active-view-transition`; it does not handle application promises or change routing. Transition state changes are sampled on animation frames, so their times have frame-level granularity. See the [CSS view transitions specification](https://drafts.csswg.org/css-view-transitions-2/) for that readonly state. Initial FP/FCP/LCP remain separate browser-reported measurements.
