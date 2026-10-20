# Mutation testing

## What it is

[cargo-mutants](https://mutants.rs) changes one expression at a time (`+` to `-`, a comparison to its neighbour, a function body to `Default::default()`) and runs the tests. A mutant that no test fails is a place where the tests do not constrain behaviour. The weekly `mutants` workflow does this to the code where a wrong answer is silent: the type checker's calls, conversions and interval analysis, the interpreter's arithmetic and the parser.

## How it works

`.github/workflows/mutants.yml` (Saturdays, or by dispatch) installs the pinned cargo-mutants (`tools/ci/install_cargo_mutants.sh`, checksum from `.github/pins.json`) and runs eight shards of `cargo mutants -p jaic --shard k/8` over the files in its `SCOPE`. Each shard uploads `mutants.out` and prints the surviving mutants (`missed.txt`) in its job summary. The job does not fail on survivors or timeouts, only on a broken build or baseline: it is a report, not a gate.

Settings are in `.cargo/mutants.toml`: only `--lib` tests run (the unit tests; they build faster than the integration binaries and sit next to the code), and `Debug`/`Display` impls are not mutated.

`sema/reachability.rs` and `ide/values.rs` are left out: only the language server's `unreachable` test exercises what it fades, and cargo-mutants 27.1 does not run another package's tests against a mutant (`--test-package` is ignored once `-p` names the mutated package), so every mutant would survive. Moving a unit test of `warn_unreachable` into `jaic` would bring it in.

## How to change it

- Run it on one file: `cargo mutants -p jaic -f crates/jaic/src/ide/interval.rs`. Each mutant rebuilds `jaic`, about a minute, so use `--list` first and a small file. `-F 'regex'` narrows to mutants whose name matches.
- A surviving mutant is a missing test, an equivalent change, or dead code. Add a test that fails with the mutant (the report names the line and the replacement); if the code is unreachable, delete it. A mutant that cannot be told apart from the original can be skipped in place with `#[mutants::skip]` and a comment saying why; add the `mutants` crate only when one is needed.
- Widen the scope by adding a glob to `SCOPE` in the workflow.
- Timeouts count as caught (the mutant made the program loop), so a mutated loop condition is not a finding.

## Configuration

`.cargo/mutants.toml` (`additional_cargo_test_args`, `exclude_re`, `minimum_test_timeout`); the `scope` dispatch input.

## Dependencies

cargo-mutants (pinned in `.github/pins.json`, `cargo_mutants` and `cargo_mutants_sha256_linux_x64`).
