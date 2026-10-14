# Public-language contract fixtures

These original fixtures turn the language guide's demonstrated rules into small regression cases. They are independent of the supplied tutorial source and do not contain copied tutorial bodies.

Build jaic from this workspace, then run the cases with its explicit target artifact:

```sh
cargo build -p jaic-cli --bin jaic
python3 tests/cleanroom-language/run.py --jaic "$(pwd)/target/debug/jaic"
```

The runner requires an absolute `jaic` executable located under this workspace's `target/` directory. It runs positive cases with `-interp`, checks one expected visibility error with `-check`, and compares exact `ok` output. It never locates or runs a compiler from an external example tree. To run against another source build, build it in this workspace's target directory and pass that executable explicitly.

## Coverage

The manifest covers declarations and conversions, ranges and iteration, defaults and named arguments, `defer`, file/module/export scopes, independent `#load` scopes, filtered imports, aggregate `using,map`, module parameter identity, polymorphism, `#run`/`#assert`/`#if`, macro expansion, and `#insert`.

The fixtures intentionally do not assert unresolved behavior such as merging overload sets from multiple filtered imports, module-import name mapping, Runtime_Support instance identity, evaluation order for side-effecting arguments, or ambiguous/reversed range boundaries. The guide remains the contract for these undecided edges; add a fixture only when an observable rule is evidenced.

The optional `--stdlib-root` selects a newly implemented library in another worktree; it defaults to this checkout's `stdlib`. Each fixture has a 30-second timeout, adjustable with `--timeout`. A timeout is a test failure.
