# jacket doc

## What it is

`jacket doc` writes the API reference for the standard library as a static website: one page per module and per procedure, struct, enum or constant, a search index, and the Tools pages (the files in `docs/reference/`). Every example in a doc comment is shown with a Run button that compiles and runs it in the browser, using the same compiler build the playground uses. Nothing it makes is committed; a release builds it and attaches it as `jacket-docs-<version>.tar.gz`.

```sh
jacket doc                                  # target/doc-site/, served from /
jacket doc --base-url /docs/jai/ --out site # for a site that lives under /docs/jai/
jacket doc --module String --module Basic   # just these, for a quick look
jacket doc --check                          # build in memory and check the links only
```

The flags are in the [command-line reference](../reference/command-line.md); `jacket doc --help` lists them too. Without `--runtime` the Run buttons are disabled and say why, and everything else works.

To look at a build, serve the folder with any static server (`python3 -m http.server --directory target/doc-site`). The links are absolute from `--base-url`, so a file opened straight from disk will not find its style.

## How it works

1. **Facts from the compiler.** `jaic -doc_model <dir>` (`crates/jaic/src/doc_comments/model.rs`) reads one library with the parser the language server uses and prints a JSON model: the public constants of the module, aliases followed to what they name (`split :: Impl.split;`), procedures declared under one name collected as overloads, parameters and results with their types, doc comments with intra-doc links resolved, struct fields and enum members, the platforms the module asserts (`#assert OS == ...`), and the systems an `#if OS == ...` around an item limits it to. Docs are found by the same rules as hover, so what the editor shows is what the site shows. Intra-doc links come out as `jaidoc:Module/item`.
2. **Pages.** `crates/jacket/src/doc/` turns the model into HTML. `model.rs` reads the JSON; `pages.rs` has the index, module and item pages; `categories.rs` reads `stdlib/categories.toml`; `links.rs` checks the written links; `site.rs` knows where pages live and builds signatures with links to the types they mention; `markdown.rs` renders doc comments with `pulldown-cmark` and hands back fenced blocks as separate pieces; `highlight.rs` colours Jai; `guide.rs` renders `docs/reference/*.md`; `search.rs` writes the index; `shell.rs` is the header and footer.
3. **Examples.** A ```` ```jai ```` fence is read with the same function `jacket test` uses (`test::doctest::fenced`), so the options on it (`ignore`, `no_run`, `compile_fail`, `skip_on(wasm)`, `only(...)`) mean the same thing. The Run button fetches `examples/<id>.jai`, which is the program `jacket test` builds for that example (`test::doctest::generate`), runs it in a worker with the compiler's wasm build and shows the output. A failing `assert` shows its message and call stack. An example `jacket test` would skip on wasm is shown without a working button and with a line saying so. "Open in playground" uses a plainer program: the example as a `main` with its imports.
4. **Search.** `search-index.json` is fetched on the first keystroke. A query is matched by name; a query with `->` is matched by the types of each overload (`-> bool`, `string -> [] string`, `string, u8 -> bool`).

### Pages and files

| Path | What |
| --- | --- |
| `index.html` | the module list, by category |
| `api/<Module>/index.html` | a module: docs, procedures grouped by the file that declares them, types, constants |
| `api/<Module>/<name>/index.html` | an item |
| `tools/<page>/index.html` | the reference pages |
| `examples/<id>.jai` | the programs the Run buttons run; the id is a digest of the text |
| `runtime/`, `runtime.json` | the browser compiler (`--runtime`) and where the page looks for it |
| `assets/` | `site.css`, `site.js`, `run-worker.js`, the logo |

Every page is an `index.html` in its own folder and every link ends in `/`, so the addresses have no `.html` and a plain static server (`python3 -m http.server`) serves them with no rewrite rules. An item whose name has characters that can't be in a file name (`operator<`) gets a folder where each such character is `-` and its code in hex (`operator-3c`); `Site::slug` and `hrefOf` in `site.js` do the same. `--check` fails if any internal link ends in `.html` or leads to a file that was not written, and if an item and a module would share a folder.

Modules are the folders of `stdlib/` with a `module.jai`, minus `internal/`, `prelude/` and `tests/`; `Extensions/Http` is the page `api/Extensions/Http/`. A library whose files declare foreign procedures (a binding) lists nothing, as in the [doc gaps](stdlib-doc-gaps.md) report.

### Running examples

The page loads `runtime.json` (`{"engine": "...", "wasm": "..."}`, relative to the base path or absolute) and starts `assets/run-worker.js` as a module worker, which imports `engine.mjs`, compiles `jai_wasm.wasm` once and calls `play` with a step budget, so an endless loop is stopped. This is the bundle `tools/browser/build_scripting_wasm.py --release --output <dir>` writes (see [browser build](../browser/playground.md)); `--runtime <dir>` copies `jai_wasm.wasm` and `engine.mjs` out of it. A site that already hosts the browser compiler can leave the copy out and edit `runtime.json` to point at its own.

### The playground link

"Open in playground" goes to `<playground-url>#code=<base64url of the program>` (UTF-8, URL-safe alphabet, no padding). The hosted playground is in another repository and has to read that fragment to show the code; until it does, the link just opens the playground. A fragment that is not `code=...` is a file name, as before.

### Categories

`stdlib/categories.toml` puts every module in one category (`[[category]]` with a `name` and a `modules` list, in the order the categories are shown). The sidebar folds each category (the current module's is open, and the fold animates where the browser supports `::details-content`), and the home page lists modules under their category. Extensions are listed under their own heading, split by the same categories. `jacket doc` fails when a module has no category, is listed twice, or a name matches no module, so a new module has to be added to the file.

## How the site is published

The release workflow's `docs` job builds the browser compiler and runs `jacket doc --base-url /docs/jai/ --runtime <bundle>`, then attaches `jacket-docs-<version>.tar.gz` to the GitHub release. The tarball holds the site at its top level, so extracting it gives `index.html`, `api/`, and so on.

The portfolio site serves it at `/docs/jai`. Its build downloads the asset for the Jacket tag it already pins, extracts it into `/docs/jai` of the site's output and does nothing else:

```sh
gh release download "$TAG" --repo matteopolak/jacket --pattern 'jacket-docs-*.tar.gz' --dir "$TMP"
mkdir -p dist/docs/jai && tar -xzf "$TMP"/jacket-docs-*.tar.gz -C dist/docs/jai
```

Nothing is committed to either repository. If the portfolio would rather not carry a second copy of the 23 MB browser compiler, delete `runtime/` after extracting and rewrite `runtime.json` to the URLs of the one it hosts; the pages read it at run time. To make "Open in playground" load the code, the playground needs to understand `#code=`.

CI runs `jacket doc --check` (which also checks the categories and every written link) in the Jai part of the `test` job (the one that runs when the stdlib, the compiler or the docs change), so a broken intra-doc link fails before a release does.

## How to change it

- **What a page says** comes from the doc comment, so change that (and see [doc comments](../compiler/doc-comments.md) for the conventions). Parameters and results are documented with a list item in the docs (`- name: text`) or a comment on the parameter.
- **A fact the compiler should supply** (a new kind of item, a platform note) goes in `crates/jaic/src/doc_comments/model.rs` and the JSON it prints, then `doc/model.rs`.
- **Layout and colours** are `doc/assets/site.css`; every colour is a custom property at the top, with a dark set. Behaviour is `doc/assets/site.js`, which only adds to pages that already read fine without it.
- **Markup** is in `pages.rs`, `example.rs` (the example boxes), `shell.rs` and `guide.rs`.
- Grouping on a module page is by the file that declares a procedure (`bytes.jai` is "Bytes"), because the sources carry no other grouping. Words with their own casing (`jks-generation.jai` is "JKS generation") are listed in `WORDS` in `model.rs`; add to it when a heading comes out wrong.
- Moving between pages uses cross-document view transitions (`@view-transition` in `site.css`): the main content fades, the header, sidebar and footer don't. The sidebar's scroll position and open categories are kept in `sessionStorage` and put back by a small inline script right after the sidebar.
- Source links go to the commit `jacket doc` was built from, so line numbers match what the page shows. That needs `stdlib/` to be the `stdlib/` folder of a git checkout; a file with uncommitted changes (or any run where `git` or the checkout is missing) links to the version tag instead. `--source-ref` turns this off.
- A module page has no filter: its procedure groups are plain headings (each with a link to itself), and on screens 1280px wide or more an "On this page" outline sits on the right and marks the heading being read.
- Scrollbars are thin and reserve their gutter (`scrollbar-gutter: stable`), so pages that scroll and pages that don't line up.
- A module whose procedures all come from one file has one flat list.
- The version in the header is plain text. If the host serves `versions.json` next to `index.html` (`[{"version": "0.31.0", "url": "/docs/jai/"}, ...]`) with two or more entries, it becomes a menu. `jacket doc` never writes that file; the site hosting several versions does.
- **"Since" versions.** Items and modules show `since 0.24.0` in small dim text (rows, item pages, the module page). The data is `crates/jacket/src/doc/since.tsv`, compiled into `jacket`: for each release tag it covers, the items that tag's `stdlib/` has and no earlier tag had, as `jaic -doc_model` sees them. `tools/generators/generate_doc_since.py` writes it (each new tag's `stdlib/` is extracted into a scratch folder, so the checkout is untouched; about 4 s per tag, 90 s for all 20 tags with a debug build) and `check_generated.py` runs it with `--check`, which only validates the file against the tags that exist. Reading the file is a hash lookup per item, so a build pays nothing and needs neither `git` nor tags, which is what a release job (a tagged checkout, often shallow) has.
  - Items in the oldest covered release are filed under `-` and show nothing: they are older than the data. A module shows a version only when all its items are newer than that.
  - An item with no row is new in the version `jacket` is built as (`Cargo.toml`), unless the data already covers that release, in which case it is a development build and the item shows nothing. So the 0.31.0 build labels everything added since 0.30.4 `since 0.31.0` with no step in between.
  - **After a release is tagged** run `python3 tools/generators/generate_doc_since.py` and commit the file; it adds only the new tags. The next release's build is correct as long as that happened before it (a missing release would hand its items to the next one). `--full` rereads every tag, which can move items if the compiler reads old sources differently.
  - A renamed or moved item counts as new under its new name.
- **Platform badges.** A module shows the systems it asserts (`#assert OS == .MACOS || ...`), or macOS, Linux and Windows when it asserts none, plus `wasm` when it builds for `-os wasm` and most of its items exist there. `jacket doc` finds that out by checking a one-line program that imports the module (`jaic -check -no_dce -os wasm`), once per module. An item gets small badges only when an `#if` on `OS` around it (or around what its alias names; a name declared in every branch counts for each) makes it available on fewer systems than its module. Conditions the compiler can't decide from `OS` alone (`CPU`, a constant) count as true, so they never hide a system. The code is `limits` and `holds_on` in `crates/jaic/src/doc_comments/model.rs` and `Module::item_systems` in `doc/model.rs`.

## Configuration

| Flag | Default | |
| --- | --- | --- |
| `--out` | `target/doc-site` | replaced on each run, but only if `jacket doc` made it (it leaves a `.jacket-doc` file) |
| `--base-url` | `/` | where the site is served, with both slashes |
| `--runtime` | none | folder with `jai_wasm.wasm` and `engine.mjs` |
| `--playground-url` | `https://matteopolak.com/playground/jacket` | the Playground link and "Open in playground" |
| `--source-ref` | the commit, else `v<version>` | tag or branch the Source links point at; given, it is used for every link |
| `--module` | all | build only these; broken links are then only warnings |
| `--check` | off | build and check, write nothing |
| `--allow-broken-links` | off | warn about broken intra-doc links instead of failing |

`JAIC_STDLIB` picks the standard library as it does for `jacket test`; by default it is the `stdlib/` of the project you run it in, which is the Jacket repository.

## Dependencies

`jaic` (`-doc_model`, found next to `jacket` or on `PATH`), `pulldown-cmark` for Markdown, `sha2` for example ids, and the doc test code in `crates/jacket/src/test/doctest.rs`. The Run button needs a browser that supports module workers and WebAssembly; the compiler's wasm build is `crates/jai-wasm`.
