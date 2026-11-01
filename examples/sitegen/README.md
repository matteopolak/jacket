# sitegen

A static site generator. A folder of Markdown pages and a template go in; HTML comes out. Every other file is published under a name that contains a hash of its content, and small images can be written into the page as `data:` URIs.

```sh
jaic examples/sitegen/main.jai -exe sitegen
./sitegen examples/sitegen/demo public          # the demo site in this folder
./sitegen site public --template theme.html --inline-limit 4096 --verbose
```

`sitegen <src> <out>` reads everything in `<src>` before it writes anything to `<out>`. Problems that do not stop a build (a link to a page that does not exist) are printed as warnings; a broken template stops it.

## What a site looks like

```
site/
  _template.html        the page template (names starting with _ or . are never published)
  index.md              becomes index.html
  guide/intro.md        becomes guide/intro.html
  style.css             becomes style.<hash>.css
  img/logo.png          becomes img/logo.<hash>.png
```

A page may start with front matter: `title: ...` overrides the first `# heading`, and `draft: true` leaves the page out.

In a page, `[text](other.md#part)` becomes a link to `other.html#part`, and `![alt](img/logo.png)` points at the hashed file (or inlines it when it is no bigger than `--inline-limit`, 2048 bytes by default). The Markdown subset: headings, paragraphs, `*emphasis*`, `**strong**`, `` `code` ``, links, images, fenced code, block quotes, bullet and numbered lists (nested by indenting) and `---` rules. HTML in a page is escaped.

The template is HTML with `{{ placeholders }}`: `title`, `content`, `nav`, `root`, `path`, `{{ asset file.css }}` for an asset's URL and `{{ inline logo.png }}` for a data URI. [demo/_template.html](demo/_template.html) uses all of them.

## Map

- [main.jai](main.jai): The command line (`Extensions/Args`).
- [markdown/](markdown): `blocks.jai` finds the structure line by line, `inline.jai` handles emphasis, code, links and images. It knows nothing about sites.
- [site/](site): `build.jai` is the pipeline and the link rewriting, `assets.jai` the hashed names (`md5`) and `data:` URIs (`Base64`), `template.jai` the placeholders, `pages.jai` front matter and the navigation (pages live in a `Xar`), `paths.jai` relative-path arithmetic.
- [demo/](demo): A small site to try it on.
- [tests/](tests): `jacket test` in this folder. The Markdown forms, the small parts, and whole builds in temporary folders.
- [tools/smoke.sh](tools/smoke.sh): Builds the program and checks the demo site's output. Not part of CI.
- [FRICTION.md](FRICTION.md): What got in the way while writing it.

## How it works

`load_site` lists the source folder, parses the template, and reads each file: `.md` files become `Page`s in a `Xar` (so the pointers the link checker keeps stay valid as more are added), everything else an `Asset` whose output name is `stem.<first six hex digits of md5>.ext`. `render_pages` runs the Markdown renderer once per page with a hook (`rewrite_url`) that decides what each link points at, then sorts the pages for the navigation. `write_site` fills the template for each page and writes the files.

All of it is allocated in one `Flat_Pool` that is dropped when `build_site` returns, so the steps never free anything; only the result (counts, file names, warnings) is copied out.

Output names depend only on the bytes of a file, so building twice gives the same names, and a changed file gets a new one. Old files in `<out>` are never deleted.

## Things to try

- Change a byte in `demo/style.css` and see the name in the pages change.
- Link to `nowhere.md` from a page and read the warning.
- Use `--inline-limit 100000` and look at the banner in `index.html`.
- Swap the hash in [site/assets.jai](site/assets.jai) for another module and run the tests.
