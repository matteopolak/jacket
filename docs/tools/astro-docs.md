# Astro documentation builder

`website/docs` contains the static documentation website's presentation source. Jacket supplies compiler-derived, versioned documentation data; custom Astro components own the pages and browser interface. Human-authored guides remain under the repository's root `docs/` folder.

## Setup

Use the Node release in `website/docs/.node-version` and the npm release declared by `packageManager` in its `package.json`. Install the frozen dependency set explicitly:

```sh
cd website/docs
npm ci
npm run check
```

`PYTHONPATH=tools python3 tools/generators/generate_doc_site.py --check` (from the repository root)
checks the Rust DTO schema authority and the generated TypeScript/browser validators without
rewriting checked-in files. Run it without `--check` after changing the contract; it requires
the owned Rust toolchain and explicitly installed builder packages.

`npm run check` runs Astro and TypeScript checking. Astro's development server does not replace this check. `npm test` runs the checkout's builder tests, and `npm run check:generated` checks the schema-derived TypeScript declarations. Production code obtains export data through the validated loader; it must not assert that unchecked JSON has a generated type.

Dependency installation is a setup step. `jacket doc` must not run an implicit network installation or select a newer package release. Once setup is complete, rendering uses local dependencies. An offline first-time setup requires a separately prepared npm cache; the source package does not include that cache or `node_modules`.

## Export and presentation development

Create a semantic bundle with the owned compiler and Jacket before starting the website:

```sh
jacket doc --export-only --export /tmp/owned-doc-export
cd website/docs
JACKET_DOC_EXPORT=/tmp/owned-doc-export npm run dev
```

For source development, `just docs --setup --dev` explicitly checks dependency policy,
installs the frozen packages, builds debug Rust tools and a matching native `jaifmt` beside
Jacket, then exports once before starting Astro. Semantic refresh checks that formatter for
source changes and rebuilds it when stale; it does not trigger another Rust release build.
Use `just docs --dev` after setup. The wrapper watches semantic source/configuration/assets
and performs a full deterministic export on a real content change; it restarts the dev server
to clear route and loader state. A failed export keeps the last complete bundle, displays a
DEV-only stale banner, and waits for another actual source change. There is no timer retry.

A component or stylesheet edit is handled by Astro without rerunning compiler extraction.
Guide, API and configuration changes require a fresh complete export; do not treat an old
bundle as current source data. Keep the producer's schema and the selected builder compatible.
Export and rendering do not execute documentation examples; example validation is an explicit
separate action.

Type declarations use the existing native `jaifmt --declarations` documentation profile. The compiler exports every parsed conditional definition, its guards, private source digest, token identities and declared-name positions. Jacket verifies the source digest and formats all unique definition programs in one batch with `indent_width=4`, `max_width=88` and configuration discovery disabled. The manifest records the formatter binary digest, version and profile. Install the matching native formatter beside Jacket or on `PATH`; export never compiles a missing tool or starts a formatter per page.

The producer pairs non-comment code tokens before and after formatting. Exact token spelling and order must agree, including string literals; otherwise export fails. Compiler identities map onto the formatter's UTF-8 byte positions, so grouped fields, nested declarations and same-spelled names retain correct anchors and links. Astro displays that formatted text with responsive soft wrapping and adds semantic anchors; it does not reconstruct statements or apply a second Jai syntax formatter. Conditional definitions retain their own kind, documentation, scope, source and platform facts. Change layout in `Extensions/Jai_Format`, semantic identities in the compiler model, and display styling in `DeclarationCode.astro`. Formatter-owned newline boundaries also provide presentation wrappers for field-link highlighting. The compact module catalogue hides semantic comment/blank lines, while literal-contained blank lines remain visible; it keeps the formatter's code indentation, spacing and wrap choices. Detail pages retain the complete formatted text.

Optional Astro render sections use boolean conditions, such as `definitions.length > 0`. A numeric zero is rendered as text by Astro. Run `python3 tools/ci/check_doc_conditionals.py --work /tmp/owned-doc-conditionals` through the build gate to render empty and populated API sections and retain meaningful authored `0` descriptions and values.

Generated lint references declare example ownership in fence metadata: `jai jacket-example/1 {"kind":"example"}` starts an example, an immediately following `diagnostic jacket-result/1 {"kind":"diagnostic"}` supplies its result, and `jai jacket-example/1 {"kind":"fixed"}` starts a distinct corrected example. An optional `note` belongs to the example header. Settings precede the example. Ordinary authored “Example:” paragraphs and unmarked fences retain their original meaning.

The Markdown producer converts those markers into typed `example_group` nodes, strips metadata before normal code handling, and rejects unknown versions, unknown fields, orphan results and duplicate results. Astro renders one captioned container with attached source/result panes; existing syntax coloring and example controls remain inside their panes. Change the emission in `crates/jailint/examples/lint_reference.rs`, the typed contract/parser, and `Document.astro` together. Run `PYTHONPATH=tools python3 tools/ci/check_doc_examples.py --work /tmp/owned-doc-examples` through the build gate to check real self-assignment, duplicate-condition and use-after-free examples through the production parser and component. It requires the debug Rust tools and installed builder dependencies; the work directory must not exist.

Runnable example programs preserve the visible source's blank lines. The shared doctest generator keeps one diagnostic-origin entry for each generated line; its editable wrapper must reproduce that same program exactly. Extend the template equivalence regression in `crates/jacket/src/test/doctest.rs` when changing wrapper generation.

The production wrapper reads the validated export, renders into staging, checks the static
output and replaces only a Jacket-owned destination. It also supports a check without publishing:

```sh
npm run build -- --export /tmp/owned-doc-export --out /tmp/owned-doc-site
npm run build -- --export /tmp/owned-doc-export --out /tmp/owned-doc-site --check
```

`JACKET_DOC_EXPORT_STALE` is a development-only status message used by the watcher; production pages ignore it.
`JACKET_DOC_EXPORT` selects the input bundle when `--export` is omitted.
`JACKET_DOC_BASE` and `JACKET_DOC_OUT` configure Astro's page base and output directory;
the production wrapper sets them from the validated bundle and its staging directory.
When running Astro directly for development under a nested base, set `JACKET_DOC_BASE`
to the same normalized base as the export. Canonical publication origin and playground origin
remain separate producer settings. These environment values contain paths and public URL
settings, never credentials.

## Source and installed layout

The source package includes these roots:

| Path | Purpose |
| --- | --- |
| `astro.config.mjs`, `tsconfig.json`, `tsconfig.worker.json` | Static routing, build and type-checking configuration |
| `src/` | Astro components, page entry points, styles, browser modules and validated data loading |
| `src/generated/doc-site.d.ts` | Generated declarations; change the producer contract and regenerate instead of editing this file |
| `schema/doc-site.schema.json` | Generated wire schema used by runtime validation and TypeScript generation |
| `scripts/` | Build, validation and type-generation entry points |
| `public/` | Declared presentation assets, including local fonts and images |
| `package.json`, `package-lock.json`, `.node-version` | Builder identity and pinned setup inputs |

Native archives carry the same source closure in `docs-builder/` beside the tools. Homebrew keeps it in `libexec/docs-builder`, beside the real executable behind the command symlink. Node remains an external dependency. `JACKET_DOCS_BUILDER` selects an explicit builder source directory when the installed layout is unsuitable; the selected builder must support the producer's export schema.

The installed source package excludes `node_modules`, Astro caches, build output, semantic exports, tests and environment files. It includes the schema, generated declarations, build scripts and presentation assets needed to render a library. A prebuilt standard-library docs archive is a separate artifact: it can be served directly, but it cannot render an arbitrary library.

## Packaging changes

`tools/ci/package_docs_builder.py` owns the explicit source-package inventory. For example, from the repository root:

```sh
PYTHONPATH=tools python3 tools/ci/package_docs_builder.py --output /tmp/owned-toolchain/docs-builder
```

The destination must not already exist. The helper validates required files and package/lockfile identity, rejects symlinks, stages the complete directory and writes `builder-manifest.json` with relative paths, byte counts and SHA-256 hashes. It does not invoke Node, install dependencies or build a site.

When adding a new runtime source root, update the helper's inventory and packaging tests. Keep required inputs explicit so an archive cannot silently omit a schema or generated declaration. The installed scripts are explicitly `build.mjs`, its runtime HTML checker `check-site.mjs`, and `generate-types.mjs`; CI-only fixture scripts remain in the checkout. `tools/ci/package_release.sh` places this directory into native archives; `packaging/homebrew/jacket.rb.tmpl` must also install it because the formula uses an explicit allowlist.

## Delivery and validation

The final site is a standalone static tree. Serving it does not need Node, Astro or the compiler. Preview production output with the repository's server, which resolves slashless page URLs and returns the custom missing page with HTTP status 404:

```sh
python3 tools/dev/doc_serve.py --directory target/doc-site --port 8000
```

Test root and nested publication bases, deep missing URLs, fragments, assets, canonical metadata and archives against built output. Component checks, browser accessibility/layout checks and deployment acceptance prove different properties; none substitutes for the others. Keep compiler extraction and explicit example execution separate from rendering author-controlled documentation.

Changes to page appearance belong in Astro components and styles; changes to declaration identity, aliases, source provenance or resolved targets belong in the compiler/Jacket producer. Preserve the existing visual design and canonical routes when changing component structure. The builder uses Astro and pinned local JavaScript packages; its semantic inputs depend on Jacket, the compiler's documentation model, configured Markdown and declared assets.

The npm source lock is checked before installation by `tools/ci/check_docs_dependencies.py`.
It verifies every locked registry release, including nested and optional platform packages,
and the pinned npm release: exact registry identity, HTTPS tarball URL, SHA-512 integrity
matching registry metadata, and a publication age of at least fourteen days. Metadata caches
expire after three days; branch CI restores main's cache without overwriting it. This network
policy check belongs to explicit setup and CI, not to an already installed offline builder run.

The builder's package identity follows Jacket's workspace version. `tools/bump_version.py`
updates `website/docs/package.json`, the lockfile envelope and its root package version while
preserving dependency pins. Its `--check` mode reports disagreements before release packaging.
After a release-version change, rerun the generated schema/type/validator check and the installed
builder smoke: matching package identity does not replace schema compatibility or complete setup.

## CommonMark conformance

`tools/ci/fixtures/commonmark` retains all 652 unmodified CommonMark 0.31.2 executable examples, 55 separately identified table/strikethrough provider cases, their source/hash manifest and license notices. The parser dependency's own test helper enables more extensions than Jacket uses; the conformance adapter imports the production parser with Jacket's exact table/strikethrough profile instead.

Run `PYTHONPATH=tools python3 tools/ci/check_doc_commonmark.py --work /tmp/owned-commonmark` inside the shared build gate with the normal per-job Cargo variables. The work directory must be absent. It compiles the Rust fixture adapter, builds isolated static Astro routes importing the production Document component, and writes one explicit ledger row per original ID (707 total). Reviewed transformation rules and affected ID groups are pinned independently; unknown membership fails. The base parser output is compared with official expected HTML through independent parse5 trees. A test-only commonmark reference AST supplies the presentation oracle for escaped raw HTML, heading IDs, code presentation and deterministic fixture URL mapping. Code text in the JSON must remain byte exact; the presentation retains legacy trailing-whitespace trimming. Missing, duplicate, skipped or failed cases reject the result.

The fixture URL map preserves original URL bytes in synthetic HTTPS destinations so unrelated missing image files cannot prevent parser case accounting. Real guide resolution, assets and root/nested base URL policies require their separate installed CLI checks. Whole-site HTML validation, browser accessibility and visual parity are additional gates. Change the Rust parser/profile, Document component and reviewed oracle together when intentionally changing Markdown policy; never regenerate expected results from actual output.

For installed guide resolution, run `PYTHONPATH=tools python3 tools/ci/check_doc_resolver.py --jacket /owned/bin/jacket --work /tmp/owned-doc-resolver` inside the same build gate. It exports/builds an authored tiny library at `/` and `/nested/docs/`, inspects actual guide/API/fragment/external URLs and escaping, and requires unsafe and missing images to fail their policy checks. It never executes Markdown code examples.

Code-block semantic links use a heavier font weight as their non-color affordance, with underlines on hover and the normal keyboard focus outline. Prose links stay underlined; card titles and signatures keep their existing resting styles. Keep this distinction when changing selectors: color alone does not identify code links reliably in either theme. The full axe suite and `doc_declaration_bands.mjs --links` cover accessibility and the normal/hover/focus appearance together.

## Client navigation

The custom Shell includes Astro's supported ClientRouter. Only its same-document main transition is enabled. The legacy CSS cross-document opt-in is retired: full reloads and native document history use ordinary browser navigation, while ClientRouter keeps the main fade and stationary live chrome. A compiled-CSS browser regression rejects a second cross-document transition lifecycle. Pages remain complete static HTML with working ordinary links when JavaScript is absent. Client navigation uses Astro's history, hash and focus behavior; there is no second fetch/swap router. The configuration explicitly sets `prefetchAll: false` and the `hover` strategy. Only annotated documentation links prefetch after hover or keyboard focus, including links added by lazy sidebar/search rendering. Leaving the target or replacing the page cancels pending intent timers; Astro's prefetch API deduplicates requests and respects connection/data-saver policy.

Full-document reload/Back restoration starts with native `history.scrollRestoration = 'auto'` in the synchronous head bootstrap. The departing document also returns its history entry to native restoration on `pagehide`; no `beforeunload` listener or duplicate position store is added. Initial and client-history restoration use temporary instant-scroll CSS, avoiding Astro 7.4.1's smooth restoration calls. The incoming root carries that state through a traversal swap. `astro:page-load` restores manual ClientRouter ownership and ordinary smooth anchors; a persisted `pageshow` clears the same temporary state. This preserves static content before deferred scripts and does not read private router history fields. The complete layout suite checks reload/Back frames and exact restored positions.

Search dismissal invalidates pending result and error rendering independently of page lifetime. Escape, outside click, focus leaving the field and an empty query keep late responses closed; navigation aborts release old rows without moving the retained field's focus or text. Arrow keys explicitly reopen current results and select first/last before cycling; Enter uses only a visible current selection. The first lazy request remains shared, without retries.

The entry module owns a new AbortController on each `astro:page-load` and aborts it on `astro:before-swap`. Navigation, theme/version controls, search and examples attach to that lifetime; queued layout work, stale data requests and example workers must not survive a discarded page. The Shell's single guarded bootstrap persists across swaps: it saves the storage fallback, prepares the incoming theme, and measures header/outline layout before paint. Same-context navigation keeps the live sidebar attached, so it does not reconstruct categories or restore saved scroll over reader state. Keep these synchronous prepaint steps separate from deferred enhancement so delayed JavaScript cannot cause a theme or sidebar shift. Extend browser request/history/focus/worker-cleanup checks whenever changing this lifecycle.

The attached header's measured border-box height sets `--header-offset` before sidebar restoration and layout. Sidebars and page outlines use that shared offset, so scrolling or navigation cannot move a rail into the header's fractional height gap. The bootstrap carries the last measurement into the incoming document, then measures the attached header before paint. One header ResizeObserver and one window resize listener handle wrapping, font changes and viewport changes; this measurement adds no scroll listener. Keep the CSS metric fallback usable without JavaScript, and extend the bootstrap and browser geometry tests when changing header controls or spacing. Run `node tools/dev/doc_sidebar_geometry.mjs URL OWNED_OUTPUT api/Basic --header-resize` through the heavy-work gate against a fresh production build. This mode checks trusted magnifier/shortcut hit areas, Slash focus, actual header wrapping, viewport resize, both rails, and scroll positions 0/1/2/5; it temporarily constrains `.top-in` to 700px and restores it, without injecting measurement code. Cropped light/dark screenshots and SVG geometry accompany the result. Browser tools honor `CHROME_PATH`; Linux checks discover installed Chrome/Chromium on PATH, while macOS uses the standard application path.

The sidebar declares one stable outer rail with Astro's supported `transition:persist` ownership marker. ClientRouter uses its supported custom `astro:before-swap` function to keep the body, layout and rail physically attached: Astro's root/head/script utilities update document state, while only main content and the right outline are replaced. This avoids the default persist implementation's temporary reparenting through the root element. The router still owns fetching, history, accessibility announcements and script execution.

The admitted `Bundle` computes one SHA-256 identity from its typed manifest, once at construction. Each page carries that identity; source, module, configuration or runtime changes therefore invalidate cached navigation together. The preparation loader delegates ordinary document navigation when the shell protocol, base, version, export identity, fresh/stale development state, non-main module scripts, stylesheets or marked bootstrap code differ, or when main content is missing. This also prevents same-version frontend deployments from accumulating entry lifetimes. Version-menu anchors explicitly use `data-astro-reload`, so a known version change does not first fetch a page for SPA handling. A stale development banner appearing or clearing takes the same full-document path.

Global navigation keeps the same navigation, category and link nodes across global-page swaps; only the current-page marker changes. A bounded cache retains that global tree and the latest item-peer context. Context changes update the inner navigation deliberately, preserve loaded peer rows where their context matches, and refresh the page-local outline. Same-context swaps never reconstruct categories or restore a saved scroll position over the live reader state. The existing resize callback exposes the rail when crossing into desktop layout and restores the reader's mobile disclosure choice when returning, without changing category state. Storage restoration remains for initial document loads; storage failures retain the document-local fallback. The root, header, sidebar and footer explicitly use `transition:name="none"`; only main content participates in the native fade/slide.

Verify the generated transition names and live node identity on built output when changing this policy. Run `node tools/dev/doc_sidebar_geometry.mjs URL OWNED_OUTPUT api/Basic --live-rail --source-y=600 --theme=dark` through the build gate, then repeat with a fresh output directory and `--theme=light`. It scrolls the rail, opens a category with a trusted pointer gesture, clicks a top-half entry, and captures every rendered frame through native transition completion, Back/Forward and full window-scroll restoration. Assertions cover the same rail/navigation/category/focused link, live scroll, unaffected row/text geometry, main-only capture names, pointer hit targets and one route announcer. This mode injects no replacement swap or transition CSS.

The prepaint bootstrap retains the latest sidebar state and manual theme choice in memory when storage is blocked or a write fails. An unchanged stale stored value cannot erase a manual theme choice on a client navigation; a changed stored theme remains authoritative. Stored sidebar state initializes a new document, while a same-context persistent rail keeps its live scroll and groups even if another writer changes stored sidebar data. This fallback lasts within the current browser document and does not promise persistence through a full reload when storage is unavailable. It does not add another set of control or navigation listeners.

The browser request/lifecycle gate is `node tools/dev/doc_spa.mjs URL`. It uses the same owned Chrome/CDP transport as layout checks and retains a JSON receipt for targeted hover/focus requests, duplicate prefetch prevention, SPA/history/hash/focus/theme behavior, native modified/middle clicks, static no-JavaScript navigation and actual compiler-worker destruction across a swap. Run it on a fully integrated production build with runtime assets; a partial run or a sample without an eligible example is a failure receipt, not a pass. Every unexpected browser runtime exception remains visible. The entry module observes the native transition `ready` promise: a DOM `AbortError` from a skipped visual animation is expected only after `updateCallbackDone` succeeds. Failed DOM updates and other transition failures still reach `reportError`; the browser gate does not filter exceptions. Native gesture tests wait until the intended anchor passes an actual hit test, then send one gesture. DOM readiness during an active transition is not sufficient evidence of interactivity.

Build and browser performance comparisons follow [the documentation measurement procedure](doc-site-benchmarks.md). Correctness checks do not establish timing or transfer improvements.

Ordinary non-Jai fences support the existing `sh`, `bash`, `shell`, `zsh`, `console`,
`powershell`, `ps1`, `pwsh` and `toml` aliases through Astro's bundled Shiki component.
The generated spans use the existing theme variables; wrapper background, wrapping and
focus remain controlled by the site's CSS. Unknown fence names remain escaped plain text.
Jai coloring uses the shared clean-room spec; compiler facts supply reference identities.
Diagnostic fences use typed presentation roles. Highlighting never changes the raw source in the export; trailing
whitespace trimming is a display policy. Extend `src/lib/code-highlight.ts` and the real
production-component check `tools/ci/check_doc_highlighting.py` when adding an alias.

Module catalogue declarations use the same full-width member band as field tables. Keep
both `pre.sig` and `.semantic-declaration` covered by the `.row > .members` band rules
when changing declaration markup; item-page declaration cards keep their separate styling.
`tools/dev/doc_declaration_bands.mjs` compares actual built module snippets with a pinned
legacy site at wide/narrow widths in both themes. It checks outer styles and geometry while
retaining formatter-authored `pre.sig.decl.semantic-declaration` lines and their compiler-resolved spans; author source-line heights are not a parity invariant. Multiple definition variants form a contiguous stack of declaration bands.
The optional `--band-css` argument is a diagnostic source-rule ablation, not acceptance of
a rebuilt production artifact.

Editable Jai examples use the [shared declarative coloring spec](doc-coloring.md); compiler-resolved links remain separate and return when the original source is restored. The existing `doc_edit_coloring.mjs` production probe also compares rendered character positions against a plaintext mirror of the textarea's actual font, padding and mobile transform at 375px and 1280px in both themes, before edits, after trusted input and after reset. The mirror is removed synchronously; the terminal newline caret spacer is presentation only. Keep these alignment checks when changing code-link weights or editor metrics.

Persistent link underlines belong to prose paragraphs, lists, quotations and table text.
Code blocks, signatures, catalogue titles and control links retain their separate hover/focus
styles. Keep `main a` limited to shared underline spacing; applying a persistent underline
there also decorates linked code tokens and card names. The existing declaration-band browser
gate's `--links` mode records baseline/current computed decorations and clipped screenshots
in both themes. Its optional `--link-css` source-rule ablation is diagnostic evidence; verify
an uninjected production rebuild before accepting the CSS change.

The retained search field keeps its text and focus during navigation. Its page controller closes and clears the result list, including the active-descendant state, before the next page attaches listeners. Aborted requests cannot repopulate that list.

On desktop, the live sidebar keeps its full viewport height above the footer. Footer credits use the main content column, so the last navigation row and footer link remain independently reachable at maximum scroll. Mobile keeps the normal stacked footer.


The former Jai template tree, embedding build script, compiled-theme cache, HTML-rich view/search model and Rust HTML writers are removed. Their replacement owners are:

| Behavior | Maintained owner |
| --- | --- |
| Compiler facts, routes, resolved references and publication data | `crates/jacket/src/doc/export/` and its strict typed contract |
| HTML, metadata tags and navigation presentation | `website/docs/src/components/` |
| Sidebar, search, controls and example lifetimes | `website/docs/src/browser/` and Astro ClientRouter |
| Lexical coloring | Shared spec/interpreters described in [docs code coloring](doc-coloring.md) |
| Platform badges | `website/docs/src/lib/api.ts` platform policy consumed by `Platforms.astro` |
| Real compiler-to-export regressions | `crates/jacket/tests/doc_export.rs` |

The small Rust Markdown helper only detects indented-code warnings. Sitemap escaping remains XML-specific. CommonMark's reference HTML writer is test-only oracle code, not a production renderer. Preserve the producer, conformance, DOM, 24-case accessibility and 560-case layout gates when changing these owners; do not restore a second renderer or a legacy-template fallback.


### Cache identities across deployments

The admitted manifest supplies each copied asset and search shard's SHA-256 request identity. Search, the lazy engine module and Wasm use separate `?v=<sha256>` URLs; unchanged files keep their keys when another export input changes. Module navigation uses the digest of its exact compact serialized payload, shared by the writer and server component and memoized per admitted module. Module prose, source links and metadata do not invalidate that navigation request. The bundle identity still determines whether retained page chrome is compatible; it is not a global payload cache buster. The shell guard also compares the UI sprite URL because the retained header owns those server-rendered references. Bump the `data-doc-shell` protocol and its expected value in `src/browser/lifetime.ts` when changing retained server-only chrome markup incompatibly beyond the checked configuration, runtime modules, styles, bootstrap and sprite URLs.

Configured logos, publication-owned local social images and semantic asset targets acquire their own asset digest. Arbitrary external image URLs stay unchanged. Astro/Vite owns hashed filenames for the UI icon sprite and default favicon. Change the explicit asset binding in `src/lib/urls.ts`, the admitted inventory accessor in `loader.ts`, or the navigation projection in `navigation-path.ts` when extending these boundaries. No browser runtime or search data is fetched merely by loading a page.

The archive menu requests mutable `versions.json` only on its first pointer, focus or activation intent, with `cache: 'no-cache'` so a warm response is revalidated. Requests belong to the page abort signal; unavailable, invalid or single-version manifests leave a closed disabled menu for that page. Archive anchors perform native document navigation. Escape, outside click, focus leaving the button/menu, a second pointer or keyboard button activation, or page teardown cancels a pending opening intent; a late catalogue response cannot reopen the menu or steal focus. The same first-intent request remains shared without retries.

Digest query parameters distinguish ordinary HTTP cache entries; they do not provide historical storage or browser-side integrity verification. A tab holding old HTML can request an uncached old digest URL after deployment and receive the server's current bytes if the host ignores query parameters when locating files. The typed JSON boundary still validates shape, and the shell compatibility guard forces native navigation across changed admitted bundles, but neither promises a retained immutable export snapshot. Deployments needing historical tabs to retrieve historical bytes require versioned hosting outside this builder's current contract.


### Installed package closure smoke

`tools/ci/check_installed_docs.py` packages the actual source builder beside relocated `jacket`, `jaic` and native `jaifmt` binaries, then runs ordinary `jacket doc` from authored projects outside the checkout. It removes builder/export overrides and records every Node entry point and working directory with a preload. Root-base, nested-base and supported symlink launches must execute only the sibling `docs-builder/scripts/build.mjs`, render authored guide and formatted declaration content, and emit hashed UI assets. The normal builder performs the production HTML/link validation.

Run it after explicitly installing the pinned dependencies and building the matching tools:

```sh
PYTHONPATH=tools python3 tools/ci/check_installed_docs.py \
  --jacket target/debug/jacket --jaic target/debug/jaic \
  --formatter target/jaifmt --work /tmp/jacket-installed-docs
```

The work directory must be absent and outside the checkout. Set `JAIC_STDLIB` to the matching standard library and put the pinned Node on `PATH`. This smoke reuses that platform's already installed locked dependencies in independent staged directories; regular files are hardlinked or copied, only internal relative dependency symlinks are retained, and `.astro`/`.vite` caches are excluded. It performs no implicit `npm` or network installation. It proves relocation and source closure, not a fresh registry installation or the separate standard-library archive layout. The receipt records tool digests, source-file count and each CLI case. CI runs it in the Linux Jai conformance group; script changes select the existing code test job.

`node tools/dev/doc_examples.mjs http://127.0.0.1:8000/ /tmp/owned-lint-pages.json` checks the three actual lint reference examples at narrow and desktop widths in both themes. Passing a generated fixture directory instead retains the full 16-case error/warning operation matrix. Both modes verify attached source/result ownership; the real lint pages also verify minimal statement and branch deletions.


### Real browser deployment cache regression

`node tools/dev/doc_cache.mjs /owned/production-site /owned/cache-receipt.json` serves an existing complete standard-library production output through an owned loopback HTTP fixture. It never edits that output or any running preview. The fixture overlays separately labeled valid search/navigation payload generations and matching HTML digest attributes; it uses an existing module-summary search row and a peer outside the initial five server-rendered sidebar rows. These are synthetic cache-policy fixtures, not compiler results or performance measurements.

The real compiled client must show the first generation's summary-query result and hydrated peer, reuse both JSON responses on a warm revisit, fetch new digest URLs when relevant facts change, and retain the same keys/cache hits after an unrelated HTML/export identity change. CDP cache flags and the independent server request ledger support each assertion. A held search request must be canceled during a trusted supported SPA navigation; the retained document's fresh search results must survive the attempted late response.

A separate empty isolated browser cache requests an old navigation digest after the simulated deployment and receives the new valid bytes. This explicitly demonstrates the historical-tab limit above. JSON/assets use `max-age=3600`, HTML uses `no-store`, and there is no compression or service worker. The probe uses a fresh default Chrome cache without forced cache capacity. It currently covers the root base; the installed CLI smoke separately checks genuine nested-base builds. CI runs this alongside the unchanged accessibility/layout/browser checks. Change the overlay projection and assertions in this script when the navigation/search contract changes; retain the real client validator and server/CDP evidence.

For a manual hosted docs acceptance run, set `docs-pages` inputs `ref` to the exact candidate commit, `version` to the displayed package version, `browser_checks=true`, and `deploy=false`. This builds the real static artifact and runs the existing 24 accessibility and 560 layout cases plus the routing, cache, declaration, rail and production-example checks. Select other affected workflow groups separately; do not open a PR merely to trigger its first CI run.
