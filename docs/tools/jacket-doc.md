# jacket doc

`jacket doc` writes the API reference for a Jai library as a static website: one page per module and per item, a search index, and any extra Markdown pages the project lists. The standard library's site is one such library, set up in the root `jacket.toml`. Every doc-comment example gets a Run button that compiles and runs it in the browser with the playground's compiler build. Nothing it makes is committed; a release builds it, attaches `jacket-docs-<version>.tar.gz` and deploys it to https://jacket.matteopolak.com. Writing docs and examples is in the [documenting guide](../guides/documenting.md); this page is the reference and how the site is built.

```sh
jacket doc                                  # the [doc] library of the nearest jacket.toml, else the standard library
jacket doc path/to/my_lib                   # a module (folder with module.jai) or a folder of modules
jacket doc --module String --module Basic   # just these, for a quick look
jacket doc --check                          # build in memory and check links only
```

Flags are in the [command-line reference](../reference/tools.md#documentation). Without `--runtime` the Run buttons are disabled; hovering one says why. Serve the output with a static server that finds `/api/Basic/print` in `api/Basic/print.html` (`npx serve target/doc-site`; in this repository `python3 tools/dev/doc_serve.py`); `python3 -m http.server` does not. Links are absolute from `--base-url`, so opening files from disk won't style.

## How it works

1. **Facts from the compiler.** `jaic -doc_model <dir>` (`crates/jaic-syntax/src/doc_comments/model.rs`) reads a library with the language server's parser and prints a JSON model: public constants, aliases followed to their targets, overloads, parameter and result types, doc comments with links resolved, fields and members, platform limits, and for each type name the declaration the compiler resolved it to. A module with a public top-level `#insert` is also run once (see Top-level `#insert` below).
2. **The view (Rust).** `crates/jacket/src/doc/` works out everything a page shows and writes it as JSON: `site.json`, `modules/<n>.json` (a module page and its item pages) and `guides/<n>.json`. `view.rs` holds the structs, `view/build.rs` fills them, `site.rs` decides where pages live, signatures and links, and `categories.rs`, `markdown.rs`, `highlight.rs`, `search.rs`, `since.rs` and `metadata.rs` do the rest. Prose, highlighted code and linked signatures arrive as ready HTML in `*_html` fields; everything else is plain data. A type name links to the item the compiler resolved it to, never by matching the name.
3. **The pages (Jai).** The theme in `crates/jacket/src/doc/site/` is a Jai program on [Extensions/Web](../extensions/web.md). It reads the view, renders every page with `render_parallel` (a job per module, big modules split into chunks), copies the assets and runs the link check. Components are procedures named after what they draw (`ItemRow`, `Crumbs`, `RenderSidebar`); each page kind supplies its own `Side`, `Main` and `Toc`. `contract.jai` mirrors `view.rs`; both sides check `format`.
4. **Examples.** A ```` ```jai ```` fence is read by the function `jacket test` uses (`test::doctest::fenced`), so `ignore`, `no_run` and `skip_on(wasm)` mean the same. The Run button fetches `examples/<id>.json` and runs the program in a worker with a step budget. That file holds the program `jacket test` builds as the text around the example (`head`, `middle`, `tail`, from `doctest::generate_template`), the same for the playground program (`open`) and the playground URL. The page puts the example's current text into the hole (`fill` in `site.js` mirrors `Template::fill`; a unit test keeps both equal to `generate` and `standalone`). "Open in playground" goes to `<playground>#code=<base64url of the program>`.
   - Examples with a Run button are editable: a transparent textarea lies over the highlighted `<pre>`, and `site.js` recolours the text as you type with a small copy of `highlight.rs`'s rules. Run, Copy and the playground link use the edited text. Without JavaScript the code is a plain highlighted block.
5. **Search.** `search-index.json` loads on the first keystroke. A query matches by name; one with `->` matches overload types (`string -> [] string`).

**Testing examples.** A build runs every example that has a Run button and fails if one fails. Each runs as the exact program the page sends, for the `wasm` target, with the `// test:` header of the module's `module.jai` applied as `jacket test` does, and is stopped at its own `timeout(n)` (60 s without one). Without `--runtime` they run in `jaic -interp -os wasm`, three at a time; with `--runtime` they run in the bundle's own `engine.mjs` under Node, which is what the browser loads. `--runtime` is an error when `node` is not on `PATH` and when the library has extra import folders. For a library other than the standard library the library's own `.jai` files go to the compiler below `modules/` beside each example, in the test and in the site (`runtime/modules.json`). Neither engine is picked by what happens to be installed. Mark an example that can't run in a browser (threads, sockets, files, native libraries) `skip_on(wasm)` or `skip_on(.WASM, reason = "...")`; a whole module goes in its `// test:` header. `--no-test-examples` skips this and `--check` never runs examples.

**Running the theme.** The theme's files are embedded in `jacket` (`crates/jacket/build.rs`). On first use `jacket doc` writes them to `<user cache>/jacket/doc-site/<key>/` and compiles them with `jaic -o`; the key hashes the jacket and jaic versions, the theme files, any overrides and `stdlib/Extensions/Web`, so later runs reuse the binary. The user cache is `~/Library/Caches` on macOS, `$XDG_CACHE_HOME` or `~/.cache` on Linux, `%LOCALAPPDATA%` on Windows. A `jaic` without the native backend can't build the theme: `jacket doc` stops with an error asking for one and does not interpret the theme. The view goes to a temporary folder deleted afterwards; `--emit-input <dir>` keeps it. Rust itself writes `search-index.json`, `examples/`, `versions.json`, `runtime/` and the logo. `--timings` shows the phases.

## Output

Pages are linked without a trailing slash: `/api/Basic/print` is the file `api/Basic/print.html`, and the front page is `/` (`index.html`). GitHub Pages, Netlify and Cloudflare Pages look a path up as `<path>.html`; a server that doesn't needs that rule. An old link with a trailing slash lands on `404.html`, which `site.js` sends to the slashless address. A module and its items share a name (`api/Tls.html`, `api/Tls/Thing.html`). Characters that can't be in a file name become `-` plus a hex code (`operator-3c`). Names that differ only in case (`Head`, `head`) would share a file on macOS and Windows, so the lowercase one keeps its name and the others add their kind (`Head-struct`); the search index carries such a name as an extra column (`case_folders` in `site.rs`).

| Path | What |
| --- | --- |
| `index.html` | the front page: the `[doc] home` file, else the name and description |
| `api/<Module>.html`, `api/<Module>/<name>.html` | a module; an item |
| `<pages_title>/<page>.html` | the `[[doc.page]]` pages |
| `<section>.html`, `<section>/<heading>.html` | a `[[doc.section]]`: its overview and one page per `###` |
| `examples/<id>.json` | the parts of each example's program, for Run and the playground link |
| `runtime/`, `runtime.json` | the browser compiler (`--runtime`) and where pages look for it |
| `assets/` | `site.css`, `site.js`, `run-worker.js`, `icons.svg` (the icon sprite), `categories.css`, `favicon.svg`, the logo, the social image. Pages link the CSS, scripts and sprite with `?v=<content hash>`, so they can be cached for a long time |
| `404.html` | search and the module list; links are absolute from `--base-url`, so it works at any depth |
| `sitemap.xml`, `robots.txt` | with `[doc] url` only; past 50,000 URLs or 50 MB, `sitemap.xml` is an index of `sitemap-1.xml`, … |

Modules are the folders of the library with a `module.jai`, minus `internal/` and `tests/`. `prelude/` is a module page too; since every program imports it, the page says so instead of showing an `#import` line (`implicit` in the module JSON). When the path is itself a module, its id is the folder name and the modules inside it are `name/sub`. A library's own folder is passed to `jaic` as `-import_dir`, so its modules can import each other, followed by the nearest `jacket.toml`'s `[project] import_path` (`compile::project_flags`, as for `jacket build` and `jacket test`), then any flags after `--` (`jacket doc lib -- -import_dir vendor`; relative paths are read from the current folder). Every `jaic` run of the build takes these: the model, the wasm check and the interpreter that tests examples. Put only import folders there; a `-os` or `-cpu` would fight the wasm check. The model finds imports through those folders and then the standard library, so a type from another module of the library links to its page.

### Link checks

`--check` (and every build) fails on:

- a doc-comment link written like an intra-doc link (`[Name]`, `[Module.item]`, `` [`name`] ``) that names no item of the library or of another module, printed as `file:line: [Name] names nothing` (`docs::unresolved::unresolved`, listed as `unresolved` in the model JSON). Prose such as `[x]`, `[TODO]` and a plain lowercase word is left alone. A bare name resolves only within the library, so link another module's item as `[Basic.print]`. `--allow-broken-links` turns these into warnings.
- internal links ending in `.html` or `/`, links to files that weren't written, and an item and module of the same name.
- a type in a signature, field or type value with no page to link to (`type_links.rs`). Exempt: language builtins and prelude names, polymorphic and module parameters, and private declarations with no documented re-export. There is no allow list; fix a failure by documenting or re-exporting the type. A type declared in a module that is not part of the site (`Json_Value` in `Extensions/Gltf` documented alone) can't have a page, is not counted, and shows as plain text.
- a generated item with no doc comment (see Top-level `#insert`).

### Module pages

- **Module parameters.** Each `#module_parameters` of the module's files becomes a "Module parameters" section before the items: name, type, default, and the doc comment above or after the parameter (one parameter per line so each has its own comment). Names in the type and default link to what they resolve to. They are not counted by `jaic -doc_gaps`. Declarations in a `#module_parameters` body (an enum a parameter's type needs) are ordinary items, and `#scope_file`/`#scope_module` inside the body hides them as at the top level.
- **Native bindings.** A file exporting a top-level `#foreign` procedure mirrors a native API (files under `internal/` don't count, `#intrinsic` or `#compiler` procedures never make a file a binding, and private native helpers under `#scope_file` or `#scope_module` don't mark the API as foreign). The model marks such a file's declarations `"binding": true`; the module page lists them in the usual sections after the module's own items, with a "Foreign" badge, and each gets an item page. A procedure with a body in such a file is not a binding. Search ranks bindings below the library's own API, and their `since` label is left off. `jaic -doc_gaps` uses a broader rule: it skips every file that exports a bodiless top-level procedure, including an exported intrinsic; private bodiless helpers don't count.
- **Rows.** A struct, union or enum row shows its fields or members inline, folded with "Show more" past 200px (opening is one-way until reload). A constant that instantiates one of the module's own structs (`u8x16 :: Vector(u8, 16)`) is listed as a type.
- **"On this page" rail.** `site.js` (`outline()`) opens every list with a section on screen, marks the items on screen with a guide line (`--bar-y`, `--bar-h`) and folds the rest as you scroll; a list the reader opens stays open. Without JS every list is open.

Page chrome ids (`search-q`, `search-results`, `theme-toggle`, `ver-button`, `main-content`) contain a `-` so they can never equal an item's anchor, which is its Jai name.

### The browser compiler

`runtime.json` names the files of the browser compiler: `engine`, `wasm`, `formatter` (the jaifmt driver) and `examples` (the playground's index). They come from the bundle `tools/browser/build_scripting_wasm.py --release --output <dir>` writes ([browser build](../browser/playground.md)). `--runtime <dir>` copies `jai_wasm.wasm`, `engine.mjs`, `jaifmt-playground.jai`, `examples.json` and every example file the index lists into `runtime/`, and fails with the build command when one is missing. A site that already hosts the compiler can drop the copy and edit `runtime.json`; the site never loads the compiler from another host.

## Playground

Every site has a page at `playground` (`playground.html`, made by `doc/playground.rs`; markup in the theme's `site/playground.jai`, behaviour in `assets/playground.js`, `assets/playground-layout.js` and `assets/playground.css`). The header's Playground link and every example's "open in playground" link go to it. `[doc] playground` or `--playground-url` replaces them with another address; the page is still written. Without `--runtime` the page says how to build a bundle and example links are left out. The jacket binary never needs Node: `--runtime` only copies files.

The page loads `runtime.json`, starts `assets/run-worker.js` and keeps the compiler warm in it. Run sends the files, the main file and `{ budget, args, stdin }` to `engine.play`, the request the hosted playground made. Format runs the bundle's `jaifmt-playground.jai` driver with a generated `jacket.toml`. Hover, go to definition, messages and quick fixes come from `engine.lsp` in the same worker; a bundle without it leaves these off. Definitions in the standard library open in a read-only tab, fetched with `jai/source`.

The target is fixed: the browser build interprets the program for WebAssembly, so there is no optimisation level or native target to offer.

| Control | Choices | Effect |
| --- | --- | --- |
| Examples (toolbar menu) | the bundle's `examples.json` | loads a workspace of one or more files |
| Step limit | 20 million, 200 million, 2 billion, 20 billion | interpreter budget; a program over it stops with "execution budget exhausted" |
| Arguments | text | the program's `argv` after its name; quotes group words |
| Standard input | text | the program's whole input |
| Messages | errors and warnings, errors only | which diagnostics the Problems list shows |
| Indent, Braces | 2, 4 or 8 spaces; same line or as written | `indent-width` and `brace-style` of Format |

State lives in the address fragment so links are shareable: `#code=<base64url>` for a one-file program (the form example links use) or `files=<base64url JSON>&file=<name>`, then `example`, `limit`, `args`, `stdin`, `show`, `indent` and `braces` when they differ from the defaults. `?code=` (raw text) is read too, for Zulip. The last code and options are kept in `localStorage` (`jacket-playground`) and used when the address has none; the panel layout is kept in `jacket-playground-layout` and `jacket-playground-width`. Below 720px the layout is a one-pane switcher.

To add a control: add the element with `data-opt="<key>"` to `playground.jai`, the key to `Options` and `DEFAULTS` in `playground.js`, and use it where the request is built. Keep only options the engine honours.

## Settings: `[doc]` in `jacket.toml`

`jacket doc` with no path documents the `[doc]` library of the nearest `jacket.toml` (from the current folder), else the bundled standard library with no settings. With a path, the nearest `jacket.toml` above it applies only if its `[doc]` is for that folder; otherwise the site uses defaults. Paths and patterns are relative to the `jacket.toml`. Flags win over settings.

```toml
[doc]
path = "src/my_lib"                 # default: the jacket.toml's folder
name = "my_lib"                     # header and titles; default: the folder name
breadcrumb = "my_lib"               # first crumb on every page; default: the folder name
description = "API docs for my_lib."
home = "notes/home.md"              # the front page; default: the name and description
logo = "assets/logo.svg"            # .svg or .png for the header and tab; default: none
icon = "assets/icon.svg"            # tab icon when it should differ from the logo; default: logo
repo = "https://github.com/me/my_lib"   # Source links need it
playground = "https://example.com/play"  # default: the site's own playground page
since = "notes/since.tsv"           # "since X" labels; used only with --version
pages-title = "Guides"              # sidebar heading and folder of the [[doc.page]] pages
url = "https://docs.example.com"    # canonical origin: canonical links, og:url, sitemap.xml, robots.txt
og-image = "assets/social.png"      # a file (copied to assets/) or a URL; 1200x630 suits most sites
site-name = "my_lib"                # og:site_name; default: name
canonical-version = "1.4.0"         # other versions point their canonical here and get noindex
theme = "doc-theme"                 # files that replace or extend the built-in theme
files = ["install.sh"]              # published as they are at the site root (/install.sh); a name a page uses is an error

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
color = "oklch(51% 0.1 70.8deg)"      # oklch(), #hex or a colour name; default: muted
color-dark = "oklch(80% 0.11 75.4deg)" # default: color

# Folding groups inside each heading. With any, every module must be in one (last match wins).
[[doc.group]]
name = "Core"
modules = ["src/my_lib/Core", "src/my_lib/Util"]

# Extra pages, listed under `pages-title` in the sidebar. The header's link goes to the first.
[[doc.page]]
file = "notes/guide.md"
name = "Guide"

# A sidebar heading of its own, split from one file (see "Sections" below).
[[doc.section]]
file = "notes/recipes.md"
name = "Recipes"
```

`[doc.format]` is not a site setting: it holds `jaifmt` overrides for the examples in doc comments ([jaifmt-rules](jaifmt-rules.md#format-doc-examples)).

A pattern matching a module's folder also matches the modules inside it. `jacket doc` fails on a pattern that matches no module, a module in no group (when there are groups), and a colour that is not `#hex` or a plain name. The header shows the version, repository button, playground link and pages link only when set.

The standard library's `[doc]` sits in the root `jacket.toml`: categories "Standard library" (`stdlib/**`) and "Extensions" (`stdlib/Extensions/**`, teal "extension" badge), the topic groups, and the "Tools", "Lint rules" and "Format rules" sections (generated into `docs/reference/` and `docs/tools/jaifmt-rules.md`). Add new stdlib modules to a group there.

**Sections.** A `[[doc.section]]` file is split at its headings: the text before the first `##` (plus each `##`'s own text and a list of its pages) is the overview at `<name as a slug>/`, each `##` is a folding group in the sidebar, and each `###` is a page at `<section>/<heading slug>/` titled with the heading. A `###` before any `##` goes in a group with no label. Inside a page, `####` and `#####` headings move up to `##` and `###`, so they reach its outline. The header link (`pages-title`) goes to the first `[[doc.page]]`, else to the overview of the section with that name. A link to `#heading` or `file.md#heading` of a split file goes to that heading's page. A `##` with no `###` under it (the Format rules' "Indentation") stays text on the overview, so `format-rules#indentation` is still an anchor there.

**Home.** `[doc] home` names a Markdown file rendered as the front page, with the module sidebar. Links in it (and in every extra page) to a module's folder or `module.jai` go to that module's page, so `[Basic](../../stdlib/Basic)` works both on GitHub and on the site. Without it the front page shows the name and description; the stdlib's is `docs/reference/home.md`. Raw HTML in any Markdown the site renders shows as text, except a bare `<kbd>` and `</kbd>` for a key (`Press <kbd>/</kbd>`); one left open closes with its paragraph. Configured pages and sections come first, followed by module categories, wherever the module sidebar is shown (`view/guide.rs`, `Site::extra_nav`).

**Before and after.** Two fences in a row, ```` ```jai before ```` then ```` ```jai after ````, render as one block split by a line ("Before", "After"), stacked under 700px. Words after `after` are shown as the settings used (`indent-width=2` reads "After, with `indent-width = 2`"). **Example, finding and fix.** A ```` ```jai example ```` fence, then optionally a ```` ```diagnostic ```` fence and a ```` ```jai fixed ```` fence directly after it, render as one card with a caption over each block ("Example", "Diagnostic", "Fixed"). `docs/reference/lint-rules.md` is written that way (`crates/jailint/examples/lint_reference.rs`). In a diagnostic, `-` marks a removed line of a fix preview. Jai fences on these pages are highlighted and scroll sideways; other fences are plain and wrap long lines (usage lines, shell output).

**Console blocks.** A ```` ```console ```` fence holds commands, each on a line starting with `$ `, and the lines each one prints. `jacket doc` (with `--check` too) runs them and fails when the output differs from the block, so a guide can't drift from the tools. `--no-test-examples` skips them.

````text
```console cwd=../examples/testing/slugs
$ jacket test
    Starting 4 tests
```
````

- **What runs.** `jacket` (this one), `jaic` and `cat`, without a shell: quotes group words, and `|`, `>`, `$` and globs are errors. Commands run in `cwd=` (from the guide's folder; the guide's own folder without it), in turn, with stdout and stderr merged in order, colour forced, `JAIC_PROGRESS=plain` (the `Compiling`, `Linker` and `Finished` lines a terminal shows, without the redrawn status line), and `CI`, `NO_COLOR`, `JAIC_DIAGNOSTICS` and `COLUMNS` unset. Files they make are removed afterwards; changing a file that was there is an error.
- **Comparing.** Line ends, trailing spaces and blank lines at the end are ignored, and times in brackets (`[   0.057s]`) match any time. The diff names the guide's line.
- **Colours.** The markdown holds plain text. What the commands printed, escapes and all, is kept beside the guide in `<guide>.console.json`; the site renders its SGR colours (bold, dim, the 16 colours) as `ansi-*` spans under a "Terminal" caption. A block whose capture is missing or stale is an error, and renders as plain text: colours never come from guessing.
- **`cache`.** ```` ```console cwd=... cache ```` is run only when its capture is missing or no longer matches its text, so `jacket doc` keeps what it printed once. Use it for output that depends on the machine: timings, installed cross tools, the toolchain cache. To run one again, edit its text or delete its capture.
- **`jacket doc --accept`** writes what the commands printed into the guides and the captures. A block whose text still matches, apart from times, is left alone.
- **Example projects.** The folders the guides run in live in `docs/examples/`. Set `[test] jobs = 1` in their `jacket.toml` so tests print in a fixed order.

## Page metadata

Every page gets a title made from `[doc] name` (`String.trim — Jacket`, `String — Jacket`; the index is the name alone), a description from the summary (plain text, at most 155 characters), `lang`, `theme-color` for both schemes, a favicon, Open Graph and Twitter tags. Item pages carry `TechArticle` JSON-LD, and pages with a breadcrumb a `BreadcrumbList`. Without `[doc] url` there is no canonical, `og:url`, sitemap or `robots.txt`, and `jacket doc` warns once. A page with an empty description gets no description tags, and `404.html` has no canonical or `og:url`. Rust works out these facts (`metadata.rs`, into each page's `head` in the view) and `HeadMeta` in `shell.jai` writes the tags, leaving out any that are empty.

Versions: the current release lives at `<url><base>`, older ones at `<url><base>versions/<version>/` (built with that as `--base-url`). With `canonical-version` set and different from `--version`, each page's canonical points at the same path under that version and the page is `noindex`.

## Accessibility and layout checks

The site targets WCAG 2.2 AA: landmarks, a skip link, one `h1` per page, real lists and buttons, `aria-expanded` on folds and nav categories, layouts that work at 320px, smooth scrolling only without reduced motion, and a tab stop on wide code blocks, tables and field lists so they can be scrolled.

- **Search** is a modal `<dialog id="search-dialog">` opened by `#search-open`, `/` outside text fields, or Cmd/Ctrl+K. Result icons come from `assets/icons.svg` (`k-proc`, `k-module`, `k-type`, `k-const`, `enter`; add a kind with a symbol and a `.kind.k-<kind>` colour in `site.css`). Search-row classes are prefixed (`hit-sig`) where a page class of the same name would style them.
- **Colour.** Every colour in `site.css` and the theme is `oklch(L% C Hdeg)` (stylelint enforces the notation). `tools/dev/test_doc_palette.py` checks every colour token in both themes at 4.5:1 for text. `jacket doc` rejects a `[doc.category]` `color` or `color-dark` under 4.5:1 and prints the ratio; it reads `oklch()` (out-of-gamut colours are clamped to sRGB), `#hex` and colour names.
- **Whole-pixel rows.** Every row in the sidebar and the outline has a whole-pixel height (`line-height` in px, not a ratio): a part-pixel row is rounded differently in the page-change transition's snapshot than on the page and would jump.
- **CI.** `docs-pages.yml` runs two Chrome checks after the build (it needs Chrome; `CHROME_PATH` overrides its location): `tools/dev/doc_a11y.ts <url>` runs axe-core on six pages in both themes at 1280px and 320px (and with the drawer open at 320px), and `tools/dev/doc_layout.ts <url>` loads a sample of page kinds at 1280px and 375px in both themes and fails on any `layout-shift` entry or any key box that differs between a frame and where the page settles. Its scenarios are a fresh load, a reload after scrolling, leave and Back, in-page swaps and the drawer. It holds `site.js` back 250ms (`--script-delay`) so a frame paints without it, as on a slow connection. A page-kind sample takes about two minutes; `--all --every 600` samples every 600th page. Serve the site with `tools/dev/doc_serve.py`. `DOCS_BROWSER_CHECKS=false` disables both, and an opted-out run is no browser evidence; validate a repair with a manual run, `browser_checks=true` and `deploy=false`.
- **First paint.** Anything that must be right at first paint belongs in `early.js` or the markup, not `site.js`. The outline's column is held open by `body > .wrap[data-toc]::after` until the outline is parsed (the template sets `data-toc`; `swap()` copies it). `early.js` closes the drawer and restores saved groups as their nodes are parsed, and drops the buttons of folds that fit; `jacketRestore()` (after the footer) finishes a scroll restore the unparsed footer cut short. Runnable examples render their `.ed` wrapper before paint; `site.js` only adds the textarea.
- **Unit tests** that need no Chrome: `node --test tools/dev/test_chrome_startup.ts tools/dev/test_doc_samples.ts tools/dev/test_doc_sidebar.ts tools/dev/test_doc_search.ts`.

## Publishing

The stdlib site is served by GitHub Pages at https://jacket.matteopolak.com. `.github/workflows/docs-pages.yml` builds it with `jacket doc --base-url / --version <version> --source-ref <tag> --runtime <bundle>`:

- A release calls it from its `docs` job, which also attaches `jacket-docs-<version>.tar.gz`. The release's `pages` job deploys the site once the release is published, for the current release only.
- A manual run builds a tag or branch and deploys it (default: the latest release tag); a branch build shows the branch name as its version. To ship a site fix before the next release, build `main` with the release's version and Source links:

```sh
gh workflow run docs-pages.yml -R matteopolak/jacket -f ref=main -f version=0.38.0 -f source_ref=v0.38.0
```

- `-f deploy=false -f browser_checks=true` builds and checks a branch without deploying it.

Repository settings: Pages source "GitHub Actions", custom domain `jacket.matteopolak.com` (a `CNAME` record to `matteopolak.github.io`), and the `github-pages` environment allows `main` and `v*` tags. The Pages limit is 1 GB per site. CI runs `jacket doc --check` when the stdlib, the compiler or the docs change.

A successful build does not show that the public site serves it. To check a deployment, compare `gh api 'repos/matteopolak/jacket/deployments?environment=github-pages'` with the run's `headSha`, then fetch the public home, a guide, a module and an item at their slashless URLs. A nonexistent nested path must return HTTP 404 with the custom page, and an old trailing-slash item URL must recover to its slashless page. `sitemap.xml` (and every shard of an index) must use the configured origin, and `robots.txt` must reference it. The 404 page has no canonical or `og:url`. Never deploy a development branch over production to check.

The implicit `#Context` reference is a configured Markdown guide at `guides/context`, linked from the home page. Directive identifiers such as `#Context` are not declaration names, so the doc model does not resolve them to a same-spelled user type.

## How to change it

- **What a page says** comes from the doc comment ([doc comments](../compiler/doc-comments.md)). Document parameters with `- name: text` or a comment on the parameter.
- **Module names.** A module has three: `id` (its page's place below the documented folder, so `Json` when only `stdlib/Extensions/Json` is documented), `doc_key` (what the compiler's `jaidoc:` links call it) and `import-path` (what an example imports). Links resolve through `doc_key` (`Site::module_for_link`); examples and the import line use `import-path` (`Module::import_name`). Two modules with one `doc_key` leave their links unresolved rather than guessing.
- **A fact the compiler should supply:** `model.rs` in `doc_comments/` and the JSON it prints, then `doc/model.rs`.
- **Layout and colours:** `doc/site/assets/site.css` (colours are custom properties at the top, with a dark set; run `test_doc_palette.py` after changing one). Behaviour: `doc/site/assets/site.js`, which only adds to pages that read fine without it; it reads the header's height into `--header-offset` (61px until it runs). Lint and type checks: [doc site assets](doc-site-assets.md).
- **Markup** is the Jai theme: `shell.jai` (head, header, footer, layout), `sidebars.jai`, `index.jai`, `module.jai`, `item.jai`, `guides.jai`, `examples.jai`, `playground.jai`, `parts.jai` (badges, crumbs, Source links). A new value on a page goes in `view.rs`, `view/build.rs` and `contract.jai` together. Markup tests are in `site/tests/suite.jai` with JSON fixtures; `cargo test -p jacket doc::` runs them (they need a built `jaic` beside `jacket` and are skipped otherwise), as does `jacket test --path crates/jacket/src/doc/site`.
- **Runnable example markup:** `ExampleCode` in `examples.jai` emits `<div class="ed"><pre class="code">…</pre></div>` when an example has a program id. Custom templates must keep that wrapper; JavaScript attaches one textarea to it.
- **Your own theme:** `[doc] theme = "folder"` (or `--theme`). Each file there replaces the built-in file at the same path and new files are added (hidden files are skipped), so overriding `assets/site.css` or `item.jai` is enough. A compile error in an override is printed as jaic reports it. `--emit-input in/` writes the view so you can see what a template gets. Compare two builds with `python3 tools/dev/doc_diff.py OLD NEW`.
- **Module page grouping** is by declaring file (`bytes.jai` is "Bytes"). When procedures are split into groups, foreign bindings get one "Foreign" group instead of their file's, so each item is listed once. Words with their own casing go in `WORDS` in `model.rs`.
- **Source links** need `repo`. They go to the commit `jacket doc` was built from when the library is in a git checkout (a file with uncommitted changes gets none); `--source-ref` overrides. Outside a checkout, or when `git` fails, and without `--source-ref`, there are no Source links and `jacket doc` warns. Guides get the same warning: their links that leave the config folder resolve from the checkout's top.
- **Versions menu:** `versions.json` lists one version; a site hosting several replaces it with one entry each.
- **"Since" versions.** `since 0.24.0` labels come from the file `[doc] since` names, and only when `--version` is given; the stdlib's is `crates/jacket/src/doc/since.tsv`. `tools/generators/generate_doc_since.py` writes it from the release tags (`check_generated.py` only validates it). An item with no row is new in the version being built. After a release is tagged, run the generator and commit the file (it adds only new tags; `--full` rereads all).
- **Indented code** in a doc comment is a plain code block and never run; `jacket doc` warns for each. Use ```` ```jai ````; `jai,ignore` for fragments, `jai,no_run` for complete ones needing a file or network.
- **Platforms.** A module page lists every system under its title as a chip: green when everything works there, amber when some items aren't available (the tip says how many), red when the module doesn't build there. A module is available on the systems it asserts (`#assert OS == ...`), else macOS, Linux and Windows, plus Wasm when it builds for `-os wasm`; Android and iOS are listed only for modules that name them (`Module::support` in `doc/model.rs`, `platforms` in `view/build.rs`, `.pf` in `site.css`). An item row gets a small red chip for each system its module runs on but it does not (`limits`/`holds_on` in `model.rs`, `Module::item_missing`). An item page lists the same systems as its module page, green or red for that item alone (`Module::item_support`).
- **Procedure pages.** A procedure card is its signature, description and examples; there are no Parameters or Returns sections. Docs for a parameter or result are listed under the signature (`notes` in `OverloadView`), only for those with text. Declarations of one signature in the branches of an `#if`, or in files loaded for different systems, are one procedure limited to the systems of them all (`model_overloads.rs` in jaic-syntax); only different signatures or docs stay as separate cards. A lone overload's first paragraph is the page's summary and its card keeps the rest; with several, the summary repeats the first one's lead. With several overloads each is a card headed "Overload N". Module rows and search results list every overload's signature.
- **Item pages.** The signature (or a constant's value) comes first, then the description. A procedure's doc comment belongs to its overload: the model leaves the item's own `docs` empty. Only an alias (`twice :: scale;`) keeps a comment of its own, shown as the page's lead.
- **Alerts.** GitHub's alert syntax (`> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`) in pages and doc comments renders as a block quote with the kind's title and colour (`markdown::segments`, `.markdown-alert-*` in `site.css`).
- **Badges.** A doc comment line that is exactly `@experimental` adds an amber badge (`ExperimentalBadge` in `parts.jai`); every item of an experimental module inherits it (`Site::new`). A category with a `badge` puts it on its modules and items (`CategoryBadge`), coloured by `assets/categories.css` (`CategoryStyle` in `shell.jai`, classes `.cat-N`). Sidebars and search show them as coloured dots (`.dot` in `site.css`). In the search index a row's flags hold one more than the badge's category index above the low three bits, and `cats` lists each category's badge and tip. The Foreign flag is bit 2 (`ForeignBadge` in `parts.jai`; `search.rs`, `site.js`) and Generated bit 4.
- **Search order.** `byName` in `site.js` scores each term against an item's name: exact, then a prefix that ends a word (`time` in `time_of_day`), a bare prefix, a word inside the name, any letters, the `Module.name` path and the summary. Items of the module a term names get a small bonus, and a module is matched by the last part of its path. Foreign bindings score a quarter. `tools/dev/test_doc_search.ts` runs the real `load` and `byName` over a small index.
- **Linked types.** Signatures, fields and the values of type aliases and constants link each type to its page (`Item.refs` from the model, `Site::type_html`). The model resolves a name through every alias hop in order and links the nearest one with a page, so `background: Color` goes to `Color`, not to the `Vector4` it aliases (`model_resolution.rs`). Resolution follows `using` imports, loaded files and module parameters, and ignores names inside strings. A constant whose value is a type (including a procedure type) is kind "type"; its page shows the alias as written rather than the target's fields.
- **One name, several definitions.** A struct, enum, type or constant defined in more than one branch of an `#if` (`File_Watcher` is one struct on macOS and another elsewhere) keeps every definition that differs from the first (`variants` in the model JSON, each with its own `platforms`; `defined_on` is the first's). Branches that say the same stay one definition. The item page shows each under its systems (`Variant_View`, `variant_views` in `view/build.rs`; anchors start with `d<n>-m-`), a module row lists each (`RowVariant`), and search covers them all.
- **Declarations.** A struct, union or enum page opens with its declaration as written, without the comments that are a field's docs (`declaration` and `decl_refs` in the model JSON, `highlight::declaration` for links). Each field's doc is written above its line as `// ...` comments (`commented_declaration` in `view/build.rs`; the model gives each member its `line` in the declaration, counted after those comments are removed, `Item::member_lines` in `doc/model.rs`), and the line is `id="m-<name>"` so `#m-<field>` links highlight it (`:target` in `site.css`). A doc of more than one paragraph or with a code block shows its first sentence there and keeps a Fields (Members) table row with the full text. A field with no line (a name in a nested struct) keeps a table row if it has docs. Both branches of an `#if` in a struct list their fields; the condition and the operand of a value directive (`#run`, `#place`, `#code`, `#insert`, `#location`, `#bake_arguments`) are never type references. A `[Type.field]` link goes to the field's line or row, else to `#declaration`.
- **Private types.** A `#scope_module` or `#scope_file` struct, union or enum that a public item names (directly or through other private types' fields) gets a page marked "Not exported" with `noindex`, so links to it work. It is left out of the module page, sidebar, search and sitemap (`Module.private` in `doc/model.rs`). A private type whose name clashes with a public item is skipped.
- **Top-level `#insert`.** `jaic -doc_model` (`jaic::doc_inserts`) compiles a module that has a public top-level `#insert` (not under `#scope_file` or `#scope_module`; `ide_facts::top_level_inserts`) once, as `#import "Module";` with default module parameters, and takes the full text each insert produced (`IdeFacts::insert_sources`). That text is parsed like a file the module loads, so its declarations are items with docs from their own comments. A module with no such insert is never compiled. Compile-time code runs in the compiler's sandbox with the usual step limit.
  - **Generated.** The model names the text `<file>#insert:<line>` (`generated_path`, `generated_origin` in `doc_comments/model.rs`); each declaration has `"generated": true`, and its Source link goes to the `#insert` (`model::source_place`). Pages tag it "generated" (`GeneratedBadge`, `GeneratedDot` in `parts.jai`, colour `--generated`).
  - **Failures.** An insert that can't be expanded (a compile error, module parameters without a default) is a warning with its file, line and the compiler's message (`unexpanded` in the JSON, `unexpanded_warnings` in `doc/mod.rs`); what it declares is missing from the pages. One that expands to nothing, or sits in an `#if` branch that is off, is silent. An `#insert` inside generated text is followed to 32 levels (`MAX_NESTED_INSERTS`); past that, as when an insert inserts itself, it is a warning. A nested insert's declarations link to the outermost `#insert`.
  - **Docs.** A generated item with no doc comment fails the build and names the `#insert` that made it (`undocumented_generated`); the comment has to be in the text the generator emits.
- **Item sidebar.** An item page's sidebar holds up to five items of its group around the current one, and `early.js` fills in the rest from `assets/nav/<module path>/<group>.html`, shared by every item page of the group. The list's padding (`--before`/`--after`, 30.5px a row in `site.css`) reserves the missing rows so the sidebar never changes height; change that constant if the row's font, padding or gap changes (`ItemNavList`, `WriteItemNav` in `sidebars.jai`).
- **Sidebar and drawer.** The sidebar is one `<dialog class="side" id="side">`: open beside the page from 900px up, and below that a bottom sheet opened by `#modules-btn` with `showModal()` (`setupDrawer` in `site.js`; `early.js` closes it before first paint on a narrow screen). Closing slides the sheet down first (`.closing`, then `close()`), and focus returns to the button. Without scripts the sidebar stays open above the page. If you change the 900px breakpoint, change it in `site.css`, `setupDrawer` and `early.js` together.
- **Moving between pages.** `site.js` prefetches a page inside the site when the pointer rests on its link (65 ms) or it gets focus, and a click swaps in `<main>`, the outline, sidebar, title and description without a page load, through `document.startViewTransition` (skipped under reduced motion). Back and Forward restore each page's scroll (`history.state.y`). Per-page setup is `setupPage` and runs again after each swap; listeners on `window` take its `AbortSignal`. Links with a modifier key, a `target`, or outside `--base-url` load normally, and so does any page that fails to fetch.
- **Long blocks.** A `.fold` taller than 440px shows its top with a faded edge and a "Show more" button. The first six folds are measured by an inline `fold()` call while the page loads (`EARLY` in `shell.jai`) so the top never flashes; `site.js` measures the rest in one pass.
- **Big pages.** The rail's scroll handler binary-searches the headings for the visible range, touches only entries that changed and reads positions before writing. A marked entry only changes colour, so marking never triggers a layout; a `font-weight`, `text-shadow` or `content-visibility` change on rail entries brings the cost back (the SDL3 page has about 78k elements).

## Configuration

| Flag | Default | |
| --- | --- | --- |
| `--out` | `target/doc-site` | replaced each run, only if `jacket doc` made it |
| `--base-url` | `/` | where the site is served, with both slashes |
| `--runtime` | none | folder with `jai_wasm.wasm` and `engine.mjs` |
| `PATH` | `[doc] path`, else the stdlib | the library to document |
| `--name`, `--logo`, `--icon`, `--repo`, `--playground-url` | the `[doc]` setting, else none | override `[doc]` |
| `--version` | none | the header's version; also turns on `since` labels |
| `--source-ref` | the checkout's commit | tag or branch for Source links |
| `--module` | all | build only these; broken links are then warnings |
| `--check` | off | build and check, write nothing |
| `--allow-broken-links` | off | warn instead of failing |
| `--site-url`, `--og-image` | `[doc] url`, `og-image` | override `[doc]` |
| `--theme` | `[doc] theme` | theme override folder, relative to the current folder |
| `--emit-input` | off | also write the view JSON to this folder |
| `--timings` | off | print each phase's time to stderr |

`JAIC_STDLIB` picks the standard library, as for `jacket test`; the default is the `stdlib/` of the project you run it in. The `[doc]` keys are listed above.

## Dependencies

`jaic` (`-doc_model`, and compiling the theme), `stdlib/Extensions/Web` and `Extensions/Json` (the theme), `jacket-config` (the `[doc]` schema and path patterns), `pulldown-cmark`, `serde_json`, `sha2`, and `crates/jacket/src/test/doctest.rs`. The Run button needs module workers and WebAssembly; the compiler's wasm build is `crates/jai-wasm`.
