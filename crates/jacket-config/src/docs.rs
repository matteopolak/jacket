//! One-line descriptions of every `jacket.toml` key, for editors. A test keeps this in step with
//! [`crate::schema::known_keys`].

/// What `key` of `table` (`lint.portability`, `dependencies.jaison`) is for.
pub(crate) fn describe(table: &str, key: &str) -> Option<&'static str> {
    let table = if table.starts_with("dependencies.") {
        "dependencies.<name>"
    } else if table.starts_with("test.profile.") {
        "test.profile.<name>"
    } else if table.starts_with("target.") {
        "target.<name>"
    } else {
        table
    };

    DOCS.iter()
        .find(|(t, k, _)| *t == table && *k == key)
        .map(|(.., text)| *text)
}

/// What a table itself holds.
pub(crate) fn describe_table(table: &str) -> Option<&'static str> {
    let (parent, key) = match table.rsplit_once('.') {
        Some((parent, key)) => (parent, key),
        None => ("", table),
    };

    if parent == "dependencies" || parent == "target" || parent == "test.profile" {
        return Some(match parent {
            "dependencies" => "A git dependency, pinned by the commit of its submodule.",
            "target" => "A named build target that `--target` selects.",
            _ => "A named set of test settings that `--profile` lays over `[test]`.",
        });
    }

    describe(parent, key)
}

const DOCS: &[(&str, &str, &str)] = &[
    (
        "",
        "project",
        "What `jaic` builds and where it looks for modules.",
    ),
    (
        "",
        "toolchain",
        "The compiler release the project is written for.",
    ),
    ("", "lint", "`jailint` settings."),
    ("", "format", "`jaifmt` settings."),
    ("", "experimental", "Compiler switches that may change."),
    ("", "postgres", "Typed-query settings."),
    ("", "test", "How `jacket test` finds and runs tests."),
    ("", "doc", "`jacket doc` site settings."),
    (
        "",
        "target",
        "Named build targets (`[target.pico]`) that `--target` selects.",
    ),
    (
        "",
        "dependencies",
        "Git dependencies (`[dependencies.jaison]`), each a submodule pinned by its commit.",
    ),
    ("", "resolver", "How `jacket dep` picks versions."),
    (
        "",
        "package",
        "What this project is, when other projects depend on it.",
    ),
    (
        "project",
        "entry",
        "The files given to `jaic` (a metaprogram, or the program), relative to the file.",
    ),
    (
        "project",
        "import-path",
        "More module folders, as `-import_dir` adds.",
    ),
    (
        "toolchain",
        "version",
        "The `x.y.z` release the project is written for.",
    ),
    (
        "resolver",
        "min-release-age",
        "How long ago a version must have been published (`3d`, `12h`, `0`). Default `2w`; `0` turns the policy off.",
    ),
    (
        "resolver",
        "release-age-jitter",
        "Adds a per-version offset in `[0, jitter]` to the age, so a bad release reaches projects in stages. Default `3d`.",
    ),
    (
        "resolver",
        "seed",
        "Random text the jitter is hashed from, written by `jacket dep init` and the first `jacket dep add`.",
    ),
    (
        "resolver",
        "dedupe",
        "Experimental: solve every package's ranges together and share one copy per name.",
    ),
    (
        "package",
        "name",
        "The import name. Default: the repository's name.",
    ),
    (
        "package",
        "version",
        "The version this tree is; `jacket dep add` warns when it differs from the tag.",
    ),
    (
        "package",
        "module",
        "The folder holding `module.jai`. Default: the repository root.",
    ),
    (
        "package",
        "jacket-version",
        "The jacket versions this package builds with (`>=0.40`).",
    ),
    ("package", "platforms", "The operating systems it supports."),
    (
        "package",
        "import-path",
        "More module folders searched for this package's files only.",
    ),
    (
        "dependencies.<name>",
        "git",
        "`user/repo` (GitHub), `host/path` (HTTPS), or a full URL or `git@...` address.",
    ),
    (
        "dependencies.<name>",
        "version",
        "A version range (`^1.2`, `>=1.0, <2.0`); the newest allowed tag is pinned.",
    ),
    ("dependencies.<name>", "tag", "An exact tag."),
    (
        "dependencies.<name>",
        "rev",
        "A commit, for a repository with no usable tags.",
    ),
    (
        "dependencies.<name>",
        "path",
        "The submodule folder, relative to the project. Default `modules/<name>`.",
    ),
    (
        "dependencies.<name>",
        "module",
        "The folder of the repository that holds `module.jai`. Default: its `[package] module`, else the root.",
    ),
    (
        "dependencies.<name>",
        "scheme",
        "How tags are read: `semver` (default), `calendar` or `integer`.",
    ),
    (
        "dependencies.<name>",
        "tag-prefix",
        "A fixed text stripped from tags before they are read (`alpha-`).",
    ),
    (
        "dependencies.<name>",
        "min-release-age",
        "Replaces `[resolver] min-release-age` for this entry (`\"0\"` waives it).",
    ),
    (
        "dependencies.<name>",
        "shared",
        "Written by `jacket dep` under `[resolver] dedupe`: another package asks for this one and it is shared.",
    ),
    (
        "target.<name>",
        "triple",
        "The LLVM target triple (`aarch64-none-elf`). A board supplies it when this is left out.",
    ),
    (
        "target.<name>",
        "board",
        "A board module under `Extensions/Embedded/Boards` that configures the build: memory map, linker script and startup code.",
    ),
    (
        "target.<name>",
        "runner",
        "The command that runs the built program, with `{exe}` standing for its path.",
    ),
    ("lint", "lifetimes", "Settings of the lifetime rules."),
    (
        "lint",
        "freestanding",
        "Limits for the `freestanding` rule.",
    ),
    (
        "lint",
        "exclude",
        "Patterns of paths not to lint, relative to the file.",
    ),
    (
        "lint",
        "max-nesting",
        "How many blocks deep code may nest before `excessive_nesting` fires.",
    ),
    (
        "lint",
        "max-complexity",
        "The score above which `cognitive_complexity` reports a procedure.",
    ),
    (
        "lint",
        "max-arguments",
        "How many parameters a procedure may take before `too_many_arguments` reports it.",
    ),
    (
        "lint",
        "max-bools",
        "How many `bool` parameters a procedure may take before `excessive_bools` reports it.",
    ),
    (
        "lint",
        "max-stack-array-bytes",
        "How many bytes a local fixed array may take before `large_stack_array` reports it.",
    ),
    (
        "lint",
        "magic-number-allow",
        "Numbers `magic_number` never reports, besides its built-in ones.",
    ),
    (
        "lint",
        "magic-number-min",
        "Whole numbers smaller than this in magnitude are not magic (`magic_number`).",
    ),
    (
        "lint",
        "magic-number-in-tests",
        "`magic_number` also checks files in `tests` and `test` folders.",
    ),
    (
        "lint",
        "rules",
        "Rule name and level text (`allow`, `warn`, `deny`).",
    ),
    (
        "lint",
        "overrides",
        "Rule levels for some paths (`[[lint.overrides]]`); a later entry wins over an earlier one.",
    ),
    (
        "lint",
        "portability",
        "Which code must build on every operating system (`platform_leak`).",
    ),
    (
        "lint.lifetimes",
        "mode",
        "How strictly the lifetime rules read a procedure.",
    ),
    (
        "lint.freestanding",
        "max-frame-bytes",
        "How many bytes of fixed-size locals a procedure's stack frame may hold before `freestanding` reports it.",
    ),
    (
        "lint.portability",
        "targets",
        "The operating systems portable code is built for: `linux`, `macos`, `windows`. Left out, all three.",
    ),
    (
        "lint.portability",
        "portable",
        "Patterns (relative to the file) of the code that must build on every target.",
    ),
    (
        "lint.portability",
        "platforms",
        "Globs of files that build only on the operating systems listed; the most specific glob wins.",
    ),
    ("format", "indent-width", "Spaces per indent level."),
    (
        "format",
        "case-indent",
        "Indent of `case` lines from their `if` or `#if`. Default: `indent-width`.",
    ),
    (
        "format",
        "case-body-indent",
        "Indent of a case's body from its `case`. Default: `indent-width`.",
    ),
    (
        "format",
        "max-blank-lines",
        "The most consecutive blank lines kept.",
    ),
    ("format", "max-width", "The column lists and calls wrap at."),
    (
        "format",
        "brace-style",
        "Where a block's opening brace goes.",
    ),
    (
        "format",
        "blank-lines-between-items",
        "Blank lines between top-level items.",
    ),
    (
        "format",
        "blank-line-after-declarations",
        "A blank line after a run of declarations.",
    ),
    (
        "format",
        "blank-lines-around-blocks",
        "Blank lines around multi-line blocks.",
    ),
    (
        "format",
        "blank-line-between-cases",
        "A blank line between `case`s.",
    ),
    (
        "format",
        "blank-line-before-comments",
        "A blank line before a comment that starts a paragraph.",
    ),
    (
        "format",
        "blank-line-after-guards",
        "A blank line after an early `return` guard.",
    ),
    (
        "format",
        "blank-line-before-return",
        "A blank line before a final `return`.",
    ),
    (
        "format",
        "format-doc-examples",
        "Format the code examples in doc comments.",
    ),
    (
        "format",
        "ignore",
        "Glob patterns of paths not to format, relative to the file.",
    ),
    (
        "experimental",
        "parse-threads",
        "Threads that parse files ahead of the checker (`-parse_threads`).",
    ),
    (
        "experimental",
        "serial-parse",
        "Parse on the checking thread (`-serial_parse`).",
    ),
    (
        "experimental",
        "codegen-units",
        "Object files a native build is split into (`-codegen_units`).",
    ),
    (
        "postgres",
        "managed",
        "Describe queries in a managed server when no database URL is set.",
    ),
    (
        "postgres",
        "managed-version",
        "The pinned PostgreSQL release the managed server runs, such as `\"17.6.0\"`.",
    ),
    (
        "test",
        "paths",
        "Where the tests are, relative to the file: folders, or path patterns.",
    ),
    ("test", "jobs", "How many tests run at once."),
    ("test", "timeout", "How long one test may run."),
    (
        "test",
        "slow",
        "How long before a test is reported as slow.",
    ),
    (
        "test",
        "retries",
        "How many times a failed test is run again.",
    ),
    ("test", "fail-fast", "Stop at the first failure."),
    (
        "test",
        "flaky-fails",
        "Whether a test that passes only on a retry fails the run.",
    ),
    (
        "test",
        "runner",
        "The command that runs a built test program.",
    ),
    (
        "test",
        "needs",
        "What a test needs installed before it runs.",
    ),
    (
        "test",
        "profile",
        "Named sets of settings (`[test.profile.ci]`) that `--profile` lays over these.",
    ),
    ("test.profile.<name>", "jobs", "How many tests run at once."),
    (
        "test.profile.<name>",
        "timeout",
        "How long one test may run.",
    ),
    (
        "test.profile.<name>",
        "slow",
        "How long before a test is reported as slow.",
    ),
    (
        "test.profile.<name>",
        "retries",
        "How many times a failed test is run again.",
    ),
    (
        "test.profile.<name>",
        "fail-fast",
        "Stop at the first failure.",
    ),
    (
        "test.profile.<name>",
        "flaky-fails",
        "Whether a test that passes only on a retry fails the run.",
    ),
    (
        "test.profile.<name>",
        "runner",
        "The command that runs a built test program.",
    ),
    (
        "test.profile.<name>",
        "needs",
        "What a test needs installed before it runs.",
    ),
    ("doc", "url", "Canonical origin of the published site."),
    (
        "doc",
        "og-image",
        "Social preview image: a URL or a file relative to jacket.toml.",
    ),
    ("doc", "site-name", "Social site name; defaults to `name`."),
    (
        "doc",
        "canonical-version",
        "Preferred version; other versions use noindex.",
    ),
    (
        "doc",
        "theme",
        "Theme files that replace or extend the embedded documentation theme.",
    ),
    (
        "doc",
        "path",
        "The library: a module (a folder with `module.jai`) or a folder of modules.",
    ),
    (
        "doc",
        "name",
        "The name in the header and page titles. Default: the library's folder name.",
    ),
    (
        "doc",
        "breadcrumb",
        "The first breadcrumb on every page, linking to the front page.",
    ),
    (
        "doc",
        "home",
        "A Markdown file for the front page, relative to jacket.toml.",
    ),
    (
        "doc",
        "description",
        "The front page's description for search engines and link previews.",
    ),
    (
        "doc",
        "logo",
        "An `.svg` or `.png` for the header, and the tab icon unless `icon` is set.",
    ),
    (
        "doc",
        "icon",
        "An `.svg` or `.png` for the browser tab when it should differ from the logo.",
    ),
    (
        "doc",
        "repo",
        "The repository web address, for Source links.",
    ),
    (
        "doc",
        "playground",
        "The playground address examples open in.",
    ),
    (
        "doc",
        "since",
        "A file of the releases items first appeared in.",
    ),
    (
        "doc",
        "category",
        "Headings the modules are listed under, in order; a module goes under the last one that matches it.",
    ),
    (
        "doc",
        "group",
        "Folding groups inside each heading, in order.",
    ),
    (
        "doc",
        "pages-title",
        "The heading of the extra pages, and the header link to them. Default `Guides`.",
    ),
    ("doc", "page", "Markdown files rendered as extra pages."),
    (
        "doc",
        "section",
        "Markdown files split into one page per `###` heading.",
    ),
    (
        "doc",
        "files",
        "Files published as they are at the site root under their own names.",
    ),
    (
        "doc",
        "format",
        "`[format]` settings that differ for the examples in doc comments.",
    ),
    (
        "doc.format",
        "indent-width",
        "Spaces per indent level in examples.",
    ),
    (
        "doc.format",
        "case-indent",
        "Indent of `case` lines in examples.",
    ),
    (
        "doc.format",
        "case-body-indent",
        "Indent of a case's body in examples.",
    ),
    (
        "doc.format",
        "max-blank-lines",
        "The most consecutive blank lines kept in examples.",
    ),
    ("doc.format", "max-width", "The column examples wrap at."),
    (
        "doc.format",
        "brace-style",
        "Where a block's opening brace goes in examples.",
    ),
    (
        "doc.format",
        "blank-lines-between-items",
        "Blank lines between top-level items in examples.",
    ),
    (
        "doc.format",
        "blank-line-after-declarations",
        "A blank line after a run of declarations in examples.",
    ),
    (
        "doc.format",
        "blank-lines-around-blocks",
        "Blank lines around multi-line blocks in examples.",
    ),
    (
        "doc.format",
        "blank-line-between-cases",
        "A blank line between `case`s in examples.",
    ),
    (
        "doc.format",
        "blank-line-before-comments",
        "A blank line before a paragraph comment in examples.",
    ),
    (
        "doc.format",
        "blank-line-after-guards",
        "A blank line after an early `return` guard in examples.",
    ),
    (
        "doc.format",
        "blank-line-before-return",
        "A blank line before a final `return` in examples.",
    ),
    ("doc.category", "name", "The heading."),
    ("doc.category", "modules", "Patterns of module folders."),
    (
        "doc.category",
        "badge",
        "The badge text; none for no badge.",
    ),
    (
        "doc.category",
        "tip",
        "What the badge says on hover. Default: the name.",
    ),
    ("doc.category", "color", "The badge colour (`#1f7a8c`)."),
    (
        "doc.category",
        "color-dark",
        "The badge colour for dark themes. Default: `color`.",
    ),
    ("doc.group", "name", "The group's name."),
    ("doc.group", "modules", "Patterns of module folders."),
    ("doc.page", "file", "The Markdown file."),
    ("doc.page", "name", "Its name in the sidebar and title."),
    ("doc.section", "file", "The Markdown file."),
    (
        "doc.section",
        "name",
        "The sidebar heading; its folder is this as a slug.",
    ),
];
