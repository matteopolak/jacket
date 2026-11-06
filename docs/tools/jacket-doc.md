# jacket doc

`jacket doc` builds a static API reference for a library: module/item pages, configured Markdown guides, typed search data and optional runnable examples. The compiler supplies semantic facts; Jacket exports a versioned data bundle; custom Astro components produce the site. The standard library is configured in the root `jacket.toml`.

```sh
jacket doc                                  # the nearest [doc] library, else the stdlib
jacket doc path/to/my_lib --out site
jacket doc --module String --module Basic    # explicitly partial selection
jacket doc --export-only --export data       # extraction only; no Node by default
jacket doc --export data --out site          # retain the semantic bundle and build
jacket doc --check                           # production build/validation; no site replacement
jacket doc --validate-examples --runtime runtime --out site
```

The production build needs a matching `docs-builder` source package, Node in the supported Node 24 line and explicitly installed dependencies. In a checkout, use `website/docs/.node-version`, then run `npm ci` in `website/docs`; installed releases put `docs-builder` beside Jacket. `JACKET_DOCS_BUILDER` selects another compatible installation. Jacket neither downloads Node nor installs npm packages. See [Astro development and packaging](astro-docs.md) for the pinned setup, local HMR and offline-after-setup boundary.

Serve output with a static server that maps `/api/Basic/print` to `api/Basic/print.html`, such as `python3 tools/dev/doc_serve.py`. A plain `python3 -m http.server` does not provide those routes. Set `--base-url /reference/` for a nested host path; opening generated files directly from disk is unsupported.

## How it works

1. `jaic -doc_model DIR` emits strict `jai-doc-model/1` facts from the compiler parser/resolver: declaration identities, signatures, comments, visibility, platform limits, lexical declaration tokens and dependency hashes.
2. Jacket resolves links, routes, examples, publication metadata and configured guides into `jacket-doc-site/1`. The manifest inventories shards/assets by path, length and digest. Markdown is typed document data; highlighted text has UTF-8 byte spans and typed targets. No author HTML is executable markup.
3. The source builder admits the whole graph using generated runtime validators, then renders custom Astro pages. Metadata stays with each owning page shard; the root route table remains compact. A complete static tree is staged and independently checked before replacing Jacket-owned output.
4. Browser enhancements use Astro's supported ClientRouter and targeted hover/focus prefetch. Ordinary links and static content still work without JavaScript. A new page lifetime cancels old requests, listeners, timers and workers; the Shell restores theme/sidebar state before paint.

The [export contract](../compiler/doc-site-export.md) explains source ownership, schema generation and declaration formatting. The old Jai theme, ready-HTML view model and compiled renderer cache are retired. `--theme`/`[doc] theme` and `--emit-input` produce migration errors; change Astro source or use `--export` instead.

### Examples

Doc-comment fences use the same extraction and program templates as `jacket test`, including `ignore`, `no_run`, target restrictions, timeouts and module test headers. Run is disabled without compatible runtime assets or when the example cannot run in the browser; its label explains why. `--runtime` names a directory containing `engine.mjs` and `jai_wasm.wasm`.

Export and rendering do not execute examples by default. `--validate-examples` explicitly checks eligible examples and fails on an execution/compilation failure. With `--runtime`, validation requires Node and uses that exact browser engine for every library, including selected modules; it never falls back to another compiler. Without runtime assets it uses the owned interpreter's Wasm target. The default export-only path remains Node-free. `--no-test-examples` is a deprecated compatibility spelling for the default, not a way to bypass an explicitly requested check.

Validation honors each fence's wall-clock `timeout(n)` in seconds, defaulting to 60. Node retains a compiler per worker between successful examples; a timeout terminates that worker before the next example gets a fresh one. The browser's Run button separately caps an attempt at five minutes. Change `crates/jacket/src/doc/example_check.rs` and its embedded `run_examples.mjs` together; the focused worker tests use an owned fake engine without rebuilding Wasm.

Browser Run loads one example payload and starts a bounded module worker only after interaction. Stop, timeout and page replacement terminate it. Editing preserves the template around the user's current text; Run, Copy and the playground link use that text. Tab inserts four spaces, Escape then Tab leaves the editor, Enter preserves indentation, and Ctrl/Cmd+Enter runs. Reset restores the original example. No-JavaScript pages retain readable code.

### Output

| Path | Contents |
| --- | --- |
| `index.html`, `api/`, configured guide folders | Complete static pages at slashless URLs |
| `404.html` | Custom not-found page; legacy slash recovery probes only a same-origin slashless candidate |
| `_astro/` | Content-hashed styles, browser entry/router and worker chunks |
| `assets/` | Icons, favicon and configured copied assets |
| `search-index.json` | Versioned semantic search data, loaded on search interaction |
| `navigation/` | Compact typed peer facts for lazy bounded item sidebars |
| `examples/`, optional `runtime/` | Per-example data and explicitly supplied browser compiler assets |
| `versions.json` | Optional version-menu destinations |
| `sitemap.xml`, optional sitemap shards, `robots.txt` | Publication URLs when a canonical origin is configured |

Semantic export shards are separate from the published site; `--export` retains them for inspection or another consumer. Partial module selections are marked partial and do not establish full-site publication acceptance.

### Module identities and native bindings

Modules are the folders of the library with a `module.jai`, minus `internal/` and `tests/`. `prelude/` is a module page too; since every program imports it, the page says so instead of showing an `#import` line (`implicit` in the module JSON). When the path is itself a module, its id is the folder name and the modules inside it are `name/sub`. A library's own folder is passed to `jaic` as `-import_dir`, so its modules can import each other.

**Module parameters.** Each `#module_parameters` of the module's files becomes a "Module parameters" section before the items: name, type, default, and the doc comment above or after the parameter. Names in the type and the default link to what they resolve to, as procedure parameters do. Put one parameter per line so each has its own comment. They are not counted by `jaic -doc_gaps`. Declarations in a `#module_parameters` body (an enum a parameter's type needs) are ordinary items of the module: they get pages, links and search entries, and `#scope_file`/`#scope_module` inside the body hides them as at the top level.

**Native bindings.** A file exporting a top-level `#foreign` procedure mirrors a native API (files under `internal/` don't count, and `#intrinsic` or `#compiler` procedures never make a file a binding). Private native helpers under `#scope_file` or `#scope_module` do not mark the surrounding API as foreign. The model marks a binding file's declarations `"binding": true`; the module page lists them in the usual Procedures, Types and Constants sections after the module's own items, with a "Foreign" badge, and each gets an item page like any other. A procedure with a body in such a file is not a binding: it gets a page like any other. Search includes bindings with a "Foreign" badge and ranks them below the library's own API. Their `since` label is left off.

Documentation coverage uses a separate, broader rule: `jaic -doc_gaps` skips every file containing a bodiless top-level procedure, even a private helper or intrinsic.

## Settings: `[doc]` in `jacket.toml`

`jacket doc` with no path documents the `[doc]` library of the nearest `jacket.toml` (from the current folder), else the bundled standard library with no settings. With a path, the nearest `jacket.toml` above it applies only if its `[doc]` is for that folder; otherwise the site uses defaults. Paths and patterns are relative to the `jacket.toml`. Flags win over settings.

```toml
[doc]
path = "src/my_lib"                 # default: the jacket.toml's folder
name = "my_lib"                     # header and titles; default: the folder name
breadcrumb = "my_lib"               # first crumb on every page; default: the folder name
description = "API docs for my_lib."
home = "notes/home.md"              # the front page; default: the name and description
logo = "assets/logo.svg"            # .svg or .png; default: none
repo = "https://github.com/me/my_lib"   # Source links need it
playground = "https://example.com/play"  # default: none, and no playground links
since = "notes/since.tsv"           # "since X" labels; used only with --version
pages_title = "Guides"              # sidebar heading and folder of the [[doc.page]] pages
url = "https://docs.example.com"    # canonical origin: canonical links, og:url, sitemap.xml, robots.txt
og_image = "assets/social.png"      # a file (copied to assets/) or a URL; 1200x630 suits most sites
site_name = "my_lib"                # og:site_name; default: name
canonical_version = "1.4.0"         # other versions point their canonical here and get noindex

# Sidebar and index headings, in order. A module goes under the last category that matches
# its folder; with none, modules are listed with no heading.
[[doc.category]]
name = "Library"
modules = ["src/my_lib/**"]

[[doc.category]]
name = "Experimental"
modules = ["src/my_lib/Labs/**"]
badge = "labs"                      # beside the module and its items; default: no badge
tip = "Not stable yet"              # hover text; default: the name
color = "#8a5a12"                   # #hex or a colour name; default: muted
color_dark = "#e0b26a"              # default: color

# Folding groups inside each heading. With any, every module must be in one (last match wins).
[[doc.group]]
name = "Core"
modules = ["src/my_lib/Core", "src/my_lib/Util"]

# Extra pages, listed under `pages_title` in the sidebar. The header's link goes to the first.
[[doc.page]]
file = "notes/guide.md"
name = "Guide"

# A sidebar heading of its own, split from one file (see "Sections" below).
[[doc.section]]
file = "notes/recipes.md"
name = "Recipes"
```

When adding a public standard-library module, add its folder to the appropriate root `jacket.toml` documentation group. Full-site export rejects ungrouped modules so new APIs cannot silently disappear from navigation.

`[doc.format]` is not a site setting: it holds `jaifmt` overrides for the examples in doc comments ([jaifmt-rules](jaifmt-rules.md#format_doc_examples)).

A pattern matching a module's folder also matches the modules inside it. `jacket doc` fails on a pattern that matches no module, a module in no group (when there are groups), and a colour that is not `#hex` or a plain name. The header shows the version, repository button, playground link and pages link only when set.

The standard library's `[doc]` sits in the root `jacket.toml`: categories "Standard library" (`stdlib/**`) and "Extensions" (`stdlib/Extensions/**`, teal "extension" badge), the topic groups, and the "Tools", "Lint rules" and "Format rules" sections (generated into `docs/reference/` and `docs/tools/jaifmt-rules.md`). Add new stdlib modules to a group there.

**Sections.** A `[[doc.section]]` file is split at its headings: the text before the first `##` (plus each `##`'s own text and a list of its pages) is the overview at `<name as a slug>/`, each `##` is a folding group in the sidebar, and each `###` is a page at `<section>/<heading slug>/` titled with the heading. A `###` before any `##` goes in a group with no label. Inside a page, `####` and `#####` headings move up to `##` and `###`, so they reach its outline. The header link (`pages_title`) goes to the first `[[doc.page]]`, else to the overview of the section with that name. A link to `#heading` or `file.md#heading` of a split file goes to that heading's page. A `##` with no `###` under it (the Format rules' "Indentation") stays text on the overview, so `format-rules#indentation` is still an anchor there.

**Home.** `[doc] home` names a Markdown file rendered as the front page, with the module sidebar. Links in it (and in every extra page) to a module's folder or `module.jai` go to that module's page, so `[Basic](../../stdlib/Basic)` works both on GitHub and on the site. Without it the front page shows the name and description; the stdlib's is `docs/reference/home.md`. Configured pages and sections come first, followed by module categories, wherever the module sidebar is shown.

**Before and after.** Two fences in a row, ```` ```jai before ```` then ```` ```jai after ````, render as one block split by a line ("Before", "After"), stacked under 700px. Words after `after` are shown as the settings used (`indent_width=2` reads "After, with `indent_width = 2`"). Jai fences on these pages are highlighted and scroll sideways. Supported shell, PowerShell and TOML fences use the shared palette and wrap long lines; unknown fence names remain escaped plain text.

## Page metadata

Jacket computes titles, plain-text descriptions, canonical/Open Graph URLs, image policy, noindex and structured facts. Page titles use the configured `name` as their brand suffix; `breadcrumb` labels the root crumb and does not become part of the brand. `HeadMetadata.astro` serializes them with normal escaping; its static JSON-LD serializer escapes script delimiters. Item pages use `TechArticle`, and module/item breadcrumbs use `BreadcrumbList`. No `CollectionPage` policy is implied.

Descriptions are bounded to 155 characters. Home uses the site title; other titles include their qualified identity and site label. Empty descriptions omit description tags. Without `[doc] url`, canonical/OG URL, sitemap and robots output are absent. The 404 has no canonical or OG URL. Private pages are noindex and absent from sitemaps. Archived public pages can be noindex while remaining in the sitemap; their canonical points at the configured canonical version.

## Accessibility

The site targets WCAG 2.2 AA with semantic landmarks, one h1, a skip link, named controls, keyboard search/version menus, exposed fold state and scrollable wide content. Reduced-motion preferences apply to scrolling. Category colors are checked for contrast before export, and generated category styles accept only canonical color values.

`tools/dev/doc_a11y.mjs URL` retains the 24 page/theme/width checks: six page kinds, two themes and widths 1280/320. Its native-binding sample comes from a validated current-schema target rather than a guessed slug. `tools/dev/doc_layout.mjs URL` retains the layout/history matrix, including delayed deferred scripts, reload scroll restoration, sidebar/header swaps and Back/Forward. Browser runtime exceptions are never exempted by a transition-error text filter.

`tools/dev/doc_spa.mjs URL` separately checks targeted request behavior, supported SPA/history/hash navigation, theme/focus/control lifetime and compiler-worker destruction on a swap. These checks need Chrome; `CHROME_PATH` selects it. The CommonMark/extension ledger, independent whole-site HTML/link checks, axe, layout, SPA behavior and deployed acceptance are separate gates. A local successful build does not establish all of them.

The Shell's initial observer applies mobile/sidebar state once as nodes arrive, then disconnects. Incoming theme/open groups are prepared before Astro swaps documents. The custom swap settles attached fold geometry before returning to Astro, so history restores against the final content height; doing this in `astro:after-swap` would let scroll anchoring move the restored viewport. Sidebar/layout restoration finishes before paint. Keep fixed prepaint code in the Shell and abort-owned deferred behavior in browser modules; adding a late measurement cannot repair a first-paint shift.

Compiler documentation links retain the module's identity under the actual standard-library root, while page routes are relative to the selected documentation library. For example, documenting `stdlib/Extensions/Json` alone keeps its pages under `api/Json`; compiler links naming `Extensions/Json` resolve to those pages. The compiler and Jacket share the module-key helper, and the producer maps that key to the selected page module before emitting links. Ambiguous compiler keys fail link validation rather than selecting an unrelated page. When changing module discovery, keep this mapping separate from route naming and test both full-library and direct-module targets.

Executable imports are separate from both identities. A selected standard-library module still imports its actual stdlib-relative path (`Extensions/Json`); an external nested library retains its selected import path (`Acme/Widget`), even when its compiler link key is only `Widget`. The producer's `importPath` supplies the displayed import, example programs, editable Run/Open templates, playground links and native Wasm probe. Change the shared compiler stdlib-key helper and producer import context together; do not derive executable imports from page routes or external compiler link keys.

## Publishing

The stdlib site is published at https://jacket.matteopolak.com. The release workflow builds and packs the static site; Pages deployment is a separate selected operation. `.github/workflows/docs-pages.yml` installs the pinned Node dependencies, checks schema/types, explicitly validates examples and runs the selected browser checks. A manual workflow run deploys by default, so a branch validation must explicitly set `deploy=false`.

### Accepting a deployed site

A successful build or uploaded Pages artifact does not establish that the public site serves that build. Read the existing deployment before dispatching anything. For example:

```sh
gh api 'repos/matteopolak/jacket/deployments?environment=github-pages'
gh api repos/matteopolak/jacket/deployments/<id>/statuses
gh run view <run-id> -R matteopolak/jacket --json headSha,jobs,url
curl --max-time 20 -I https://jacket.matteopolak.com/
```

Keep a dated acceptance receipt with the deployment SHA/ref, successful `pages` job and completion time, environment URL, and the run’s `github-pages` artifact. Check the displayed version and Source links against the intended release. The artifact links the deployment to a run; only a byte comparison can establish CDN byte identity.

Check the public home, a configured guide, module and item at their slashless URLs. A deliberately nonexistent nested path must return HTTP 404 with the custom page; check its CSS, JavaScript, module links and working search in Chrome at that depth. Separately open a real item's legacy trailing-slash URL and verify the 404 script recovers to its slashless page. A 200 fallback is a failure.

Fetch `sitemap.xml` with a finite timeout and parse the XML. Every URL must use the configured HTTPS origin/base; exclude 404 and private pages. Archived public pages remain listed even when their canonical-version policy sets noindex. If it is an index, fetch every referenced shard sequentially. Sample page kinds. `robots.txt` must reference the canonical sitemap and permit public indexing. On home, guide and item pages, inspect title, description, canonical/OG URL and image metadata; the 404 must omit canonical and OG URL. Fetch the image and verify its content type and dimensions.

Render the public page and card image, retaining a screenshot or concrete visual observations. Report third-party card-cache refresh separately; HTML metadata and a fetched image do not establish it. Use bounded requests, retain failures, and distinguish access problems from incorrect contents. Any later recheck should state what changed. Never deploy a development branch over production to obtain acceptance.

To validate a repaired branch without deployment:

```sh
gh workflow run docs-pages.yml -R matteopolak/jacket --ref my-docs-branch \
  -f ref=my-docs-branch -f deploy=false -f browser_checks=true
```

The implicit `#Context` reference is a configured Markdown guide at `guides/context`, linked from the home page. It documents the compiler-provided type without inventing a library declaration or a fixed layout. Directive identifiers such as `#Context` are not declaration names: the doc model does not resolve them to a same-spelled user type. Ordinary `Context` and qualified names retain their declaration identity. Guide publication and explicit prose links do not by themselves promise automatic links from signatures or search.

## How to change it

- Documentation conventions and resolved compiler identities belong in the [compiler doc model](../compiler/doc-comments.md).
- Permanent data changes start in `export/contract.rs`; regenerate the schema, declarations and all runtime validators, then update producer mapping, graph admission and consumers together.
- Markup/components live in `website/docs/src/components`; declaration layout is `src/lib/declaration.ts`, styles are `src/styles/site.css`, and per-page interaction is `src/browser`.
- Change example templates in the shared doctest code and keep the browser fill tests aligned. Change Markdown policy with the complete pinned conformance ledger, not a handpicked screenshot.
- `tools/dev/doc_diff.py OLD NEW` compares generated trees. Keep benchmark inputs/runtime/base fixed and report measured methodology/results rather than assuming a renderer change is faster.

## Configuration

| Flag | Default | Behavior |
| --- | --- | --- |
| `PATH` | `[doc] path`, else stdlib | library to document |
| `--out` | `target/doc-site` | replace only Jacket-owned site output after successful validation |
| `--export-only` | off | emit semantic bundle without rendering; default requires no Node |
| `--export` | temporary | retain the versioned semantic bundle at this directory |
| `--check` | off | stage and validate a production build without replacing site output |
| `--validate-examples` | off | explicitly execute eligible examples |
| `--runtime` | none | compatible browser engine/Wasm assets |
| `--base-url` | `/` | host path with leading/trailing slash |
| `--name`, `--logo`, `--repo`, `--playground-url` | matching `[doc]` setting | publication/display overrides |
| `--version` | none | displayed release and since-label selection |
| `--source-ref` | checkout commit | Source link ref |
| `--module` | all | explicitly partial module selection |
| `--allow-broken-links` | off | explicit broken-document-link policy; does not relax schema/path/graph checks |
| `--site-url`, `--og-image` | `[doc] url`, `og_image` | publication origin/image overrides |
| `--timings` | off | measured build phase reporting |
| `--theme`, `--emit-input` | retired | migration errors; use Astro source and `--export` |

`JAIC_STDLIB` selects the owned compiler's stdlib. `JACKET_DOCS_BUILDER` selects a matching explicitly installed source builder. Node/package versions, export/build environment inputs and developer setup are documented in [Astro docs development](astro-docs.md).

## Dependencies

The compiler parser/resolver, Jacket's typed producer and config, shared doctest templates, pinned Markdown parser, Rust schema derivation, and the pinned Astro/TypeScript/runtime validators. Rendering requires Node after explicit setup; default semantic extraction does not. Browser Run requires module workers, WebAssembly and supplied assets from the owned browser compiler build.

Documentation resolves the selected configuration directory and library directory to their canonical filesystem identities before deriving module category paths. This keeps relative categories such as `lib/**` stable for Windows verbatim paths and lexical directory aliases such as `child/..`. Configuration discovery still chooses the nearest authored `jacket.toml`; guide and asset paths remain relative to that selected configuration directory.

The installed builder runs Node with the fixed relative entry `scripts/build.mjs` from its validated canonical builder directory. The entry argument contains no Windows namespace prefix; canonical paths used for identity, filesystem arguments, and output checks remain unchanged. No drive or UNC prefix is stripped or translated. Export and output paths pass through unchanged; unsupported filesystem paths fail at the actual Node operation rather than being rewritten. The relocated installed smoke records the actual Node entry, arguments, and working directory for root, nested-base, and supported symlink launches. Python’s separate Astro development launcher retains its own path handling. Native UNC-share execution requires an actual share and is not established by the drive-path regression.
