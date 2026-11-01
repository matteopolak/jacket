# Introduction

Put pages and assets in a folder, then run:

```sh
sitegen site public
```

Every `.md` file becomes an `.html` file at the same path. Every other file is copied under a name
with a digest in it, for example `style.css` becomes `style.<digest>.css`, and the pages are
rewritten to match.

1. Write a page.
2. Link to it from another page: `[intro](guide/intro.md)`.
3. Build, and look at the warnings. A link to a page that does not exist is reported.

Back to the [home page](../index.md).
