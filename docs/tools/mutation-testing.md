# Mutation testing

## What it is

[cargo-mutants](https://mutants.rs) changes one expression at a time (`+` to `-`, a comparison to its neighbour, a function body to `Default::default()`) and runs the tests. A mutant that no test fails is a place where the tests do not constrain behaviour. The weekly `mutants` workflow does this to the code where a wrong answer is silent: the type checker's calls, conversions and interval analysis, the interpreter's arithmetic and the parser.

## How it works

`.github/workflows/mutants.yml` (Saturdays, or by dispatch) installs the pinned cargo-mutants (`tools/ci/install_cargo_mutants.sh`, checksum from `.github/pins.json`) and runs eight shards of `cargo mutants -p jaic --shard k/8` over the files in its `SCOPE`. Each shard uploads `mutants.out` and prints the surviving mutants (`missed.txt`) in its job summary. The job does not fail on survivors or timeouts, only on a broken build or baseline: it is a report, not a gate.

Each mutant runs a **fast subset** of the `jaic` tests, chosen in the workflow's `Mutate` step: the unit tests (`--lib`) and the integration binaries that check programs in-process (`cast_shifts`, `literals`, `call_diagnostics`, ...). Running every `jaic` test costs about a minute per mutant (roughly four more hours per shard), the subset about 30 seconds. A file whose behaviour is only exercised by a test outside the subset reports every mutant as missed, so give such code a unit test next to it (`interp/arith/tests.rs` checks the register arithmetic against native Rust operations) or add the integration binary to the subset. Dispatch with `full_suite: true` and a small `scope` for a deep run of one file. `.cargo/mutants.toml` holds the mutant filters only (`Debug`/`Display` impls are not mutated).

`sema/reachability.rs` and `ide/values.rs` are left out: only the language server's `unreachable` test exercises what it fades, and cargo-mutants 27.1 does not run another package's tests against a mutant (`--test-package` is ignored once `-p` names the mutated package), so every mutant would survive. Moving a unit test of `warn_unreachable` into `jaic` would bring it in.

## How to change it

- Run it on one file: `cargo mutants -p jaic -f crates/jaic/src/ide/interval.rs`. Each mutant rebuilds `jaic`, about a minute, so use `--list` first and a small file. `-F 'regex'` narrows to mutants whose name matches.
- A surviving mutant is a missing test, an equivalent change, or dead code. Add a test that fails with the mutant (the report names the line and the replacement); if the code is unreachable, delete it. A mutant that cannot be told apart from the original is listed in `exclude_re` in `.cargo/mutants.toml` with a comment saying why (rotation's `|`, the type-only early return and `fold_cast`'s guards are there); `#[mutants::skip]` would need the `mutants` crate. `sema/convert/casts.rs` is tested by `crates/jaic/tests/cast_semantics.rs` and `interp/arith.rs` by `interp/arith/tests.rs`; both ended with only those listed survivors.
- Widen the scope by adding a glob to `SCOPE` in the workflow; extend the subset by adding a test binary name to the `for name in` list.
- Deep run of a hot file: dispatch `mutants` with `scope` set to the file and `full_suite` true.
- Timeouts count as caught (the mutant made the program loop), so a mutated loop condition is not a finding.

## Configuration

`.cargo/mutants.toml` (`exclude_re`, `minimum_test_timeout`); the `scope` and `full_suite` dispatch inputs.

## Dependencies

cargo-mutants (pinned in `.github/pins.json`, `cargo_mutants` and `cargo_mutants_sha256_linux_x64`).
