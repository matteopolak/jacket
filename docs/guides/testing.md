# Testing

Tests are procedures with a `@test` note. `jacket test` finds them, builds them once and runs each in its own process, in parallel.

## A first project

Make a library and give it a function worth testing:

```sh
jacket new slugs --lib
cd slugs
```

```jai
// slugs/module.jai
#import "Basic";
#import "String";

// Turns a title into a URL slug: lowercase words joined by `-`.
//
// ```jai
// assert(slugify("Hello, World!") == "hello-world");
// ```
//
// - title: the text to convert.
slugify :: (title: string) -> string {
    builder: String_Builder;
    pending_dash := false;

    for 0..title.count - 1 {
        c := title[it];
        if c >= #char "A" && c <= #char "Z" c += 32;

        if (c >= #char "a" && c <= #char "z") || (c >= #char "0" && c <= #char "9") {
            if pending_dash && builder_string_length(*builder) > 0 append(*builder, "-");
            pending_dash = false;
            append(*builder, string.{1, *c});
        } else {
            pending_dash = true;
        }
    }

    return builder_to_string(*builder);
}
```

Put tests in `tests/`:

```jai
// slugs/tests/slugify.jai
#import "Basic";
#import "Extensions/Testing";
#import "slugs";

lowercases_and_joins :: () {
    got := slugify("Hello World");
    defer free(got);
    expect_equal(got, "hello-world");
} @test

drops_punctuation :: (t: *Test) {
    got := slugify("Rock & Roll!");
    defer free(got);
    log(t, "slug: %", got);
    expect_equal(got, "rock-roll");
} @test

keeps_digits :: () {
    got := slugify("Top 10 Tips");
    defer free(got);
    assert_that(got, starts_with("top-10"));
} @test
```

```sh
jacket test
```

```text
    Starting 4 tests
        PASS [   0.057s] lowercases_and_joins
        PASS [   0.057s] drops_punctuation
        PASS [   0.057s] keeps_digits
        PASS [   0.221s] doc: slugs.slugify#1
     Summary [   0.221s] 4 tests run: 4 passed
```

The fourth test is the example in the doc comment; see [Doc tests](#doc-tests).

Each test runs with a tracking allocator, so a test that allocates and doesn't free fails. Drop the `defer free(got)` lines above and all three fail:

```text
--- lowercases_and_joins (FAIL) ---
  module.jai:28:
    leaked 11 bytes in 1 allocation made here in slugify
    28 | return builder_to_string(*builder);
  rerun: jacket test --exact 'lowercases_and_joins'
```

Put `@allow_leaks` after the body to turn the check off for one test.

The same layout works for a program. With `main.jai` and `math.jai` in the project folder, `tests/math.jai` can `#load "../math.jai";` and test `double` directly. For a project that keeps its modules in a folder, set `import_path = ["src"]` under `[project]` in `jacket.toml` and the tests can `#import` them by name.

## Writing tests

A test is a procedure at file level with `@test` after its body. It takes nothing, or a `*Test` when it wants `log`, `skip`, `temp_dir` or subtests. The procedure's name is the test's name.

Checks come in two kinds. `expect*` reports a failure and lets the test go on. `require*` reports it and returns from the test, for when the rest can't work without it.

| Check | Holds when |
| --- | --- |
| `expect(condition, message)` | `condition` is true |
| `expect_equal(got, want)` | equal by value: numbers by value, strings by bytes, arrays and structs member by member |
| `expect_near(got, want, tolerance)` | floats are within `tolerance` |
| `expect_null(p)`, `expect_not_null(p)` | a pointer is / isn't null |
| `assert_that(value, matcher)` | the value satisfies the matcher |
| `expect_snapshot(t, value)` | the value matches its stored snapshot |
| `require`, `require_equal`, `require_near`, `require_not_null`, `require_that` | as above, then return |

A failure names the line and shows what differed:

```text
--- breaks_on_purpose (FAIL) ---
  tests/more.jai:28:
    got:  "one-two"
    want: "one_two"
              ^ first difference at byte 3
    28 | expect_equal(got, "one_two");
  rerun: jacket test --exact 'breaks_on_purpose'
```

### Matchers

`assert_that(value, matcher)` reads like a sentence and says what it wanted:

```jai
assert_that(count, is(3));
assert_that(name, all_of(starts_with("jac"), has_length(6)));
assert_that(ports, has_item(between(8000, 8999)));
require_that(users, each(not_null()));
```

```text
--- bad_matcher (FAIL) ---
  tests/fails.jai:19:
    expected "top-ten" to have length 5
    19 | assert_that(got, all_of(starts_with("top"), has_length(5)));
```

| Matcher | Holds when |
| --- | --- |
| `is(want)` | equal, as `expect_equal` compares |
| `not(m)`, `all_of(m...)`, `any_of(m...)` | `m` doesn't hold / all hold / one holds |
| `greater_than(v)`, `at_least(v)`, `less_than(v)`, `at_most(v)` | the value orders that way against `v` |
| `between(low, high)` | `low <= value <= high` |
| `near(want, tolerance)` | within tolerance |
| `contains(part)` | a string has the substring, or an array the element |
| `starts_with(x)`, `ends_with(x)` | a string or array begins / ends with `x` |
| `is_empty()`, `has_length(n)` | a string or array has no / `n` elements |
| `has_item(m)`, `each(m)` | some / every element matches `m` |
| `is_null()`, `not_null()` | a pointer is / isn't null |
| `is_true()`, `is_false()` | a `bool` has that value |
| `of_type(T)` | the value's type is exactly `T` |

### Output, files and cleanup

`log(t, "step % of %", 1, 3)` prints a line that `jacket test` shows only when the test fails. `temp_dir(t)` is a fresh folder for the test's files; it is deleted after a pass and kept, with its path printed, after a failure. `cleanup(t, callback, data)` runs `callback(data)` when the test ends. `skip(t, "reason")` ends the test as skipped.

### Many inputs

`@cases(LIST)` runs a test once per entry, each as a subtest named after its values. A struct entry is spread over the arguments. The list is a constant at file level whose name has no spaces:

```jai
Case :: struct { title: string; want: string; }
TITLES :: Case.[
    .{ "Hello World", "hello-world" },
    .{ "  padded  ", "padded" },
    .{ "A--B", "a-b" },
];

slugifies :: (title: string, want: string) {
    got := slugify(title);
    defer free(got);
    expect_equal(got, want);
} @test @cases(TITLES)
```

```text
$ jacket test --list
slugifies: test
slugifies/Hello World,hello-world: subtest
slugifies/  padded  ,padded: subtest
slugifies/A--B,a-b: subtest
```

Run one case with `jacket test --exact 'slugifies/A--B,a-b'`. A failure inside a case names it:

```text
  tests/fails.jai:9 (in bad_case/c):
    got:  1
    want: 3
  failed subtests: bad_case/c
  rerun: jacket test --exact 'bad_case/c'
```

`@property` runs a test on generated arguments, 100 times by default (`@property(500)`). Integers, floats, bools, strings, enums, arrays and structs of those all work:

```jai
has_no_uppercase :: (title: string) {
    got := slugify(title);
    defer free(got);
    for 0..got.count - 1 expect(got[it] < #char "A" || got[it] > #char "Z");
} @test @property
```

A failing input is shrunk to a small one, and the report gives the seed:

```text
--- bad_property (FAIL) ---
  tests/fails.jai:12:
    property failed on trial 4 (seed 873921072)
    counterexample: bad_property(10)
    shrunk from:    bad_property(15) in 3 steps
    rerun: jacket test --seed 873921072 --exact 'bad_property'
```

`--seed N` replays the same inputs on any machine. Without it, one seed is drawn per run. `assume(condition)` discards a trial that doesn't suit the test, and `@generate(NAME)` points at a `(rng: *Rng) -> T` procedure for values of your own.

### Options

Notes after the body change how a test runs. Spaces inside a note's parentheses work (`@skip_on(.MACOS, reason = "needs a display")`), but jaic warns that the official compiler rejects them; write `reason="paths"` to stay portable. The `// test:` line below takes spaces without a warning.

```jai
reads_the_registry :: () {
    // ...
} @test @only(.WINDOWS) @timeout(30)

flaky_on_ci :: () {
    // ...
} @test @retries(2)

known_bug :: () {
    // ...
} @test @xfail(issue_142)
```

| Note | Effect |
| --- | --- |
| `@timeout(N)` | kill the test after N seconds |
| `@slow` | start it first |
| `@serial` | run it alone |
| `@retries(N)` | try again up to N times; a pass on a retry is reported `FLAKY` |
| `@only(.LINUX, .MACOS)` | run only there; `.WINDOWS`, `.WASM`, `.ANDROID`, `.X64`, `.ARM64`, `.INTERP`, `.NATIVE` also work |
| `@skip_on(.WINDOWS,reason="paths")` | skip there |
| `@needs(.NETWORK)` | skip unless run with `--needs network` |
| `@should_fail`, `@should_fail(word)`, `@should_fail("a", "b")` | must fail, with `word` (or any one of the quoted texts) in the message |
| `@xfail`, `@xfail(reason)` | a known failure; reported `XFAIL`, and passing is an error |
| `@exit(N)` | the program must exit with status N |
| `@allow_leaks` | don't fail on leaks |
| `@compile_time` | run inside the compiler, for code meant for `#run` |

A `// test:` line at the top of a file gives the same options to every test in it:

```jai
// test: only(.LINUX, .MACOS) timeout(10)
```

## Running tests

```sh
jacket test                         # everything
jacket test parse split             # names containing parse or split
jacket test --exact lowercases_and_joins
jacket test -E 'module(Sort) and not kind(property)'
jacket test --list                  # names and kinds; runs nothing
jacket test --watch                 # rerun when a .jai file changes
jacket test --fail-fast             # stop after the first failure
jacket test --interp                # in the compiler's interpreter
jacket test -- -O2                  # flags after -- go to the compiler
```

A filter is a part of a name. `--exact` takes whole names, and `name/sub` picks one subtest. `-E` takes an expression of `test(x)`, `module(x)`, `file(glob)` and `kind(k)`, joined with `and`, `or`, `not` and parentheses. `kind` is one of `unit`, `property`, `compile_time`, `snapshot`, `golden`, `compile_fail`, `doc`.

Statuses are `PASS`, `FAIL`, `SKIP`, `TIMEOUT`, `XFAIL`, `XPASS` and `FLAKY`. The exit status is 0 when everything passed or was skipped, 1 when a test failed, and 2 when nothing matched or the tests didn't compile.

`--format json` writes one JSON object per line, and `--junit FILE` writes JUnit XML, for editors and CI. `--coverage` reports which lines the tests ran.

Where `jacket test` looks is set by `[test] paths` in `jacket.toml`, else the nearest folder above the current one with a `tests/` folder or a `module.jai`. All options are in the [command-line reference](../reference/tools.md#testing).

## Golden output

A program that prints, or exits with a status, can be tested without any checks. Put a `.jai` file with a `main` in `tests/` and write the expected output beside it:

```jai
// slugs/tests/hello.jai
#import "Basic";
#import "slugs";

main :: () {
    print("%\n", slugify("Hello, Golden World"));
}
```

```text
$ cat tests/hello.stdout
hello-golden-world
```

The program is built and run alone. It passes when its exit status equals `hello.exit` (0 if there is none) and its standard output and error equal `hello.stdout` and `hello.stderr`. Only the files that exist are compared. `exits.exit` holds the status on one line, for a program that must fail:

```jai
// slugs/tests/exits.jai
#import "Basic";

main :: () {
    print("bad input\n");
    exit(3);
}
```

```text
$ cat tests/exits.exit
3
```

When the output changes on purpose, rewrite the files instead of editing them:

```sh
touch tests/hello.stdout       # a new golden test needs one expected file to exist
jacket test --accept hello
```

```text
--- hello (FAIL) ---
  standard output differs from hello.stdout:
  --- want
  +++ got
  + hello-golden-world
  - wrong
```

A bare `--accept` rewrites every golden file, so narrow it with a filter.

### Programs that must not compile

A file in `tests/compile_fail/` must fail to type-check, with exactly the diagnostics in `name.stderr`:

```jai
// slugs/tests/compile_fail/needs_text.jai
#import "slugs";

main :: () {
    slugify(42);
}
```

```sh
jacket test --accept needs_text     # writes needs_text.stderr
```

A first run writes a missing `.stderr` by itself and prints `CREATED` with its path, so `--accept` is for changing one that exists. When the `CI` environment variable is set, a missing file fails the test instead.

```text
needs_text.jai:4:13: error[call-mismatch]: in call to `slugify`: argument of type `s64` does not match parameter type `string`
        slugify(42);
                ^^
../../module.jai:11:12: note: `slugify` is declared here
    slugify :: (title: string) -> string {
               ^^^^^^^^^^^^^^^^^^^^^^^^^
help: to turn a number into text, format it: `tprint("%", value)` (from Basic)
```

The folder of the test is written as `$DIR` and the standard library as `$STDLIB` where they appear, so the file is the same on every machine. To check only that a message appears, put `// test: compile_fail("does not match")` at the top of the file and skip the `.stderr`.

## Snapshots

`expect_snapshot(t, value)` compares a value, printed one member per line, with a file next to the test:

```jai
snapshot_of_slug :: (t: *Test) {
    got := slugify("Snapshot Me");
    defer free(got);
    expect_snapshot(t, got);
} @test
```

The first run fails and writes the proposed file:

```text
--- snapshot_of_slug (FAIL) ---
  tests/more.jai:38:
    there is no snapshot tests/__snapshots__/more/snapshot_of_slug.snap yet
    the new one is in tests/__snapshots__/more/snapshot_of_slug.snap.new; `jacket test --accept` keeps it, `jacket test --review` goes through them
```

`jacket test --accept --exact snapshot_of_slug` keeps it. `jacket test --review` shows each pending change as a diff and asks whether to accept, reject or skip it. Commit the `.snap` files and ignore `*.snap.new`. Give a second snapshot in one test a name: `expect_snapshot(t, value, "color")`. Pointers print as `<pointer>`, so keep them out of snapshotted values.

## Doc tests

Every ```` ```jai ```` block in a doc comment is a test. `slugify` above has one, and it ran as `doc: slugs.slugify#1`. A block that is only statements becomes the body of a procedure that has `Basic` and your library in scope; a block with its own `main` is used as written. A failed `assert` fails the test:

```text
--- doc: slugs.slugify#1 (FAIL) ---
  the example was killed by signal 6 (SIGABRT)
  stderr:
    slugs/module.jai:7: assertion failed:
```

Add `#import "Extensions/Testing";` to the block to use `expect_equal` and get a readable difference:

```text
  the example exited with status 1
  stderr:
    slugs/module.jai:8: got:  "hello-world"
    want: "hello_world"
                ^ first difference at byte 5
```

The words after `jai` choose what is done with the block:

| Fence | The example is |
| --- | --- |
| ```` ```jai ```` | built and run; it must exit 0 |
| ```` ```jai,no_run ```` | only type-checked, for code that needs a file, the network or a device |
| ```` ```jai,compile_fail ```` | expected not to type-check |
| ```` ```jai,should_fail ```` | built and run; it must fail |
| ```` ```jai,ignore ```` | skipped |

`only(linux, macos)`, `skip_on(wasm)`, `needs(network)` and `timeout(5)` also work there, after a comma. To apply a note to every example of a library, put a `// test:` line in a comment of its own after the module docs at the top of `module.jai`, with a blank line on each side:

```jai
// test: skip_on(.WASM, reason = "uses the file system")
```

`jacket test --doc` runs only doc tests and `--no-doc` leaves them out. Writing examples that read well on the documentation site is covered in [Documenting a library](documenting.md).

`jacket test --doc --check-only --os windows` type-checks every example for another OS without building or running anything, so a mistake that only exists there shows up on your machine.

## Which standard library

`jacket test` uses the `stdlib/` of your project if it has one, else the installed standard library. Set `JAIC_STDLIB` to use another.

## Settings

Run-wide defaults live in `jacket.toml`:

```toml
[test]
paths = ["src/**"]    # test roots, folders or patterns
jobs = 8              # tests at a time
timeout = 120         # seconds a test may take; 0 for no limit
retries = 0
fail_fast = false

[test.profile.ci]     # jacket test --profile ci
retries = 2
fail_fast = true
```

Flags win over the profile, and the profile over `[test]`.

## Related

- [Benchmarking](benchmarking.md) for `@bench`.
- [Testing module reference](../extensions/testing.md) for how checks, cases and properties work inside.
- [jacket test](../tools/jacket-test.md) for how the runner works.
