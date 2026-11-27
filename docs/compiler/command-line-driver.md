# Compiler command-line driver

`jaic-cli-args` owns the compiler's command-line model: the `Cli` struct, whose fields are the
options (declared with [`argdef`](../tools/argdef.md)). The actual `jaic` parser and tooling such as
jacket read the same struct's spec, so they agree on spellings and arity.
The neutral crate has no execution backend, allocator, crash hook or native library discovery.

`jaic_cli_args::arguments::target_flags` returns target selection, consumed option-value
positions and arguments for host discovery. It consumes unrelated option values before
looking for another flag, accepts inline values, and preserves plugin/metaprogram/program
boundaries. Missing-value errors reuse the options' `needs` text. Jacket uses those positions to resolve paths relative to the user's directory.

```rust
let parsed = jaic_cli_args::arguments::target_flags(&["-os=wasm".into()])?;
assert!(parsed.host.is_empty());
```

Change option spellings/arity by editing the field's `#[arg(...)]` in `jaic-cli-args/src/cli.rs`
or `tuning.rs`; shared value parsing belongs to `jaic::flags`. `target_flags` and `import_dirs`
find options through `Spec::scan`, the scanner the parser itself uses, and pick them by `tag`
(`check`, `interp`, `deps`, `target`, `import`). `ArgumentValue::replace` preserves spaced/inline
syntax. Extend parser parity and boundary tests when adding syntax; do not add a second arity list.

An option whose value is a path or goes on to another program (`-linker`, `-sysroot`, `-compiler_rt`,
`-link_arg`) is declared `os`: its field is an `OsString` or `PathBuf`, `jaic` reads `args_os`, and the
value keeps its bytes. Every other argument must be UTF-8, and one that is not is an error naming the
options that take any bytes. `Spec::scan` treats a non-UTF-8 argument as a value, never an option.

`jaic-cli` remains a binary. It owns execution, native paths, code-generation conversion,
allocation counting and panic initialization. Its existing `dynamic-llvm`, `static-llvm`,
`lazy-llvm` and `mimalloc` features keep their meanings; none is a dependency of jacket's
argument inspection. The neutral layer depends only on `jaic` and `argdef`.

The Mac sanitizer trigger includes this crate's sources because its options and tuning
metadata affect native builds. Preserve that path in `tools/ci/ci_changes.py::NARROW`
when moving compiler argument code; the dependency graph alone does not bypass that filter.
