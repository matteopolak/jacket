# Environment variables (the registry)

## What it is

Every environment variable the Rust tools read is declared once, in `jacket_config::env` (`crates/jacket-config/src/env.rs`): its name, meaning, what happens when it is unset, and which programs read it. [The reference page](../reference/environment-variables.md) is generated from that table.

`jacket-config` is where it lives because every tool already depends on it (`jaic`, `jailint`, `jailsp`, `jacket`, the test harnesses and the fuzz harness), and `jacket` does not depend on `jaic`.

## How it works

A variable is a constant, read through methods that name the intent:

```rust
use jacket_config::env;

let dir = env::STDLIB.non_empty();          // set and not empty
let seed = env::SYM_SCRAMBLE.get();          // Option<String>
if env::DSYM.is_set() { .. }
```

`get` and `os` return the value, `non_empty` treats `VAR=` as unset (PowerShell 7.5 passes an empty value for an unset one), and `is_set` asks only whether it exists. Code that is handed a name to look up, such as the `env` closure the colour and status rules take so tests can supply their own, calls `env::lookup(name)`, which panics on a name that is not declared. `env::for_program(name)` is for the one case where the name belongs to the program being compiled (`getenv` in compile-time code).

Two things keep the table complete:

- clippy's `disallowed-methods` (root `clippy.toml`) rejects `std::env::var`, `var_os`, `vars` and `vars_os` everywhere except `env.rs`, which carries the one `#![allow(clippy::disallowed_methods)]`. The two build scripts that read cargo's own variables (`crates/jaic-cli/build.rs`, `crates/jai-wasm/build.rs`) allow it too: they cannot depend on the crate they build.
- A test in `env.rs` scans the Rust sources for quoted `"JAIC_*"`, `"JACKET_*"` and `"JAI_*"` names (a `Command::env("JAIC_STDLIB", ..)` in a test, a message) and fails on one that is not declared, so a typo in a place that is not a read still fails.

Variables that are not ours (`PATH`, `NO_COLOR`, `CI`, ...) are declared too, in the *Standard variables* group, so the page shows everything the tools consult.

## How to change it

To add a variable, add an entry to `vars!` in `env.rs` with its group (`Settings` for users, `Handoff` for one tool configuring another, `Testing`, `Standard`), readers, default and one-sentence summary, then read it through its constant. Regenerate the page:

```sh
PYTHONPATH=tools python3 tools/generators/generate_reference_docs.py
```

CI runs the generator with `--check` (`tools/generators/check_generated.py`), so a stale page fails. The page is rendered by `crates/jacket-config/examples/env_reference.rs`.

Gotchas:

- Setting a variable (`std::env::set_var`) is not covered by the lint; it is `unsafe` in edition 2024 and each use already carries a safety comment.
- Stdlib programs and CI scripts read their own variables (`JAI_IO_*`, `JACKET_PG_*`, `JAI_INSTALL_DIR`, ...) from Jai and shell code; the registry covers only what the Rust binaries read.

## Configuration

None: the table is the configuration.

## Dependencies

`jacket-config` (no dependencies of its own for this module); clippy for the lint; `tools/generators/generate_reference_docs.py` for the page.
