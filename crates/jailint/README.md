# jailint

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

The linter for [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) in the [Jacket](../../README.md) toolchain. It finds likely mistakes and unidiomatic code, and fixes what it safely can.

```sh
jacket lint src/                  # lint every .jai file under src/
jacket lint src/ --fix            # apply the safe fixes
jacket lint -D warnings src/      # treat every warning as an error, for CI
```

`jacket lint` runs jailint with exactly these arguments; the standalone `jailint` binary takes the same ones.

jailint runs after the compiler has type-checked your program, so its rules know the type of every expression and what every name refers to. That keeps false positives rare: a rule about unsigned values only fires on values that really are unsigned.

```text
warning[index_only_loop]: `i` is only used to index `names`
  --> main.jai:4:5
   |
 4 |     for i: 0..names.count - 1 print("%\n", names[i]);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: loop over the elements: `for names` and `it`
   |
 4 ~     for names print("%\n", it);
   |
```

## What it catches

There are 33 rules; `jacket lint --list` prints them. They fall into a few groups:

- **Likely bugs:** a comparison that is always true (`u >= 0` on an unsigned value), shift precedence that differs from C (`1 << n - 1`), identical `if` branches, a `while` whose condition never changes, a `defer` inside a loop that frees something from outside it, removing from an array while a `for` walks it, a range that runs one past the end.
- **Things left unused:** variables, parameters, imports, and the result of a call that only computes a value (`trim(line);`).
- **Simpler ways to say it:** `for i: 0..xs.count-1` that only indexes `xs`, `x = x + 1`, `x == true`, a cast to the type a value already has.

Two rules are off by default because the code they flag is often deliberate: exact float comparison (`float_equality`) and narrowing with `xx` (`lossy_xx`).

## Install

jailint comes with every Jacket install, next to `jaic`, `jailsp` and `jaifmt`:

```sh
brew install matteopolak/tap/jacket
```

The [main README](../../README.md#install) lists the other ways to install. In VS Code and Neovim, jailint's findings show up as you type, through [`jailsp`](../jai-language-server/README.md), and each fixable finding is a quick fix.

To build it from a checkout, run `cargo build -p jailint --release`.

## Configuration

Put a `jacket.toml` at your project root to change rule levels or skip files (a plain `jailint.toml` with the same keys, without the `[lint]` header, still works; `jacket config migrate` converts it):

```toml
[lint]
exclude = ["generated/**"]

[lint.rules]
float_equality = "warn"
unused_parameter = "allow"
```

The levels are `allow`, `warn` and `deny`. On the command line, `-A`, `-W` and `-D <rule>` override the file, and `all` names every rule.

To silence one finding, write a comment:

```jai
// jailint: allow(unused_variable)       the next line
// jailint: allow-file(float_equality)    the whole file
```

A variable or parameter whose name starts with `_` is unused on purpose and is never reported.

## Your own procedures

Some rules need to know what a procedure does: `unused_result` needs to know a call only computes its result, and `defer_in_loop` needs to know a call frees something. jailint learns this from a note on the declaration, never from the procedure's name:

```jai
shout :: (s: string) -> string {
    return to_upper_copy(s);
} @UseResult
```

The standard library's procedures already carry these notes. Add one to your own procedures to get the same checks. The [reference](../../docs/tools/jailint.md#declaration-notes) lists all five.

## Other platforms

Platform-specific code can be checked from any machine: `jacket lint -os windows -cpu arm64 src/` type-checks `src/` as it would compile on Windows on arm64. If some modules only build on their own OS, `--allow-incomplete` reports them without failing the run.

The exit status is 0 when nothing at level `deny` was found, 1 when something was, and 2 on a usage or configuration error. `jacket lint --help` lists every option.

## More

- [jailint reference](../../docs/tools/jailint.md): every rule in detail, target checking, and how to add a rule
