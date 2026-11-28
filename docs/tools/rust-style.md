# Rust style

`rustfmt` lays out lines; `tools/rust-spacing` adds the blank lines it never adds, so guard blocks, `let` runs and items stand apart.

## Running it

```sh
cargo fmt --all
cargo run -p rust-spacing            # every .rs file under crates/ and tools/rust-spacing/
cargo run -p rust-spacing -- --check # list files that would change; exit 1 (CI, hooks)
cargo run -p rust-spacing -- a.rs b/ # given files and directories only
```

`python3 tools/dev/dev.py fmt --lang rust` runs both. The pre-commit hook checks staged files, `tools/dev/pre_push_clippy.py check` checks the push, and `tools/dev/check_rust_format.py` is CI's check. Exit status 2 means a file is not valid Rust.

## Rules

The tool parses each file with `syn` and works from statement and item spans. It only inserts blank lines: code is never changed, an existing blank line stays, and a second run changes nothing. A comment or attribute directly above a statement belongs to it, so the blank line goes above the comment.

Between two statements of a block, a blank line goes in when:

- either is a multi-line `if`, `match`, `loop`, `while` or `for` (a guard block);
- either is a multi-line item (`fn`, `struct`, ...);
- the first is a `let` and the second is not, which ends a run of `let`s. A lone `let` stays attached to the block's last single-line statement (`let n = f(); n + 1`, or `return n;`);
- the second is the last statement of a block of three or more and is a `return`, `break` or `continue`.

Between two items of a file, `mod`, `impl`, `trait` or `extern` block, a blank line goes in when either is multi-line (attributes count) or one is an import (`use`, `extern crate`, `mod name;`) and the other is not. Two imports in a row are left to rustfmt's `group_imports`. Runs of one-line items (constants, aliases, trait method signatures) stay together, with or without doc comments and attributes on a `const`, `static` or `type`.

```rust
// before
fn load(path: &Path) -> Result<Config> {
    let text = fs::read_to_string(path)?;
    let mut config = parse(&text)?;
    if config.name.is_empty() {
        config.name = default_name(path);
    }
    validate(&config)?;
    log(&config);
    return Ok(config);
}

// after
fn load(path: &Path) -> Result<Config> {
    let text = fs::read_to_string(path)?;
    let mut config = parse(&text)?;

    if config.name.is_empty() {
        config.name = default_name(path);
    }

    validate(&config)?;
    log(&config);

    return Ok(config);
}
```

Statements inside macro invocations are token trees and are left alone, as are statements that share a line with another.

## Changing it

- Rules live in `Pass::block` and `Pass::items` in `tools/rust-spacing/src/lib.rs`; classification of a statement is `node_of_stmt`.
- Add the rule as a case in `tools/rust-spacing/src/tests.rs` first (`check(input, expected)` also asserts the output is a fixed point).
- Gotcha: spans are 1-based lines and 0-based columns, from `proc-macro2`'s `span-locations`. Never insert inside a statement's own span.
- After a rule change, run the tool over the workspace and commit the result on its own (`style(rust): ...`).

## Configuration

`rustfmt.toml` at the root. Flags: `--check`. No environment variables. The hook skips the check with `JAI_HOOK_SKIP=spacing`.

## Dependencies

`syn` (parsing, spans) and `proc-macro2` with `span-locations`, both pinned in `tools/rust-spacing/Cargo.toml`.
