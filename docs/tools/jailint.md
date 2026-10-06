# jailint (Jai linter)

## What it is

`jailint` reports common mistakes and unidiomatic code in Jai programs, like clippy does for Rust, and fixes the ones it can. Its rules run on the program after `jaic` has type-checked it, so they know each expression's type and what each name refers to. The same findings appear in editors through `jailsp`, as diagnostics with quick fixes.

```sh
cargo build -p jailint --release
jailint src/                  # lint every .jai file under src/
jailint main.jai --fix        # apply the fixes that are safe to apply
jailint -D warnings stdlib    # fail on any finding (CI)
jailint --list                # the rules and their default levels
```

Output follows rustc's layout:

```text
warning[index_only_loop]: `i` counts through `names` to index it
  --> list.jai:21:5
   |
21 |     for i: 0..names.count - 1 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: loop over the elements: `for names`, with `it` and `it_index`
   |
21 ~     for names {
22 ~         if it_index > 0 print(", ");
23 ~         print("%", it);
   |
   = note: `index_only_loop` is `warn` by default
```

Options: `--fix`, `--config <file>`, `-A`/`-W`/`-D <rule>` (allow, warn, deny; `all` names every rule, `-D warnings` turns every warning into an error), `-I <dir>` (import directory), `-j <n>` (programs compiled at once), `--list`, `--color auto|always|never`, `-v` (each program as it is compiled, and programs that did not compile completely).

Exit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a usage or configuration error. A summary goes to stderr.

## Rules

| Rule | Default | Fix | Finds |
| --- | --- | --- | --- |
| `bool_comparison` | warn | yes | `x == true`, `x != false`, `false == x` |
| `defer_in_loop` | warn | no | a `defer` in a loop body that cleans up something from outside the loop |
| `float_equality` | allow | no | `==` or `!=` between two computed floats |
| `format_arg_count` | deny | no | a format string that uses more or fewer arguments than the call passes |
| `index_only_loop` | warn | yes | `for i: 0..xs.count-1` where `i` only indexes `xs` |
| `lossy_xx` | allow | yes | `xx` that narrows a number to a smaller type |
| `manual_index_counter` | warn | yes | a counter kept next to a `for` loop that always equals `it_index` |
| `redundant_cast` | warn | yes | a cast to the type the value already has |
| `shadowed_it` | warn | no | a nested `for` hides an `it` the enclosing loop still uses |
| `unused_import` | warn | yes | an `#import` nothing in its scope uses |
| `unused_parameter` | warn | no | a parameter the procedure never uses |
| `unused_variable` | warn | yes | a local variable that is never used |

Each rule's module (`crates/jailint/src/rules/<rule>.rs`) starts with a doc comment that explains why the rule exists and exactly when it fires. `tests/lint/<rule>/` has code it fires on (`bad.jai`, with the expected output in `bad.expected` and the fixed code in `bad.fixed.jai`) and code it must not fire on (`good.jai`).

**`bool_comparison`.** `if ready == true` → `if ready`; `x == false` → `!x`. Only when the other side is a non-constant `bool`, so integers and types with `operator ==` are left alone.

**`defer_in_loop`.** A loop body's scope ends every iteration, so this frees `buffer` after the first file:

```jai
buffer := alloc(SIZE);
for files {
    defer free(buffer);
    read_into(buffer, it);
}
```

It fires only when everything the deferred statement uses was declared before the loop and is not mentioned in the body before the `defer`. That leaves out the usual per-iteration pairs (`lock(*m); defer unlock(*m);`, `f := open(it); defer close(f);`).

**`float_equality`** (allow). `if total == expected` with two computed floats. Comparisons with a constant (`x == 0`), the NaN test `x != x` and comparisons inside `operator ==` are not reported. Off by default: exact comparison is often deliberate ("did this value change"). On the stdlib it finds only such cases.

**`format_arg_count`** (deny). `print("% is %\n", name)` prints an error marker in place of the second value at run time, and `print("done\n", n)` drops `n` silently. A procedure counts as print-like when a `string` parameter is followed by a variadic `..Any` and its body passes both to one call, as `print`, `sprint`, `tprint`, `log`, `print_to_builder` and user wrappers do. A `greet :: (name: string, extras: ..Any)` that uses them apart is not a format. Only literal format strings are read. Calls that spread an array (`..args`) or name arguments are skipped. Directives are read as `Basic` reads them: `%`, `%N`, `%00`, and `\%` for a literal percent. It is `deny` because the mistake is always visible at run time.

**`index_only_loop`.** `for i: 0..xs.count - 1` whose `i` only indexes `xs` becomes `for xs` with `it`. When elements are written (`xs[i].x += 1`) it becomes `for *xs` with `it`. Other uses of the index become `it_index`. It stays quiet when:

- the loop could change `xs` itself (assignment, `array_add`, taking its address, `remove`);
- it walks a grid (`for c: 0..xs[i].count - 1` inside);
- the index is used with other arrays or computed with inside an index (`ys[i]`, `xs[i + 1]`);
- elements are filled from the index (`xs[i] = i * i`).

**`lossy_xx`** (allow). `small: u16 = xx big` with `big: s64`, `n: int = xx 2.75`, `f32: float32 = xx f64`. The fix spells the conversion out as `cast(T)`. Values masked, shifted or reduced with `%` to fit, constants, widening and enum targets are skipped. Off by default: `xx` for narrowing is common, deliberate Jai style (graphics APIs take `s32`/`u32`).

**`manual_index_counter`.**

```jai
n := 0;
for names {
    print("% %\n", n, it);
    n += 1;
}
```

It fires when `n := 0` comes right before a forward `for` over an array, the body ends with `n += 1` (and has no `continue`) or starts with `defer n += 1;`, and `n` is not used after the loop. The fix removes the counter and uses `it_index`.

**`redundant_cast`.** `cast(s32) x` where `x: s32`, or `v: u8 = xx w` where `w: u8`. Aliases such as `c_long` can be `s64` on one platform and `s32` on another, so equal types are not enough. The value must be declared with the same type spelling as the cast (`int`≡`s64`, `float`≡`float32`), or be another cast to it. Polymorphic bodies, macros and cast flags are skipped.

**`shadowed_it`.** An unnamed `for` inside another hides the outer `it`, and the outer loop uses `it` again after the inner loop. Copying the outer `it` to a name first (`row := it;`), the usual idiom, is not reported. Uses are told apart by what the compiler resolved each `it` to.

**`unused_import`.** Unnamed imports count as used when any lookup went through them during checking, which covers operators and `for_expansion`. As a guard for unchecked code, an identifier anywhere in the importing module's files that the module exports also counts. That includes files `#load`ed under an `#if` for another platform. Named imports (`M :: #import "X"`) count as used when `M` is used or written anywhere in the module. These imports are not reported:

- imports with module parameters;
- imports of modules with `#program_export`;
- unnamed imports in a program that did not compile completely.

**`unused_parameter`.** Only procedures whose every caller is in view. That means named procedures that are only ever called (never taken as a value, which could make them a callback), and that are:

- called at least once;
- not overloaded;
- not exported from a module;
- not `#c_call`, `#foreign`, `#expand`, an operator or `for_expansion`;
- without notes.

Empty bodies (stubs), bodies with `#insert`, and parameters whose type has a `$` (the argument settles a polymorphic type) are skipped. Prefix a name with `_` to keep a parameter on purpose.

**`unused_variable`.** Locals in bodies that checked cleanly, decided by what the compiler resolved names to across every polymorph instance. A variable whose name appears anywhere else in its block is never reported, which covers code an `#if` left out and `#asm`. Backtick declarations, `using` and `_`-prefixed names are skipped, and so are blocks with `#insert` (inserted code may use any name) and macro bodies. The fixes:

- remove a pure single declaration;
- drop trailing results of a call (`value, found := f();` → `value := f();`);
- drop a name from `a, b: T;`;
- otherwise rename to `_`.

### Measured on this repository

Every finding below was checked by hand. Findings that turned out wrong were fixed in the rules (the exceptions listed above), not suppressed. The repository's own Jai code now has none at the default levels:

- **stdlib, examples and `tools/jaifmt`**: 84 findings were fixed in the stdlib: 74 unused variables (mostly extra results nobody read), 5 unused imports, 4 index loops and 1 shadowed `it`. `float_equality` and `lossy_xx` would add 6 and 8 hits, all deliberate (exact comparisons in tests and sorting, `xx` from floats to pixel coordinates).
- **`tests/corpus/positive`**: no findings.
- **Upstream corpus** (`tools/upstream-cases.json` entry points): 127 findings, all genuine. `lossy_xx` adds 23 deliberate narrowings.

## How it works

1. **Plan** (`driver.rs`). Paths become `.jai` files, minus `exclude`. A file that another listed file `#load`s is compiled as part of it. `module.jai`, or a file directly in an import directory, is a module: it is compiled by importing it from an empty program, so every procedure is checked whether or not anything calls it. Anything else is a program.
2. **Compile** with lint facts on (`facts::enable` sets `IdeFacts::lint`). On top of the editor facts (`jaic::sema::ide`), the checker then records:
   - each expression's type and constness (`exprs`; marked conflicting when polymorph instances disagree);
   - each cast's source and target types (`casts`);
   - every entity a name resolved to (`used`);
   - which import answered each lookup (`used_imports`).

   Compile-time code runs in the sandboxed host with a block budget. Workspaces a metaprogram creates are compiled too: a `build.jai` that adds `src/main.jai` gets `src/` linted. This uses `jaic::build::WorkspaceObserver`, which hands each workspace's compiler to jailint. Relative paths a metaprogram names resolve against the root file's directory.
3. **Lint** (`lib.rs`, `lint_files`). Per file:
   - re-parse and lex;
   - build the in-source suppressions;
   - collect procedure bodies with whether every instance checked to the end (`syntax::Cx::procs`);
   - run each enabled rule.

   Rules walk the syntax tree (`syntax::walk`) and ask the facts about types and names. Where the compiler did not look (an `#if` branch left out, a body that failed), they fall back to the text and stay quiet.
4. **Report**. Findings get their level, suppressed ones are dropped, and the rest are rendered (`render.rs`). `--fix` applies each machine-applicable fix whose edits do not overlap one already taken (`fix.rs`); run again for the rest.

A file compiled by several roots is reported once, from the first root. Within a root, a workspace's compile wins over the metaprogram's.

### In editors

`jailsp` lints each open document that parses. It uses the same cached, type-checked compile as hover and inlay hints, so an edit costs one compile however many features ask. Findings are diagnostics with the rule as `code` and `jailint` as `source`, and `deny` rules are errors. Machine-applicable fixes are `quickfix` code actions on the finding's range. `format_arg_count` is left to jailsp's own format-string diagnostics, which work without type checking. See [the language server](../compiler/language-server.md#lints-and-quick-fixes).

## Suppression

```jai
// jailint: allow(unused_variable)        covers the next line with code
x := compute(); // jailint: allow(all)     after code: covers its own line
// jailint: allow-file(float_equality)     anywhere: covers the whole file

callback :: (event: int, data: *void) {
    print("%\n", event);
} @jailint_allow(unused_parameter)        // after a declaration: covers all of it
```

Several rules can be listed, separated by commas, and `all` stands for every rule. A suppression only silences findings. A name starting with `_` is how code says "unused on purpose" for variables and parameters.

## Configuration

`jailint.toml`, found by walking up from the first path (or `--config`):

```toml
# Globs relative to this file: `*`, `**`, `?`; a directory pattern covers what is under it.
exclude = ["tests/corpus/negative/**", "generated/**"]

[rules]
float_equality = "warn"
unused_parameter = "allow"
```

Only this subset of TOML is read. An unknown key, rule or level is an error, so a typo cannot silently leave a rule on. Command-line `-A`/`-W`/`-D` override the file.

Settings live in their own file rather than `jaifmt.toml` because the formatter is written in Jai and parses its own file. Keeping the two separate means neither tool has to accept the other's keys or reject them as typos.

The repository's `jailint.toml` excludes the negative compiler cases and the formatter's golden inputs. CI runs `jailint -D warnings stdlib examples tools/jaifmt tests/corpus/positive` (see [continuous integration](continuous-integration.md)).

## How to change it

**Add a rule:**

1. Create `crates/jailint/src/rules/<name>.rs`. Start it with a doc comment saying what it finds, why it matters and exactly when it fires. Then write `pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>)`.
2. Register it in `rules/mod.rs` (`mod` line and a `RULES` entry with the default level and a one-line summary).
3. Add `tests/lint/<name>/bad.jai` and `good.jai` (each a whole program with `main`), then generate the expected files with `JAILINT_BLESS=1 cargo test -p jailint`. Read them before committing.
4. Run it over the repository and the upstream corpus with `-W <name>` and check every hit. A rule that is wrong once in a while costs more trust than it earns.
5. Add it to the table above and to the changelog.

Inside a rule:

- `cx.procs` lists the procedure bodies. Use `p.clean` (every instance checked to the end) or `p.typed()` (clean, not polymorphic, not a macro) before trusting facts.
- `cx.ty(expr)` and `cx.constant(expr)` give recorded types and constness.
- `cx.entities(span)` gives what a name resolved to, and `cx.compiler.is_used(entity)` whether anything used it.
- `cx.mentions(name, start, end, except)` is the textual guard for unchecked code.
- `syntax::walk` visits statements and expressions with their ancestors. It does not enter nested procedures or struct bodies.

A fix is a list of byte edits. Mark it `machine_applicable` only when applying it cannot change behaviour, and make the result compile (the `fixed_cases_are_clean` test lints `bad.fixed.jai` again).

**New facts.** If a rule needs something the checker does not record, add it to `IdeFacts` (`crates/jaic/src/sema/ide.rs`) behind `lint`, so normal compiles do not pay for it.

Gotchas:

- Facts are recorded only for files under the root's directory (the `IdeFacts` prefix), so module files reached from a program are not linted from that program.
- `ide_check_all` checks bodies nothing called. A body that failed in any polymorph instance is not clean.
- Compile-time code runs in the sandbox. A metaprogram that needs a foreign call the sandbox lacks (running a process, `localtime`) stops there. Its root is reported with `-v`, and only what was compiled is linted.

## Configuration reference

| Setting | Where | Effect |
| --- | --- | --- |
| `exclude` | `jailint.toml` | glob patterns of files and directories not to lint |
| `[rules] <rule> = "allow" \| "warn" \| "deny"` | `jailint.toml` | a rule's level |
| `-A`/`-W`/`-D <rule>`, `-D warnings` | command line | override levels; make warnings errors |
| `-I <dir>` | command line | extra import directory (searched after the root's `modules/`, before the stdlib) |
| `JAIC_STDLIB` | environment | stdlib directory (default: `stdlib/` next to the executable, else the checkout's) |
| `JAILINT_BLESS=1` | environment, tests | rewrite `tests/lint/*/bad.expected` and `bad.fixed.jai` |
| `BLOCK_BUDGET` | `driver.rs` | basic blocks compile-time code may run per root |

## Dependencies

- `crates/jaic`: parser, lexer, the type checker with IDE and lint facts (`sema::ide`), the workspace observer (`build::WorkspaceObserver`) and the sandboxed compile-time host.
- `crates/jai-language-server` depends on `jailint` to publish lints. Its format-string features share `jailint::format_string`.
- No other crates.
