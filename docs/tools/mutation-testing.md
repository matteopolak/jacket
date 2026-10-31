# Mutation testing

[cargo-mutants](https://mutants.rs) changes one expression at a time (`+` to `-`, a comparison to its neighbour, a function body to `Default::default()`) and runs the tests. A mutant no test fails is a place where tests don't constrain behaviour. The weekly `mutants` workflow does this to code where a wrong answer is silent: the type checker's calls, conversions and interval analysis, the interpreter's arithmetic and the parser.

## How it works

`.github/workflows/mutants.yml` (Saturdays, or by dispatch) installs the pinned cargo-mutants and runs eight shards of `cargo mutants -p jaic --shard k/8` over the files in its `SCOPE`. Each shard uploads `mutants.out` and prints the survivors in its summary. It is a report, not a gate: only a broken build or baseline fails it.

Each mutant runs a fast subset of the `jaic` tests (the unit tests plus integration binaries that check programs in-process), about 30 seconds instead of a minute. Code only exercised by a test outside the subset reports every mutant as missed, so give it a unit test next to it or add the binary to the subset. `.cargo/mutants.toml` holds the mutant filters (`Debug`/`Display` impls are not mutated).

## How to change it

```sh
cargo mutants -p jaic -f crates/jaic/src/ide/interval.rs --list
cargo mutants -p jaic -f crates/jaic/src/ide/interval.rs -F 'regex'
```

- A surviving mutant is a missing test, an equivalent change, or dead code. Add a test that fails with the mutant, or delete the dead code. A mutant that can't be told from the original goes in `exclude_re` in `.cargo/mutants.toml` with a comment saying why.
- Widen the scope with a glob in `SCOPE`; extend the subset with a test binary name in the `for name in` list.
- Deep run: dispatch `mutants` with `scope` set to the file and `full_suite: true`.
- Timeouts count as caught.
- `sema/reachability.rs` and `ide/values.rs` are left out because cargo-mutants doesn't run another package's tests against a mutant.

## Configuration

`.cargo/mutants.toml` (`exclude_re`, `minimum_test_timeout`); the `scope` and `full_suite` dispatch inputs. cargo-mutants is pinned in `.github/pins.json`.
