# jacket doc

`jacket doc` writes the API reference for a Jai library as a static website: one page per module and per item, a search index, and any extra Markdown pages the project lists. The standard library's site is one such library, set up in the root `jacket.toml`. Every doc-comment example gets a Run button that compiles and runs it in the browser with the playground's compiler build. Nothing it makes is committed; a release builds it and attaches `jacket-docs-<version>.tar.gz`.

```sh
jacket doc                                  # the [doc] library of the nearest jacket.toml, else the standard library
jacket doc path/to/my_lib                   # a module (folder with module.jai) or a folder of modules
jacket doc --base-url /docs/jacket/ --out site
jacket doc --module String --module Basic   # just these, for a quick look
jacket doc --check                          # build in memory and check links only
```

Flags are in the [command-line reference](../reference/command-line.md). Without `--runtime` the Run buttons are disabled and say why. Serve the output with any static server (`python3 -m http.server --directory target/doc-site`); links are absolute from `--base-url`, so opening files from disk won't style.

## How it works

1. **Facts from the compiler.** `jaic -doc_model <dir>` (`crates/jaic-syntax/src/doc_comments/model.rs`) reads a library with the language server's parser and prints a JSON model: public constants, aliases followed to their targets, overloads, parameter and result types, doc comments with links resolved, fields and members, platform limits, and for each type name the declaration the compiler resolved it to. Docs are found by the same rules as hover.
2. **Pages.** `crates/jacket/src/doc/` turns the model into HTML: `pages.rs` (index, module and item pages), `categories.rs`, `links.rs` (link checking), `site.rs` (where pages live, signatures), `markdown.rs`, `highlight.rs`, `guide.rs` (the `[[doc.page]]` Markdown pages), `search.rs`, `shell.rs`. A type name links to the item the compiler resolved it to, never by matching the name.
3. **Examples.** A ```` ```jai ```` fence is read by the function `jacket test` uses (`test::doctest::fenced`), so options like `ignore`, `no_run` and `skip_on(wasm)` mean the same. The Run button fetches `examples/<id>.json` and runs the program in a worker with a step budget. That file holds the program `jacket test` builds as the text around the example (`head`, `middle`, `tail`, from `doctest::generate_template`), plus the same for the playground program (`open`) and the playground URL. The page puts the example's current text into the hole (`fill` in `site.js` mirrors `Template::fill`, and a unit test keeps both equal to `generate` and `standalone`). "Open in playground" goes to `<playground-url>#code=<base64url of the program>`.

   Examples with a Run button are editable: a transparent textarea lies over the highlighted `<pre>`, and `site.js` colours the text again as you type with a small copy of `highlight.rs`'s rules (no call links once edited). Tab inserts four spaces (Escape, then Tab, leaves the box), Enter keeps the indent, Ctrl/Cmd+Enter runs. Reset shows once the text differs. Run, Copy and the playground link all use the edited text. Without JavaScript the code is the plain highlighted block.
4. **Search.** `search-index.json` loads on the first keystroke. A query matches by name; one with `->` matches overload types (`string -> [] string`).

Every page is `<path>/index.html` and every link ends in `/`, so a plain static server needs no rewrite rules. Characters that can't be in a file name become `-` plus a hex code (`operator-3c`). Names that differ only in case (`Head`, `head`) would share a folder on macOS and Windows, so the lowercase one keeps its name and the others add their kind (`Head-struct`); the search index carries such a folder as an extra column (`case_folders` in `site.rs`). `--check` fails on internal links ending in `.html`, links to files that weren't written, and an item and module sharing a folder.

| Path | What |
| --- | --- |
| `index.html` | modules, by category |
| `api/<Module>/index.html`, `api/<Module>/<name>/index.html` | a module; an item |
| `<pages_title>/<page>/index.html` | the `[[doc.page]]` pages (`tools/` for the stdlib) |
| `examples/<id>.json` | the parts of each example's program, for Run and the playground link |
| `runtime/`, `runtime.json` | the browser compiler (`--runtime`) and where pages look for it |
| `assets/` | `site.css`, `site.js`, `run-worker.js`, the logo |

Modules are the folders of the library with a `module.jai`, minus `internal/`, `prelude/` and `tests/`. When the path is itself a module, its id is the folder name and the modules inside it are `name/sub`. A library's own folder is passed to `jaic` as `-import_dir`, so its modules can import each other.

**Module parameters.** Each `#module_parameters` of the module's files becomes a "Module parameters" section before the items: name, type, default, and the doc comment above or after the parameter. Names in the type and the default link to what they resolve to, as procedure parameters do. Put one parameter per line so each has its own comment. They are not counted by `jaic -doc_gaps`.

**Native bindings.** A file with a top-level `#foreign` procedure mirrors a native API (files under `internal/` don't count, and `#intrinsic` or `#compiler` procedures never make a file a binding). The model marks its declarations `"binding": true`; the module page lists them in the usual Procedures, Types and Constants sections after the module's own items, with a "Foreign" badge, and they have no item pages. A procedure with a body in such a file is not a binding: it gets a page like any other. `jaic -doc_gaps` skips every file with a bodiless top-level procedure, intrinsics included. Search includes bindings with a "Foreign" badge and ranks them below the library's own API. Their `since` label is left off.

**Module rows.** A struct, union or enum row shows its fields or members inline, folded with Show more past 200px. A constant that instantiates one of the module's own structs (`u8x16 :: Vector(u8, 16)`) is listed as a type. The "On this page" rail nests each section's items: a list shows its first five, or a window around the item being read, and Show all keeps it open. Same-page links scroll smoothly unless the reader prefers reduced motion.

`runtime.json` (`{"engine": "...", "wasm": "..."}`) points at the bundle `tools/browser/build_scripting_wasm.py --release --output <dir>` writes ([browser build](../browser/playground.md)); `--runtime <dir>` copies `jai_wasm.wasm` and `engine.mjs` from it. A site that already hosts the compiler can drop the copy and edit `runtime.json`.

The hosted docs redirect `/docs/jacket/playground` to the playground, so the site never writes a `playground/` path or `playground.html`.

## Settings: `[doc]` in `jacket.toml`

`jacket doc` with no path documents the `[doc]` library of the nearest `jacket.toml` (from the current folder), else the bundled standard library with no settings. With a path, the nearest `jacket.toml` above it applies only if its `[doc]` is for that folder; otherwise the site uses defaults. Paths and patterns are relative to the `jacket.toml`. Flags win over settings.

```toml
[doc]
path = "src/my_lib"                 # default: the jacket.toml's folder
name = "my_lib"                     # header and titles; default: the folder name
breadcrumb = "my_lib"               # first crumb on every page; default: the folder name
description = "API docs for my_lib."
logo = "assets/logo.svg"            # .svg or .png; default: none
repo = "https://github.com/me/my_lib"   # Source links need it
playground = "https://example.com/play"  # default: none, and no playground links
since = "notes/since.tsv"           # "since X" labels; used only with --version
pages_title = "Guides"              # heading and folder of the extra pages

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

[[doc.page]]
file = "notes/guide.md"
name = "Guide"
summary = "Getting started."
```

A pattern matching a module's folder also matches the modules inside it. `jacket doc` fails on a pattern that matches no module, a module in no group (when there are groups), and a colour that is not `#hex` or a plain name. The header shows the version, repository button, playground link and pages link only when set.

The standard library's `[doc]` sits in the root `jacket.toml`: categories "Standard library" (`stdlib/**`) and "Extensions" (`stdlib/Extensions/**`, teal "extension" badge), the topic groups, and the Tools pages from `docs/reference/`. Add new stdlib modules to a group there.

## Publishing

The release workflow's `docs` job runs `jacket doc --base-url /docs/jacket/ --version <version> --source-ref v<version> --playground-url <playground> --runtime <bundle>` and attaches `jacket-docs-<version>.tar.gz`. The portfolio site's build downloads the asset for the tag it pins and extracts it into `/docs/jacket`:

```sh
gh release download "$TAG" --repo matteopolak/jacket --pattern 'jacket-docs-*.tar.gz' --dir "$TMP"
mkdir -p dist/docs/jacket && tar -xzf "$TMP"/jacket-docs-*.tar.gz -C dist/docs/jacket
```

CI runs `jacket doc --check` when the stdlib, the compiler or the docs change, so a broken intra-doc link fails before a release does.

## How to change it

- **What a page says** comes from the doc comment ([doc comments](../compiler/doc-comments.md)). Document parameters with `- name: text` or a comment on the parameter.
- **A fact the compiler should supply:** `model.rs` in `doc_comments/` and the JSON it prints, then `doc/model.rs`.
- **Layout and colours:** `doc/assets/site.css` (colours are custom properties at the top, with a dark set). Behaviour: `doc/assets/site.js`, which only adds to pages that read fine without it. Markup: `pages.rs`, `example.rs`, `shell.rs`, `guide.rs`.
- **Module page grouping** is by declaring file (`bytes.jai` is "Bytes"). Words with their own casing go in `WORDS` in `model.rs`.
- **Source links** need `repo`. They go to the commit `jacket doc` was built from when the library is in a git checkout (a file with uncommitted changes gets none); `--source-ref` overrides. Outside a checkout and without `--source-ref` there are no Source links.
- **Versions menu:** `versions.json` lists one version; a site hosting several replaces it with one entry each.
- **"Since" versions.** `since 0.24.0` labels come from the file `[doc] since` names, and only when `--version` is given; the stdlib's is `crates/jacket/src/doc/since.tsv`. `tools/generators/generate_doc_since.py` writes it from the release tags; `check_generated.py` only validates it. An item with no row is new in the version being built. After a release is tagged, run `python3 tools/generators/generate_doc_since.py` and commit the file (it adds only new tags; `--full` rereads all).
- **Indented code** in a doc comment is a plain code block and never run; `jacket doc` warns for each. Use ```` ```jai ````; `jai,ignore` for fragments that can't stand alone, `jai,no_run` for complete ones needing a file or network.
- **Platforms.** A module page has an "Available on macOS, Linux, Windows" line under its title, listing the systems it asserts (`#assert OS == ...`), else macOS, Linux and Windows, plus Wasm when it builds for `-os wasm` (`platforms_line` in `pages.rs`). An item shows "<systems> only" as small text only when an `#if OS` makes it available on fewer systems than its module (`limits`/`holds_on` in `model.rs`, `Module::item_systems` in `doc/model.rs`).
- **Foreign rows.** `foreign_row` in `pages.rs`; the search flag is bit 2 of an index row's last column (`search.rs`, `site.js`). Links to a binding item go to `api/<Module>/#<name>` (`Site::item_href`), and the outline (`outline_html`) links to rows by the same id.
- **Experimental and category badges.** A doc comment line that is exactly `@experimental` adds an amber badge (`experimental_badge` in `pages.rs`); every item of an experimental module inherits it (`Site::new`). A category with a `badge` puts it on its modules and their items (`category_badge`), coloured by a small `<style>` each page carries (`category_style` in `shell.rs`, classes `.cat-N`). Sidebars and search show these as coloured dots with a hover title (`experimental_dot`, `.dot` in `site.css`). In the search index, a row's flags hold one more than the badge's category index above the low two bits, and `cats` lists each category's badge and tip.
- **Small text.** "N overloads", "since X" and "<systems> only" are plain small text (`.since`), joined by a `·`.
- **Overloads.** Module rows list every overload's signature. On an item page with several overloads, each is a card headed "Overload N", with the signature, parameter and result tables and examples as full-width bands.
- **Linked types.** Signatures, fields, and the values of type aliases and constants link each type to its page (`Item.refs` from the model, `Site::type_html`).
- **Long blocks.** A `.fold` taller than 440px (module descriptions) shows its top with a faded edge and a "Show more" button; `site.js` adds the button and removes it once the block is expanded.

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

`JAIC_STDLIB` picks the standard library, as for `jacket test`; the default is the `stdlib/` of the project you run it in. The `[doc]` keys are listed above.

## Dependencies

`jaic` (`-doc_model`), `jacket-config` (the `[doc]` schema and path patterns), `pulldown-cmark`, `sha2`, and `crates/jacket/src/test/doctest.rs`. The Run button needs module workers and WebAssembly; the compiler's wasm build is `crates/jai-wasm`.
