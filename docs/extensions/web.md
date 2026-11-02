# Web

`Extensions/Web` builds static sites from Jai: HTML templates compiled at compile time, components as plain procedures, a parallel page writer and a link check. `jacket doc`'s theme is written with it. Native only (it uses threads and files). Experimental.

```jai
#import "Extensions/Web";

Card :: (out: *Out, title: string, href: string) {
    html(out, #string HTML
<a class="card" href="{href}"><h2>{title}</h2></a>
HTML
    );
}

Page :: (out: *Out, cards: [] string) {
    html(out, #string HTML
<main>{for cards}{Card(it, "/")}{end}</main>
HTML
    );
}

w := writer_init("site", "/");
out: Out;
Page(*out, .["A & B"]);
write_page(*w, "", *out);           // site/index.html
problems := check_links(*w);        // links to pages or files that weren't written
```

## How it works

- **Templates.** `html(out, $t)` is a macro. `#run compile_template` parses the constant template once and inserts plain append calls in the caller's scope, so holes see the caller's locals. The literal bytes are copied as written.
- **Holes.**
  - `{expr}` is escaped for where it sits: text or a quoted attribute. Strings and integers are accepted; `bool`, floats and structs are compile errors.
  - An `Html` value (`cast(Html) s`) is inserted raw. Use it for markup you made or checked yourself.
  - Inside `<script>` and `<style>` only `Html` holes are allowed.
  - `{Name(args)}` (uppercase first letter) calls a component with `out` passed first. Components are ordinary procedures, so overloads work as slots: a polymorphic layout calls `{Main(page)}` and each page type supplies its own `Main`.
  - `{if c}`, `{else if c}`, `{else}`, `{for xs}` (with `it`/`it_index`) and `{end}`. Branches must end in the same HTML state (in text, in a tag, in an attribute).
  - Mistakes are compile errors pointing at the byte in the template (`Extensions/String_Literal`).
- **Out** is a growable byte buffer. Each `href`/`src` value is recorded as a link while the page is written, unless `record_links = false`. `reset` keeps the capacity, so one `Out` per thread serves every page.
- **Writer.** `write_page(w, "api/x", out)` writes `api/x.html` (the empty path writes `index.html`) and keeps its links; `write_file` writes any other file. Both are safe to call from several threads. `writer_init(dir, base, dry_run = true)` writes nothing but still records paths and links, which is how `jacket doc --check` works. `check_links` reports internal links that point at nothing written and links ending in `.html` or `/`, and resolves a link to the file or to the file plus `.html`; `#fragment` and `?query` parts are ignored. Links inside `Html` holes count too: the whole page is scanned when it is written.
- **render_parallel(jobs, render, data)** runs `render(out, *job, data)` on a `Thread_Group`, one reused `Out` per worker, temporary storage reset after each job.
- **Assets.** `content_stamp(bytes)` is 8 hex digits (top half of FNV-1a 64) for `?v=` cache busting. `compact_css`/`compact_js` drop comments and blank or edge whitespace; they don't minify.

## How to change it

- Template parsing and the generated code are in `template.jai` (`compile_template`, `Template.*` are what the generated code calls). The HTML states are the `Mode` enum; a new hole context needs a mode and an escape rule.
- New compile errors get a case in `tests/negative/` with its `.stderr`.
- Every public declaration needs a doc comment with an example; `jacket test stdlib/Extensions/Web` runs them and `tests/suite.jai`.

## Dependencies

`Basic`, `Pool`, `Thread`, `File`, `Sort`, `Extensions/Hash_Map`, `Extensions/String_Literal`.
