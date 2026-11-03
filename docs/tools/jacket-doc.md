# jacket doc

`jacket doc` writes the API reference for a Jai library as a static website: one page per module and per item, a search index, and any extra Markdown pages the project lists. The standard library's site is one such library, set up in the root `jacket.toml`. Every doc-comment example gets a Run button that compiles and runs it in the browser with the playground's compiler build. Nothing it makes is committed; a release builds it, attaches `jacket-docs-<version>.tar.gz` and deploys it to https://jacket.matteopolak.com.

```sh
jacket doc                                  # the [doc] library of the nearest jacket.toml, else the standard library
jacket doc path/to/my_lib                   # a module (folder with module.jai) or a folder of modules
jacket doc --out site
jacket doc --module String --module Basic   # just these, for a quick look
jacket doc --check                          # build in memory and check links only
```

Flags are in the [command-line reference](../reference/tools.md#documentation). Without `--runtime` the Run buttons are disabled and say why. Serve the output with a static server that finds `/api/Basic/print` in `api/Basic/print.html` (`npx serve target/doc-site`; in this repository `python3 tools/dev/doc_serve.py`). `python3 -m http.server` does not. Links are absolute from `--base-url`, so opening files from disk won't style.

## How it works

1. **Facts from the compiler.** `jaic -doc_model <dir>` (`crates/jaic-syntax/src/doc_comments/model.rs`) reads a library with the language server's parser and prints a JSON model: public constants, aliases followed to their targets, overloads, parameter and result types, doc comments with links resolved, fields and members, platform limits, and for each type name the declaration the compiler resolved it to. Docs are found by the same rules as hover.
2. **The view (Rust).** `crates/jacket/src/doc/` works out everything a page shows and writes it as JSON: `site.json`, `modules/<n>.json` (a module page and its item pages) and `guides/<n>.json`. This is the *view*: `view.rs` (the structs), `view/build.rs`, `site.rs` (where pages live, signatures, links), `categories.rs`, `markdown.rs`, `highlight.rs`, `search.rs`, `since.rs`, `metadata.rs` (titles, descriptions, Open Graph, JSON-LD, sitemap). Prose, highlighted code and linked signatures arrive as ready HTML in `*_html` fields; everything else (rows, badges, crumbs, sidebars, outlines, example buttons) is plain data. A type name links to the item the compiler resolved it to, never by matching the name.
3. **The pages (Jai).** The theme in `crates/jacket/src/doc/site/` is a Jai program on [Extensions/Web](../extensions/web.md). It reads the view, renders every page with `render_parallel` (a job per module, big modules split into chunks), copies the assets, and runs the link check. Components are procedures named after what they draw (`ItemRow`, `Crumbs`, `RenderSidebar`); each page kind supplies its own `Side`, `Main` and `Toc`. `contract.jai` mirrors `view.rs`; both sides check `format`.
4. **Examples.** A ```` ```jai ```` fence is read by the function `jacket test` uses (`test::doctest::fenced`), so options like `ignore`, `no_run` and `skip_on(wasm)` mean the same. The Run button fetches `examples/<id>.json` and runs the program in a worker with a step budget. That file holds the program `jacket test` builds as the text around the example (`head`, `middle`, `tail`, from `doctest::generate_template`), plus the same for the playground program (`open`) and the playground URL. The page puts the example's current text into the hole (`fill` in `site.js` mirrors `Template::fill`, and a unit test keeps both equal to `generate` and `standalone`). "Open in playground" goes to `<playground-url>#code=<base64url of the program>`.

   Examples with a Run button are editable: a transparent textarea lies over the highlighted `<pre>`, and `site.js` colours the text again as you type with a small copy of `highlight.rs`'s rules (no call links once edited). The editor has no focus ring, only the caret. Tab inserts four spaces (Escape, then Tab, leaves the box), Enter keeps the indent, Ctrl/Cmd+Enter runs. Reset shows once the text differs. Run, Copy and the playground link all use the edited text. Without JavaScript the code is the plain highlighted block. On touch screens the textarea is 16px (phones zoom into smaller fields) and scaled down by `--k` to the code's size; its scroll offsets are multiplied by `--k` when copied to the `<pre>`. Moving between pages, the new content fades in; the old content disappears at once, so its title never shows over the new one.
5. **Search.** `search-index.json` loads on the first keystroke. A query matches by name; one with `->` matches overload types (`string -> [] string`).

**Running the theme.** The theme's files are embedded in `jacket` (`crates/jacket/build.rs`). On first use `jacket doc` writes them to `<user cache>/jacket/doc-site/<key>/` and compiles them with `jaic -o`; the key hashes the jacket and jaic versions, the theme files, any overrides and `stdlib/Extensions/Web`, so later runs reuse the binary. The user cache is `~/Library/Caches` on macOS, `$XDG_CACHE_HOME` or `~/.cache` on Linux, `%LOCALAPPDATA%` on Windows. A `jaic` without the native backend runs the theme with `-interp` instead, with a warning: it is much slower. The view goes to a temporary folder that is deleted afterwards; `--emit-input <dir>` keeps it. Rust still writes `search-index.json`, `examples/`, `versions.json`, `runtime/` and the logo. The stdlib site builds in about 10 s (`--timings` shows the phases).

Pages are linked without a trailing slash: `/api/Basic/print` is the file `api/Basic/print.html`, and the front page is `/` (`index.html`). GitHub Pages, Netlify and Cloudflare Pages look a path up as `<path>.html`; a server that does not needs that rule. A module and its items share a name (`api/Tls.html`, `api/Tls/Thing.html`). An old link with a trailing slash lands on `404.html`, which `site.js` sends to the slashless address. Characters that can't be in a file name become `-` plus a hex code (`operator-3c`). Names that differ only in case (`Head`, `head`) would share a file on macOS and Windows, so the lowercase one keeps its name and the others add their kind (`Head-struct`); the search index carries such a name as an extra column (`case_folders` in `site.rs`). `--check` fails on a doc-comment link written like an intra-doc link (`[Name]`, `[Module.item]`, `` [`name`] ``) that names no item of the library or of another module, printed as `file:line: [Name] names nothing` (`docs::unresolved::unresolved`, listed as `unresolved` in the `jaic -doc_model` JSON). Prose such as `[x]`, `[TODO]` and a plain lowercase word is left alone, and a bare name resolves only within the library, so link another module's item as `[Basic.print]`. `--allow-broken-links` turns these into warnings. It also fails on internal links ending in `.html` or `/`, links to files that weren't written, types with no page to link to, and an item and module of the same name.

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

Modules are the folders of the library with a `module.jai`, minus `internal/` and `tests/`. `prelude/` is a module page too; since every program imports it, the page says so instead of showing an `#import` line (`implicit` in the module JSON). When the path is itself a module, its id is the folder name and the modules inside it are `name/sub`. A library's own folder is passed to `jaic` as `-import_dir`, so its modules can import each other.

**Module parameters.** Each `#module_parameters` of the module's files becomes a "Module parameters" section before the items: name, type, default, and the doc comment above or after the parameter. Names in the type and the default link to what they resolve to, as procedure parameters do. Put one parameter per line so each has its own comment. They are not counted by `jaic -doc_gaps`. Declarations in a `#module_parameters` body (an enum a parameter's type needs) are ordinary items of the module: they get pages, links and search entries, and `#scope_file`/`#scope_module` inside the body hides them as at the top level.

**Native bindings.** A file with a top-level `#foreign` procedure mirrors a native API (files under `internal/` don't count, and `#intrinsic` or `#compiler` procedures never make a file a binding). The model marks its declarations `"binding": true`; the module page lists them in the usual Procedures, Types and Constants sections after the module's own items, with a "Foreign" badge, and each gets an item page like any other. A procedure with a body in such a file is not a binding: it gets a page like any other. `jaic -doc_gaps` skips every file with a bodiless top-level procedure, intrinsics included. Search includes bindings with a "Foreign" badge and ranks them below the library's own API. Their `since` label is left off.

**Module rows.** A struct, union or enum row shows its fields or members inline, folded with Show more past 200px. Opening is one-way: the button goes away and the block stays open until the page reloads. A constant that instantiates one of the module's own structs (`u8x16 :: Vector(u8, 16)`) is listed as a type. The "On this page" rail is drawn like the left sidebar (the `.toc` block in `site.css`). Each heading's item list folds: `site.js` opens every list with a section on screen and colours every item on screen (headings stay plain), and closes the rest as you scroll. A list the reader opens with its chevron stays open; one they close reopens once they've scrolled away and back. Without JS every list is open. The rail glides to keep the marked entries in its middle.

Page chrome ids (`search-q`, `search-results`, `theme-toggle`, `ver-button`, `main-content`) contain a `-` so they can never equal an item's anchor, which is its Jai name. Same-page links scroll smoothly unless the reader prefers reduced motion.

`runtime.json` (`{"engine": "...", "wasm": "..."}`) points at the bundle `tools/browser/build_scripting_wasm.py --release --output <dir>` writes ([browser build](../browser/playground.md)); `--runtime <dir>` copies `jai_wasm.wasm` and `engine.mjs` from it. A site that already hosts the compiler can drop the copy and edit `runtime.json`.

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
theme = "doc-theme"                 # files that replace or extend the built-in theme

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

A pattern matching a module's folder also matches the modules inside it. `jacket doc` fails on a pattern that matches no module, a module in no group (when there are groups), and a colour that is not `#hex` or a plain name. The header shows the version, repository button, playground link and pages link only when set.

The standard library's `[doc]` sits in the root `jacket.toml`: categories "Standard library" (`stdlib/**`) and "Extensions" (`stdlib/Extensions/**`, teal "extension" badge), the topic groups, and the "Tools", "Lint rules" and "Format rules" sections (generated into `docs/reference/` and `docs/tools/jaifmt-rules.md`). Add new stdlib modules to a group there.

**Sections.** A `[[doc.section]]` file is split at its headings: the text before the first `##` (plus each `##`'s own text and a list of its pages) is the overview at `<name as a slug>/`, each `##` is a folding group in the sidebar, and each `###` is a page at `<section>/<heading slug>/` titled with the heading. A `###` before any `##` goes in a group with no label. Inside a page, `####` and `#####` headings move up to `##` and `###`, so they reach its outline. The header link (`pages_title`) goes to the first `[[doc.page]]`, else to the overview of the section with that name. A link to `#heading` or `file.md#heading` of a split file goes to that heading's page. A `##` with no `###` under it (the Format rules' "Indentation") stays text on the overview, so `format-rules#indentation` is still an anchor there.

**Home.** `[doc] home` names a Markdown file rendered as the front page, with the module sidebar. Links in it (and in every extra page) to a module's folder or `module.jai` go to that module's page, so `[Basic](../../stdlib/Basic)` works both on GitHub and on the site. Without it the front page shows the name and description; the stdlib's is `docs/reference/home.md`. The extra pages and sections come after the module categories in every page's sidebar (`view/guide.rs`, `Site::extra_nav`).

**Before and after.** Two fences in a row, ```` ```jai before ```` then ```` ```jai after ````, render as one block split by a line ("Before", "After"), stacked under 700px. Words after `after` are shown as the settings used (`indent_width=2` reads "After, with `indent_width = 2`"). Jai fences on these pages are highlighted and scroll sideways; other fences are plain and wrap long lines (usage lines, shell output).

## Page metadata

Every page gets a title made from `[doc] name` and `breadcrumb` (`String.trim — Jacket std`, `String — Jacket std`; the index is the name alone), a description from the summary (plain text, at most 155 characters), `lang`, `theme-color` for both schemes, a favicon, Open Graph and Twitter tags. Item pages carry `TechArticle` JSON-LD, and pages with a breadcrumb a `BreadcrumbList`. Without `[doc] url` there is no canonical, `og:url`, sitemap or `robots.txt`, and `jacket doc` warns once. A page with an empty description gets no description tags, and `404.html` has no canonical or `og:url`.

Rust works out these facts (`metadata.rs`, into each page's `head` in the view) and `HeadMeta` in `shell.jai` writes the tags, leaving out any that are empty.

Versions: the current release lives at `<url><base>`, older ones at `<url><base>versions/<version>/` (built with that as `--base-url`). With `canonical_version` set and different from `--version`, each page's canonical points at the same path under that version, and the page is `noindex`.

## Accessibility

The site targets WCAG 2.2 AA:

- **Structure:** landmarks, a skip link, one `h1` per page, and real lists and buttons.
- **Keyboard:**
  - Search is a combobox (arrows, Enter, Escape).
  - The version menu is a menu button.
  - Folds and nav categories set `aria-expanded`.
  - Focus moving into a folded block opens it.
  - A wide code block gets a tab stop so it can be scrolled.
- **Motion and zoom:** smooth scrolling only without reduced motion; layouts work at 320px.
- **Contrast:**
  - `tools/dev/test_doc_palette.py` checks every colour token in `site.css` in both themes: 4.5:1 for text, 3:1 for the search field's border.
  - `jacket doc` rejects a `[doc.category]` `color`/`color_dark` under 4.5:1 and prints the measured ratio.
- **CI:** `tools/dev/doc_a11y.mjs <url>` runs axe-core on six pages, in both themes, at 1280px and 320px. It needs Chrome (`CHROME_PATH` to override); `docs-pages.yml` runs it after the build.
- **Layout shift:** `tools/dev/doc_layout.mjs <url>` loads a sample of page kinds (home, guide, the one-page rules pages, small and big modules, an item, a private type, the 404) at 1280px and 375px in both themes, and fails on any `layout-shift` entry or any key box (titles, platforms, outline, folds and their buttons, header, sidebars, scroll positions) that differs between a frame and where the page settles. Scenarios are a fresh load, a reload after scrolling, leave and Back, and in-page swaps by a sidebar link and the header's API link (each also on its own page, then Back and Forward). It holds `site.js` back 250ms (`--script-delay`) so a frame paints without it, as on a slow connection; with `--script-delay 0` the local server hides everything `site.js` changes. A page-kind sample takes about two minutes; `--all` is about 32,000 pages at about 3.5s per page, width and theme, so use `--all --every 600` (about 14 minutes) for a spread. Serve the site with `tools/dev/doc_serve.py`; an old trailing-slash URL is also loaded to check the 404 page's redirect. Anything that must be right at first paint belongs in `early.js` or the markup, not `site.js`. Footer-only shifts are ignored for swaps, since a taller page moves it.
- **First-paint rules** the check enforces: the outline's column is held open by `.wrap[data-toc]::after` until the outline is parsed (the template sets `data-toc` when the page has one, and `swap()` copies it); `early.js` closes the menu on a phone and drops the buttons of folds that fit, since `.members:last-child` rules change padding when a hidden button goes; `jacketRestore()` (after the footer) finishes a scroll restore that the unparsed footer cut short.

## Publishing

The stdlib site is served by GitHub Pages at https://jacket.matteopolak.com. `.github/workflows/docs-pages.yml` builds it with `jacket doc --base-url / --version <version> --source-ref <tag> --playground-url <playground> --runtime <bundle>`:

- A release calls it from its `docs` job, which also attaches `jacket-docs-<version>.tar.gz`. The release's `pages` job deploys the site once the release is published, for the current release only.
- A manual run builds a tag or branch and deploys it (default: the latest release tag):

```sh
gh workflow run docs-pages.yml -R matteopolak/jacket -f ref=v0.33.0
```

Repository settings: Pages source "GitHub Actions", custom domain `jacket.matteopolak.com` (a `CNAME` record to `matteopolak.github.io`), and the `github-pages` environment allows `main` and `v*` tags. The Pages limit is 1 GB per site, with no file-count cap.

CI runs `jacket doc --check` when the stdlib, the compiler or the docs change, so a broken intra-doc link fails before a release does.

## How to change it

- **What a page says** comes from the doc comment ([doc comments](../compiler/doc-comments.md)). Document parameters with `- name: text` or a comment on the parameter.
- **A fact the compiler should supply:** `model.rs` in `doc_comments/` and the JSON it prints, then `doc/model.rs`.
- **Layout and colours:** `doc/site/assets/site.css` (colours are custom properties at the top, with a dark set; run `test_doc_palette.py` after changing one). Behaviour: `doc/site/assets/site.js`, which only adds to pages that read fine without it.
- **Markup** is the Jai theme: `shell.jai` (head, header, footer, the layout), `sidebars.jai`, `index.jai`, `module.jai`, `item.jai`, `guides.jai`, `examples.jai`, `parts.jai` (badges, crumbs, Source links). A new value on a page goes in `view.rs`, `view/build.rs` and `contract.jai` together. Markup tests are in `site/tests/suite.jai` with JSON fixtures; `cargo test -p jacket doc::` runs them (they need a built `jaic` beside `jacket`, and are skipped otherwise), and so does `jacket test --path crates/jacket/src/doc/site`.
- **Your own theme:** `[doc] theme = "folder"` (or `--theme`). Each file there replaces the built-in file at the same path, and new files are added (hidden files such as `.DS_Store` are skipped), so overriding `assets/site.css` or `item.jai` is enough. A compile error in an override is printed as jaic reports it. `jacket doc --emit-input in/` writes the view, so you can see what a template gets. Compare two builds with `python3 tools/dev/doc_diff.py OLD NEW`.
- **Module page grouping** is by declaring file (`bytes.jai` is "Bytes"). When procedures are split into groups, foreign bindings get one "Foreign" group of their own instead of their file's, so each item is listed once (page ids must be unique). Words with their own casing go in `WORDS` in `model.rs`.
- **Source links** need `repo`. They go to the commit `jacket doc` was built from when the library is in a git checkout (a file with uncommitted changes gets none); `--source-ref` overrides. Outside a checkout and without `--source-ref` there are no Source links.
- **Versions menu:** `versions.json` lists one version; a site hosting several replaces it with one entry each.
- **"Since" versions.** `since 0.24.0` labels come from the file `[doc] since` names, and only when `--version` is given; the stdlib's is `crates/jacket/src/doc/since.tsv`. `tools/generators/generate_doc_since.py` writes it from the release tags; `check_generated.py` only validates it. An item with no row is new in the version being built. After a release is tagged, run `python3 tools/generators/generate_doc_since.py` and commit the file (it adds only new tags; `--full` rereads all).
- **Indented code** in a doc comment is a plain code block and never run; `jacket doc` warns for each. Use ```` ```jai ````; `jai,ignore` for fragments that can't stand alone, `jai,no_run` for complete ones needing a file or network.
- **Platforms.** A module page lists every system under its title as a chip: green when everything works there, amber when some items are not available there (the tip says how many), red when the module does not build there. A module is available on the systems it asserts (`#assert OS == ...`), else macOS, Linux and Windows, plus Wasm when it builds for `-os wasm`; Android and iOS are listed only for modules that name them (`Module::support` in `doc/model.rs`, `platforms` in `view/build.rs`, `.pf` in `site.css`). An item row gets a small red chip for each system its module runs on but it does not, so a system the whole module lacks is not repeated on every row (`limits`/`holds_on` in `model.rs`, `Module::item_missing` in `doc/model.rs`). An item page lists the same systems as its module page, green or red for that item alone (`Module::item_support`), since a reader landing there has not seen the module page.
- **Foreign rows.** `ForeignBadge` in `parts.jai`; the search flag is bit 2 of an index row's last column (`search.rs`, `site.js`). Every row on a module page also has its own Source link (`ItemRow` in `module.jai`).
- **Experimental and category badges.** A doc comment line that is exactly `@experimental` adds an amber badge (`ExperimentalBadge` in `parts.jai`); every item of an experimental module inherits it (`Site::new`). A category with a `badge` puts it on its modules and their items (`CategoryBadge`), coloured by `assets/categories.css` (`CategoryStyle` in `shell.jai`, classes `.cat-N`). Sidebars and search show these as coloured dots with a hover title and hidden text (`.dot` in `site.css`). In the search index, a row's flags hold one more than the badge's category index above the low two bits, and `cats` lists each category's badge and tip.
- **Small text.** "N overloads", "since X" and "<systems> only" are plain small text (`.since`), joined by a `·`.
- **Item pages.** The signature (or a constant's value) comes first, then the description. A procedure's doc comment belongs to its overload: the model leaves the item's own `docs` empty, so the text shows once, with `- name: ...` lines moved into the parameter table. Only an alias (`twice :: scale;`) keeps a comment of its own, shown as the page's lead.
- **Links.** Links are underlined only on hover, including type and name links in code.
- **Overloads.** Module rows list every overload's signature. On an item page with several overloads, each is a card headed "Overload N", with the signature, parameter and result tables and examples as full-width bands.
- **Linked types.** Signatures, fields, and the values of type aliases and constants link each type to its page (`Item.refs` from the model, `Site::type_html`). The model resolves a name through every alias hop in order and links the nearest one with a page, so `background: Color` goes to `Color`, not to the `Vector4` it is an alias of (`model_resolution.rs` in `doc_comments/`). Resolution follows `using` imports, loaded files and module parameters, and ignores names inside strings.
- **Alias pages.** A constant whose value is a type is kind "type", listed under Types, and its page shows the alias as written (`Color :: Vector4;`) rather than the target's fields. A procedure type is a type too, not a procedure.
- **Declarations.** A struct, union or enum page opens with its declaration as written, comments removed (`declaration` and `decl_refs` in the `jaic -doc_model` JSON, `highlight::declaration` for the links). Type names link to their pages; declared names never do. The Fields table below lists only fields with doc comments, and a module page row shows the declaration in place of the field list. A `[Type.field]` link goes to the field's row when it has docs, else to `#declaration`.
- **Private types.** A `#scope_module` or `#scope_file` struct, union or enum that a public item names (directly or through other private types' fields) gets a page marked "Not exported" with `noindex`, so links to it work. It is left out of the module page, sidebar, search and sitemap (`Module.private` in `doc/model.rs`). A private type whose name clashes with a public item is skipped.
- **Missing type links.** `jacket doc --check` (and every build) fails when a type in a signature, field or type value has no page to link to (`type_links.rs`). Exempt: language builtins and prelude names, polymorphic and module parameters, and private declarations with no documented re-export. There is no list of allowed names, so fix a failure by documenting or re-exporting the type.
- **Item sidebar.** An item page's sidebar lists the other items in its group. The page itself holds up to five of them, around the current one, and `early.js` (inlined after the sidebar, so it runs before the first paint) fills in the rest from `assets/nav/<module path>/<group>.html`. Every item page in that group shares this file, which is stamped with `?v=` like the other assets. Without JS the five links and the back link are what you get. With JS, the list's padding (`--before`/`--after`, 30.5px a row in `site.css`) reserves the missing rows until they arrive, so the sidebar never changes height. Change that constant if the row's font, padding or gap changes. This keeps pages about a third smaller (`ItemNavList` and `WriteItemNav` in `sidebars.jai`).
- **Moving between pages.** `site.js` fetches a page inside the site when the pointer rests on its link (65 ms) or it gets focus, and a click swaps in its `<main>`, outline, sidebar, title and description without a page load. The header stays; the sidebar keeps its open groups and scroll, and a long group already loaded in full is moved across instead of shrinking to five rows. Back and Forward restore the scroll of each page (`history.state.y`, with `scrollRestoration = 'manual'`). Clicking the page that is open scrolls to its top. Per-page setup (the outline rail, editors, folds, scrollable code) is `setupPage` and runs again after each swap; listeners on `window` take its `AbortSignal`. Links with a modifier key, a `target`, or outside `--base-url` load normally, and so does any page that fails to fetch.
- **Long blocks.** A `.fold` taller than 440px (module descriptions) shows its top with a faded edge and a "Show more" button; `site.js` adds the button and removes it once the block is expanded. The first six folds are measured by an inline `fold()` call while the page loads (`EARLY` in `shell.jai`), so the top of the page never flashes. `site.js` measures the rest in one pass, since measuring each fold as it arrives lays the page out once per fold.
- **Scrolling big pages.** The SDL3 page has about 78k elements and 4k rail entries, so a layout after a scroll costs ~10 ms there. The rail's scroll handler binary-searches the headings for the visible range and touches only entries that changed. It reads positions before writing anything. A marked entry is bold through a bold copy of its name in `::after` (the `data-t` attribute), revealed by swapping colours, so marking never triggers a layout. Keep it that way: a `font-weight`, `text-shadow` or `content-visibility` change on rail entries brings the cost back.

## Configuration

| Flag | Default | |
| --- | --- | --- |
| `--out` | `target/doc-site` | replaced each run, only if `jacket doc` made it |
| `--base-url` | `/` | where the site is served, with both slashes |
| `--runtime` | none | folder with `jai_wasm.wasm` and `engine.mjs` |
| `PATH` | `[doc] path`, else the stdlib | the library to document |
| `--name`, `--logo`, `--repo`, `--playground-url` | the `[doc]` setting, else none | override `[doc]` |
| `--version` | none | the header's version; also turns on `since` labels |
| `--source-ref` | the checkout's commit | tag or branch for Source links |
| `--module` | all | build only these; broken links are then warnings |
| `--check` | off | build and check, write nothing |
| `--allow-broken-links` | off | warn instead of failing |
| `--site-url`, `--og-image` | `[doc] url`, `og_image` | override `[doc]` |
| `--theme` | `[doc] theme` | theme override folder, relative to the current folder |
| `--emit-input` | off | also write the view JSON to this folder |
| `--timings` | off | print each phase's time to stderr |

`JAIC_STDLIB` picks the standard library, as for `jacket test`; the default is the `stdlib/` of the project you run it in. The `[doc]` keys are listed above.

## Dependencies

`jaic` (`-doc_model`, and compiling the theme), `stdlib/Extensions/Web` and `Extensions/Json` (the theme), `jacket-config` (the `[doc]` schema and path patterns), `pulldown-cmark`, `serde_json`, `sha2`, and `crates/jacket/src/test/doctest.rs`. The Run button needs module workers and WebAssembly; the compiler's wasm build is `crates/jai-wasm`.
