# Standard-library doc gaps

`jaic -doc_gaps DIR` lists the public procedures, structs, enums and aliases of one library (and the module itself) with whether each has a doc comment and an example. `tools/ci/check_stdlib_docs.py` runs it over every module of `stdlib/`, prints a report (worst module first) and fails CI when a public declaration is added without a doc comment. It also fails on code indented four spaces in a comment outside `tests/`, `bench/`, `examples/` and `fuzz/`: use a fence.

## How it works

- **Public** means an importer can name it, found in the module folder's `.jai` files and every file they `#load`. Not public: `#scope_file`/`#scope_module` items, modules only `#import`ed (reached through aliases, and the alias is listed), variables, plain constants, fields, enum members, and files that declare foreign procedures (they mirror a native API; keep hand-written helpers in their own file).
- **Documented** means the comment run directly above the declaration, as the editor shows it. An alias with no comment is documented when its target is ([doc comments](../compiler/doc-comments.md)). A module is documented by the comment that opens `module.jai`. Examples are reported (`--examples`) but not enforced.
- `tools/ci/undocumented_stdlib.txt` has one line per undocumented public declaration (`Module.symbol`, or `Module`). The list only shrinks, like [diagnostic coverage](diagnostic-coverage.md): a new undocumented declaration fails, and so does a listed one that is now documented or gone.

## How to change it

```sh
PYTHONPATH=tools python3 tools/ci/check_stdlib_docs.py --report --top 10
PYTHONPATH=tools python3 tools/ci/check_stdlib_docs.py --report --module Basic
PYTHONPATH=tools python3 tools/ci/check_stdlib_docs.py --update    # after documenting some
```

Document a listed declaration, run `--update`, commit the shorter list. The rules for public and documented are in `crates/jaic-syntax/src/doc_comments/gaps.rs`; the check and list handling in the script. `python3 tools/dev/dev.py repo-checks stdlib-docs` runs only this check (the pre-commit hook skips it, since it needs `jaic`).

## Configuration

`JAIC` names the binary; otherwise the built `jaic` under the cargo target directory is used (built if missing).
