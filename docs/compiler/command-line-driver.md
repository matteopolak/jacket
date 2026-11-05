# Compiler command-line driver

`jaic-cli-args` owns the compiler's command-line model and authoritative option tables.
The actual `jaic` parser and tooling such as jacket inspect the same spellings and arity.
The neutral crate has no execution backend, allocator, crash hook or native library discovery.

`jaic_cli_args::arguments::target_flags` returns target selection, consumed option-value
positions and arguments for host discovery. It consumes unrelated option values before
looking for another flag, accepts inline values, and preserves plugin/metaprogram/program
boundaries. Missing-value errors reuse the compiler's value descriptions. Jacket uses those positions to resolve paths relative to the user's directory.

```rust
let parsed = jaic_cli_args::arguments::target_flags(&["-os=wasm".into()])?;
assert!(parsed.host.is_empty());
```

Change option spellings/arity in `jaic-cli-args/src/options.rs` or its `tuning.rs`; shared
value parsing belongs to `jaic::flags`. Both the actual parser and metadata inspector use
`split_option` for inline spelling. `ArgumentValue::replace` preserves spaced/inline syntax.
Extend parser parity and boundary tests when adding syntax; do not add a second arity list.

`jaic-cli` remains a binary. It owns execution, native paths, code-generation conversion,
allocation counting and panic initialization. Its existing `dynamic-llvm`, `static-llvm`,
`lazy-llvm` and `mimalloc` features keep their meanings; none is a dependency of jacket's
argument inspection. The neutral layer depends only on `jaic`.

The Mac sanitizer trigger includes this crate's sources because its options and tuning
metadata affect native builds. Preserve that path in `tools/ci/ci_changes.py::NARROW`
when moving compiler argument code; the dependency graph alone does not bypass that filter.
