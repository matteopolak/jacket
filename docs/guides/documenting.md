# Documenting a library

`jacket doc` turns the comments above your declarations into a website: a page for every module and every item, search, and your own Markdown pages. The examples in those comments are checked by `jacket test` and can be run in the browser.

## A documented module

Comments directly above a declaration are its documentation, written in Markdown. The comment at the top of `module.jai` documents the module.

```jai
// src/Slugs/module.jai

// Slugs for URLs: `slugify` turns a title into lowercase words joined by `-`.
//
// ```jai
// assert(slugify("Many  Spaces") == "many-spaces");
// ```

#import "Basic";
#import "String";

// Turns a title into a URL slug: lowercase words joined by `-`.
// Cut it down with [shortened].
//
// ```jai
// assert(slugify("Hello, World!") == "hello-world");
// ```
//
// - title: the text to convert.
slugify :: (title: string) -> string {
    // ...
}

// How [`shortened`] cuts.
Cut :: enum {
    // Cut at the limit, even in a word.
    HARD;
    // Cut at the last dash before the limit.
    AT_DASH;
}

// Options for [`shortened`].
Short_Options :: struct {
    // The most bytes to keep.
    limit := 40;
    // Where to cut.
    cut := Cut.AT_DASH;
}

// The start of `slug`, cut as `options` say.
//
// ```jai
// options: Short_Options;
// options.limit = 5;
// assert(shortened("hello-world", options) == "hello");
// ```
//
// - slug: a slug from [slugify].
// - options: the limit and where to cut.
shortened :: (slug: string, options: Short_Options) -> string {
    // ...
}
```

```sh
jacket doc --out site
```

The build prints a line like `jacket doc: 2 modules, 6 items (3 procedures), written to site`. Open `site/index.html` through a static server (links are absolute, so opening the files directly won't style them):

```sh
python3 -m http.server --directory site
```

## Writing doc comments

A doc comment is the run of `//` lines directly above a declaration, with no blank line between. `///` works too, and so does a `/* */` block that ends directly above it. One space after the marker is removed. A line of only `-----` or `=====` ends the run.

Start with a sentence that says what the item does, because it is the summary in lists and search results. Say what a function returns and when it fails; leave out what the signature already says.

- **Parameters.** Either a list item starting with the name, `- title: the text to convert.`, or a comment above the parameter or after it.
- **Fields and enum members.** A comment above, or at the end of the same line.
- **Links.** `[slugify]`, ``[`slugify`]``, `[Cut.HARD]`, `[Short_Options.limit]` and `[text](slugify)` link to the item, wherever it is in your library. A name that doesn't resolve stays plain text, and `jacket lint` reports it as `broken_doc_link`. Link to other modules with `[Basic.print]`. Ordinary URLs work as in any Markdown.
- **Aliases.** `hello :: Impl.hello;` with no comment of its own shows the target's docs, so document the target once.
- **Experimental.** A line that is exactly `@experimental` marks an item, or a whole module when it is in the module comment, with a badge.
- **Types.** A type in a signature links to its page. The build fails when it has none, so document the types your public API uses or re-export them.

Markdown lists, tables and code all work.

### Examples

Fenced code is the best documentation, and every ```` ```jai ```` block runs as a test with `jacket test`:

```jai
// Counts the words in `text`.
//
// ```jai
// assert(count_words("one two three") == 3);
// ```
```

A block that is only statements becomes the body of a procedure with `Basic` and your library imported. A block with its own `main` is used as written. A failed `assert` fails the test, so end examples with one rather than a `print` nobody checks.

The words after `jai` change what happens:

| Fence | The example is |
| --- | --- |
| ```` ```jai ```` | built and run; must exit 0 |
| ```` ```jai,no_run ```` | only type-checked: for files, the network, a window |
| ```` ```jai,compile_fail ```` | expected not to compile: to show what is rejected |
| ```` ```jai,should_fail ```` | built and run; must fail |
| ```` ```jai,ignore ```` | not checked: for a fragment that can't stand alone |

Put `#import` lines inside the block when it needs more than `Basic`:

```jai
// ```jai,no_run
// #import "File";
// text := read_entire_file("title.txt");
// print("%\n", slugify(text));
// ```
```

Platform limits go after a comma: `jai,skip_on(wasm)`, `jai,only(linux, macos)`, `jai,needs(network)`, `jai,timeout(5)`. To apply one to every example in a library, put a `// test:` line in a comment of its own after the module docs at the top of `module.jai`, with a blank line on each side:

```jai
// Slugs for URLs.
//
// ```jai
// assert(slugify("Many  Spaces") == "many-spaces");
// ```

// test: skip_on(.WASM, reason = "uses the file system")

#import "Basic";
```

On the site every example has Copy and, when the site is built with a browser compiler, Run and an editor: change the code in place and press Run (or Ctrl/Cmd+Enter). Give `--runtime DIR` a folder with `jai_wasm.wasm` and `engine.mjs` to enable it, and `playground` in `[doc]` for an "Open in playground" link. Without them the Run button is disabled and says why.

Don't indent code to show it. An indented block is never run, and `jacket doc` warns about it.

Use two fences in a row, ```` ```jai before ```` and ```` ```jai after ````, in a Markdown page to show a change side by side.

## Settings

`[doc]` in `jacket.toml` sets the rest. `jacket doc` with no path reads it from the nearest `jacket.toml`.

```toml
[project]
entry = ["src/Slugs/module.jai"]
import_path = ["src"]

[doc]
path = "src"                             # a module, or a folder of modules
name = "Kit"                             # header and page titles
description = "Small text helpers for Jai programs."
home = "notes/home.md"                   # the front page; default: name and description
repo = "https://github.com/me/kit"       # turns on Source links
url = "https://kit.example.com"          # canonical links, sitemap.xml, robots.txt
logo = "assets/logo.svg"
pages_title = "Guides"                   # sidebar heading for the pages below

[[doc.category]]                         # headings in the sidebar and on the front page
name = "Text"
modules = ["src/**"]

[[doc.group]]                            # folding groups inside a heading
name = "Slugs"
modules = ["src/Slugs"]

[[doc.group]]
name = "Words"
modules = ["src/Words"]

[[doc.page]]                             # a Markdown page under pages_title
file = "notes/getting-started.md"
name = "Getting started"

[[doc.section]]                          # a heading of its own, split from one file
file = "notes/recipes.md"
name = "Recipes"
```

Paths are relative to `jacket.toml`. A command-line flag (`--name`, `--repo`, `--out`) wins over the setting.

- **`path`** is one library: a folder with `module.jai`, or a folder of modules. Every folder with a `module.jai` is a module, except `internal/`, `prelude/` and `tests/`. Your own folder is on the import path, so modules can import each other.
- **Categories** group modules under headings. A module goes under the last category whose pattern matches its folder. Add `badge = "labs"` (and `tip`, `color`) to mark its modules and items.
- **Groups** fold modules inside a heading. Once you have any, every module must be in one.
- **Pages** are plain Markdown files. The header's link goes to the first. A relative link between pages and to `../api/Module/item/` works.
- **Sections** split one file into many pages: the text before the first `##` is the overview, each `##` is a folding group in the sidebar, and each `###` is a page titled with the heading. A link to `file.md#heading` goes to that heading's page.

  ```md
  # Recipes

  Short answers for common jobs.

  ## URLs

  ### Make a blog URL

  Call [`slugify`](../api/Slugs/slugify/) on the title.
  ```

A pattern that matches no module, a module in no group (once there are groups) and a colour that isn't `#hex` or a plain name are errors.

`jacket test` needs to know where the library is when it isn't the current folder: pass `--path src/Slugs`, or set `[test] paths = ["src/**"]` once so the doc examples run along with the tests.

## Building and checking

```sh
jacket doc                    # the [doc] library, into target/doc-site
jacket doc --check            # build in memory and check links; writes nothing
jacket doc --module Slugs     # only this module, for a quick look
jacket doc --base-url /kit/ --version 1.2.0
```

`--check` fails on links to files that weren't written, types in signatures with no page to link to, and links ending in `.html`. Run it in CI.

`--base-url` is where the site will be served, with a slash on each side. `--version` shows in the header and turns on "since" labels when `[doc] since` names a file of releases. `--source-ref` picks the tag or branch Source links point at; by default, it is the commit you built from.

`jacket test --doc` runs just the examples, and `jacket test --doc --check-only --os windows` type-checks them for another OS without running anything.

## Publishing

The output is static files. Upload `target/doc-site` to any host, for example GitHub Pages from a workflow that runs `jacket doc --base-url /kit/ --version $TAG`. Set `url` so pages carry canonical links and the build writes `sitemap.xml`.

## Related

- [Testing](testing.md) for how examples run as tests.
- [Doc comments](../compiler/doc-comments.md) for how the editor reads them.
- [jacket doc](../tools/jacket-doc.md) for how the site is built and themed.
