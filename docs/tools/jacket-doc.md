# jacket doc

`jacket doc` writes the API reference for the standard library as a static website: one page per module and per item, a search index, and the Tools pages (the files in `docs/reference/`). Every doc-comment example gets a Run button that compiles and runs it in the browser with the playground's compiler build. Nothing it makes is committed; a release builds it and attaches `jacket-docs-<version>.tar.gz`.

```sh
jacket doc                                  # target/doc-site/, served from /
jacket doc --base-url /docs/jacket/ --out site
jacket doc --module String --module Basic   # just these, for a quick look
jacket doc --check                          # build in memory and check links only
```

Flags are in the [command-line reference](../reference/command-line.md). Without `--runtime` the Run buttons are disabled and say why. Serve the output with any static server (`python3 -m http.server --directory target/doc-site`); links are absolute from `--base-url`, so opening files from disk won't style.

## How it works

1. **Facts from the compiler.** `jaic -doc_model <dir>` (`crates/jaic-syntax/src/doc_comments/model.rs`) reads a library with the language server's parser and prints a JSON model: public constants, aliases followed to their targets, overloads, parameter and result types, doc comments with links resolved, fields and members, platform limits, and for each type name the declaration the compiler resolved it to. Docs are found by the same rules as hover.
2. **Pages.** `crates/jacket/src/doc/` turns the model into HTML: `pages.rs` (index, module and item pages), `categories.rs`, `links.rs` (link checking), `site.rs` (where pages live, signatures), `markdown.rs`, `highlight.rs`, `guide.rs` (renders `docs/reference/*.md`), `search.rs`, `shell.rs`. A type name links to the item the compiler resolved it to, never by matching the name.
3. **Examples.** A ```` ```jai ```` fence is read by the function `jacket test` uses (`test::doctest::fenced`), so options like `ignore`, `no_run` and `skip_on(wasm)` mean the same. The Run button fetches `examples/<id>.jai`, the program `jacket test` builds, and runs it in a worker with a step budget. "Open in playground" goes to `<playground-url>#code=<base64url of the program>`.
4. **Search.** `search-index.json` loads on the first keystroke. A query matches by name; one with `->` matches overload types (`string -> [] string`).

Every page is `<path>/index.html` and every link ends in `/`, so a plain static server needs no rewrite rules. Characters that can't be in a file name become `-` plus a hex code (`operator-3c`). `--check` fails on internal links ending in `.html`, links to files that weren't written, and an item and module sharing a folder.

| Path | What |
| --- | --- |
| `index.html` | modules, by category |
| `api/<Module>/index.html`, `api/<Module>/<name>/index.html` | a module; an item |
| `tools/<page>/index.html` | the reference pages |
| `examples/<id>.jai` | programs the Run buttons run |
| `runtime/`, `runtime.json` | the browser compiler (`--runtime`) and where pages look for it |
| `assets/` | `site.css`, `site.js`, `run-worker.js`, the logo |

Modules are the folders of `stdlib/` with a `module.jai`, minus `internal/`, `prelude/` and `tests/`. 
**Module parameters.** Each `#module_parameters` of the module's files becomes a "Module parameters" section before the items: name, type, default, and the doc comment above or after the parameter. Put one parameter per line so each has its own comment. They are not counted by `jaic -doc_gaps`.

**Native bindings.** A file that declares foreign procedures mirrors a native API. The model marks its declarations `"binding": true`; the module page lists them in a collapsed "Native API" section after the module's own items, as one line each (types fold their members), and they have no item pages. They never count as undocumented (`jaic -doc_gaps` skips them). Search includes them with a "Native" badge and ranks them below the library's own API. Their `since` label is left off.

`runtime.json` (`{"engine": "...", "wasm": "..."}`) points at the bundle `tools/browser/build_scripting_wasm.py --release --output <dir>` writes ([browser build](../browser/playground.md)); `--runtime <dir>` copies `jai_wasm.wasm` and `engine.mjs` from it. A site that already hosts the compiler can drop the copy and edit `runtime.json`.

The hosted docs redirect `/docs/jacket/playground` to the playground, so the site never writes a `playground/` path or `playground.html`.

**Categories.** `stdlib/categories.toml` puts every module in one category. `jacket doc` fails when a module has no category, is listed twice, or a name matches no module, so add new modules to the file.

## Publishing

The release workflow's `docs` job runs `jacket doc --base-url /docs/jacket/ --runtime <bundle>` and attaches `jacket-docs-<version>.tar.gz`. The portfolio site's build downloads the asset for the tag it pins and extracts it into `/docs/jacket`:

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
- **Source links** go to the commit `jacket doc` was built from, which needs `stdlib/` to be in a git checkout; otherwise they link to the version tag. `--source-ref` overrides.
- **Versions menu:** `versions.json` lists one version; a site hosting several replaces it with one entry each.
- **"Since" versions.** `since 0.24.0` labels come from `crates/jacket/src/doc/since.tsv`, compiled into `jacket`. `tools/generators/generate_doc_since.py` writes it from the release tags; `check_generated.py` only validates it. An item with no row is new in the version being built. After a release is tagged, run `python3 tools/generators/generate_doc_since.py` and commit the file (it adds only new tags; `--full` rereads all).
- **Indented code** in a doc comment is a plain code block and never run; `jacket doc` warns for each. Use ```` ```jai ````; `jai,ignore` for fragments that can't stand alone, `jai,no_run` for complete ones needing a file or network.
- **Platform badges.** A module shows the systems it asserts (`#assert OS == ...`), else macOS, Linux and Windows, plus `wasm` when it builds for `-os wasm`. An item gets badges only when an `#if OS` makes it available on fewer systems than its module (`limits`/`holds_on` in `model.rs`, `Module::item_systems` in `doc/model.rs`).
- **Native API section.** `native_section` in `pages.rs`; the search flag is bit 2 of an index row's last column (`search.rs`, `site.js`). Links to a binding item go to `api/<Module>/#<name>` (`Site::item_href`), and `site.js` opens the collapsed sections around a `#` target.
- **Experimental badge.** A doc comment line that is exactly `@experimental` adds an amber badge (`experimental_badge` in `pages.rs`).

## Configuration

| Flag | Default | |
| --- | --- | --- |
| `--out` | `target/doc-site` | replaced each run, only if `jacket doc` made it |
| `--base-url` | `/` | where the site is served, with both slashes |
| `--runtime` | none | folder with `jai_wasm.wasm` and `engine.mjs` |
| `--playground-url` | `https://matteopolak.com/playground/jacket` | the Playground and "Open in playground" links |
| `--source-ref` | the commit, else `v<version>` | tag or branch for Source links |
| `--module` | all | build only these; broken links are then warnings |
| `--check` | off | build and check, write nothing |
| `--allow-broken-links` | off | warn instead of failing |

`JAIC_STDLIB` picks the standard library, as for `jacket test`; the default is the `stdlib/` of the project you run it in.

## Dependencies

`jaic` (`-doc_model`), `pulldown-cmark`, `sha2`, and `crates/jacket/src/test/doctest.rs`. The Run button needs module workers and WebAssembly; the compiler's wasm build is `crates/jai-wasm`.
