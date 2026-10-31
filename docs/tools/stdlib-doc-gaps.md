# Standard-library doc gaps

## What it is

`jaic -doc_gaps DIR` lists the public procedures, structs, enums and aliases of one library, and the module itself, with whether each has a doc comment and an example. `tools/ci/check_stdlib_docs.py` runs it over every module of `stdlib/`, prints a report (worst module first) and fails CI when a public declaration is added without a doc comment.

## How it works

**What is public.** A declaration an importer can name, found the way the module is read: the `.jai` files of the module folder, then every file they `#load` (most modules keep their code in `stdlib/internal` and `#load` it, so those declarations are the module's own). Not public:

- anything under `#scope_file` or `#scope_module`;
- modules that are only `#import`ed (`internal/...` behind an `Impl :: #import`): they are reached through aliases, and the alias is what is listed;
- variables, plain constants, fields and enum members;
- files that declare foreign procedures (a procedure with no body): they mirror a native API (`Vulkan`, `POSIX`, `SDL3`, ...) and the vendor documents it. A hand-written helper in such a file is therefore not listed; keep helpers in their own file.

**What counts as documented.** What the editor shows: the comment run directly above the declaration. An alias (`hello :: Impl.hello;`) with no comment is documented when its target is (see [doc comments](../compiler/doc-comments.md), "Aliases"). A module is documented by the comment that opens its `module.jai`, unless that comment sits directly above the first declaration. An example is a fenced block tagged `jai` (or untagged) in the docs.

**The check.** `tools/ci/undocumented_stdlib.txt` has one line per undocumented public declaration, `Module.symbol`, or `Module` for the module's own docs (overloads share a line). The list only shrinks, like [diagnostic coverage](diagnostic-coverage.md):

- an undocumented public declaration that is not listed fails the check: write the doc comment;
- a listed one that now has docs, or no longer exists, fails until `--update` removes it.

Examples are reported (`--examples`) but not enforced.

## How to change it

```
PYTHONPATH=tools python3 tools/ci/check_stdlib_docs.py --report --top 10          # worst modules
PYTHONPATH=tools python3 tools/ci/check_stdlib_docs.py --report --module Basic    # one module, by line
PYTHONPATH=tools python3 tools/ci/check_stdlib_docs.py --report --examples        # count missing examples too
PYTHONPATH=tools python3 tools/ci/check_stdlib_docs.py --update                   # after documenting some
```

- Document a listed declaration, run `--update`, commit the shorter list. Documenting the target of an alias documents every alias of it.
- What counts as public or documented: `crates/jaic-syntax/src/doc_comments/gaps.rs` (unit tests there); the check, report and list handling: the script (`tools/ci/test_check_stdlib_docs.py`).
- `python3 tools/dev/dev.py repo-checks stdlib-docs` runs only this check. The pre-commit hook skips it (it needs `jaic`); CI runs it with the other repository checks.

## Configuration

`JAIC` names the binary to use; otherwise the built `jaic` under the cargo target directory is used, and built with `cargo build -p jaic-cli` when missing. `JAIC_STDLIB` is set to the checkout's `stdlib/`.

## Dependencies

`jaic -doc_gaps` (`jaic::doc_comments::gaps`, which uses `docs::docs_site` for aliases), `toolkit.repo`, and cargo when `jaic` is not built.
