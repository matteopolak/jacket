# Regex (jaic extension)

## What it is

`Extensions/Regex` runs regular expressions with the syntax and matching rules of Rust's `regex` crate, with full Unicode support and a linear-time guarantee. A pattern can be compiled at run time (`Regex.compile`) or while the program compiles (`Regex.Static`). A static pattern's named groups become typed fields. It is a jaic extension, not official Jai: other compilers do not have it, and it may change between releases.

```jai
#import "Basic";
Regex :: #import "Extensions/Regex";

// Compile time: a type. A bad pattern is a compile error.
Date :: Regex.Static("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})");

Pet :: Regex.Static("(?<kind>cat|dog) (?<age>\\d+)");

Point :: struct { x: s32; y: s32; }

main :: () {
    date, ok := Regex.parse(Date, "2026-10-11");    // date: Date.Groups
    print("% %\n", date.year, type_of(date.year));  // 2026 u16

    pet := Regex.captures(Pet, "my dog 7");          // pet.kind == .DOG (Pet.Groups.Kind)

    point: Point;
    Regex.parse_into(Regex.Static("(?<x>-?\\d+),(?<y>-?\\d+)"), "3,-4", *point);

    // Run time: for patterns that only exist then.
    re, compiled, error := Regex.compile("\\p{Greek}+");
    if !compiled print("% at byte %\n", error.message, error.offset);
    for Regex.find_all(*re, "abc αβγ δ") print("[%] ", it.text);  // [αβγ] [δ]
    Regex.free_regex(*re);
}
```

| Procedure | Runtime `*Regex` | `Static` type | Result |
| --- | --- | --- | --- |
| `is_match(re, text, start = 0)` | yes | yes | `bool` |
| `find(re, text, start = 0)` | yes | yes | `Match, bool` |
| `find_all(re, text)` | yes | yes | `[..] Match` |
| `captures(re, text, start = 0)` | yes (`Captures`) | yes (`R.Groups`) | groups, `bool` |
| `captures_all(re, text)` | yes | yes | `[..]` of the above |
| `parse(R, text)` | no | yes | `R.Groups, bool`, whole text must match |
| `parse_into(R, text, *out)` / `captures_into(R, text, *out, start)` | no | yes | `bool` |
| `replace`, `replace_all`, `replace_n(.., limit)` | yes | yes | `string` (`$1`, `${name}`, `$$`) |
| `split(re, text)` | yes | yes | `[..] string` |
| `group(caps, index or name)`, `span(caps, index)` | `Captures` only | | |
| `escape(text)` | | | `text` with metacharacters escaped |

Results are views into `text`; arrays and replaced strings are allocated with `context.allocator`.

## How it works

**Syntax** (`syntax.jai`) is Rust's: classes with `&&`, `--`, `~~` and nesting, `\p{..}` with general categories, scripts, `scx=` and binary properties, Perl classes (`\d \s \w`, Unicode by default), flags `imsUxRu`, named groups `(?<n>..)` / `(?P<n>..)`, `\b{start}` and friends. Backreferences and look-around are errors, by design. Errors carry a byte offset. Flags can also come from `Options` (run time only).

**Programs** (`program.jai`): the parse tree compiles to a byte-level NFA, once forward and once reversed. Classes become UTF-8 byte tries with shared suffixes. Bytes that no instruction tells apart share a byte class.

**Search** (`search.jai`) picks matchers:

1. A lazy DFA (`dfa.jai`) runs forward to find where the leftmost-first match ends. A reverse DFA then runs back from there to find where it starts. States are built on demand and cached. Looks (`^`, `$`, `\b`) are settled when a transition is computed, since both neighbouring bytes are known then.
2. Captures come from the span the DFAs found, by the first of three matchers that applies. A pattern that is one-pass (`onepass.jai`) uses its table. Otherwise a bounded backtracker (`nfa.jai`) runs, or the Pike VM when the span is too long for the backtracker's visited bitset (2M bits).
3. The Pike VM also takes over when a DFA passes `DEFAULT_STATE_LIMIT` (10,000 states; from then on, that engine uses the Pike VM).
4. When every match has the same length (`fixed_length`, from the program), the start is `end - length` and the reverse DFA is skipped.

**Unicode `\b` in the DFA**: the base column of a non-ASCII byte holds `QUIT`. On one, the search decodes the character and steps through one of `CHARACTER_COLUMNS` (4) extra copies of the columns (`dfa_step_character`). Bit 0 of the copy says a word character starts at this byte, bit 1 that one ends with it. The context `CONTEXT_CHARACTER_WORD` is a word only to Unicode looks, so `(?-u:\b)` in the same pattern still sees non-ASCII bytes as non-word.

**Skipping ahead** (`dfa_skip`, `skip_to`): each unanchored start state gets a `Skip`, computed lazily, or by `dfa_build_all` for a `Static` engine. While the DFA sits in that state, a Byte_Search call jumps to the next place a match could begin:

| Kind | Searches for | Chosen when |
| --- | --- | --- |
| `BYTES` | 1-3 bytes (`find`, `find_any`) | one position has a rare set of at most 3 bytes |
| `SET` | a byte set (`find(*Byte_Set)`) | one position is rare enough, with more bytes |
| `PAIR` / `TRIPLE` | sets at fixed distances (`find_pair`, `find_triple`) | no single position is rare (`SINGLE_SHARE`), and adding a position cuts the expected stops enough |
| `REQUIRED` | a rare byte every match contains, then a walk back over the bytes allowed before it | every position near the start is common (`SKIP_SHARE_LIMIT`) |

"Rare" comes from `BYTE_SHARE`, a rough table of how often each byte occurs in text. Without looks, the sets come from the *fingerprint*: the bytes each of the first `FINGERPRINT_DEPTH` (8) positions of a match can have, up to the shortest match. A candidate must fit every fingerprint position (`fits_checks`) before the DFA sees it. When the program is a single chain (one byte-reading instruction per position, ending in a match: `the`, `(?i)hello`), the fingerprint is the pattern, so a fitting candidate *is* the match and the DFA does not run (`Skip.exact`). With looks, the only safe sets are the bytes that leave the start state. `REQUIRED` keeps a fence so each required byte is walked back from once, which keeps the search linear.

**One-pass captures** (`onepass.jai`): a pattern is one-pass when, matching anchored at a known start, each byte leaves at most one way forward. `onepass_build` finds out when the program is compiled: each state is the entry of a stretch of the program that waits for a byte, its epsilon closure is walked in priority order (once per instruction, like the Pike VM), and two byte-consuming instructions that accept the same byte class make the pattern not one-pass. A MATCH cuts off everything of lower priority, which keeps leftmost-first. Each table edge stores the next state and the capture slots to set. `onepass_search` walks the known span once and fails if it does not end exactly at `e`. Looks are conditions on the edge (`^ $ \b`, multi-line, CRLF, Unicode `\b` included) and are checked against the text when the edge is taken. A pattern is refused when a look could change which instruction wins: a look-guarded MATCH followed by anything, or a look-guarded instruction reached twice. It is also refused over `ONEPASS_STATE_LIMIT` (1,000) states. Unicode classes often are not one-pass: `(\w+)\s(\w+)` has `\w` and `\s` characters that share a UTF-8 lead byte, so the byte automaton cannot tell them apart; `(?-u)` classes and ASCII-only classes are fine. `Regex.is_one_pass` reports it.

Every matcher is O(pattern × text). There is no exponential backtracking.

**Matching rules**: leftmost-first, like Rust and Perl. `find_all` skips an empty match that starts where the previous match ended, and never splits a UTF-8 sequence with an empty match. A group inside a repetition keeps its last iteration. Every one of these rules is checked against the crate in the differential test.

**Static** (`generate.jai`): `Regex.Static(pattern)` is a polymorphic struct type. The procedures that take one (`$R: Type`) reject any other type in their `#modify` (`is_static`), so passing a runtime `*Regex` picks the runtime overload and anything else is a compile error. They build its engine in a `#run` constant: parse, compile, then `dfa_build_all` and `dfa_freeze` both DFAs, and the one-pass table (up to `ONEPASS_STATIC_STATE_LIMIT`, 256 states; a bigger one is left out and captures take the backtracker). The frozen tables then live in the program's data and search with no setup. If either DFA needs more than `STATIC_STATE_LIMIT` (256) states, typically for big Unicode classes like `\w` or `\pL`, freezing is skipped and the engine is built lazily on first use (`cached_engine`). Freezing those would cost about a second of compile time per pattern.

`parse` uses a second engine for `\A(?:pattern)\z`. `R.Groups` is generated from the parse tree:

| A named group that matches | becomes |
| --- | --- |
| an optional sign then decimal digits (any script's `\d`) | the smallest integer that holds the most digits: ≤2 `u8`/`s8`, ≤4 `u16`/`s16`, ≤9 `u32`/`s32`, else 64-bit |
| that, then `.` and digits | `float64` |
| an alternation of 2–64 identifier words | an enum `u8` named after the group, capitalized (`kind` → `Kind`), with each word as an upper snake case member (`dog` → `DOG`, `hotDog` → `HOT_DOG`) |
| anything else | `string` |

A number that does not fit its type makes the call return `false`. Groups that may not take part (under `?`, `*`, `{0,n}` or an alternation) get a `bool` in `has`. `parse_into` and `captures_into` generate the assignments for the caller's struct from its type info. A field without a group, a group without a field, or an unsupported field type is a compile error.

## How to change it

- **Syntax**: `parse_*` in `syntax.jai`. Add cases to `tests/matching.jai` and to the corpus generator's atoms.
- **A new look**: add it to `Look`, then `look_holds` (`nfa.jai`), `look_holds_between` (`dfa.jai`), and the byte-class boundaries in `compute_byte_classes`.
- **Type inference**: `infer_kind` and `number_shape` in `generate.jai`. The emitted code is in `groups_members`, `groups_fill` and `into_fill`. That code is plain Jai text, so print it while debugging.
- **One-pass**: the rules for refusing a pattern are in `explore` and `fill_state` (`onepass.jai`). A new `Look` needs no change there (the edge keeps a bit per `Look`, and `looks_hold` calls `look_holds`). To allow more patterns, make a state's closure smarter about when two leaves cannot both apply. Never accept a pattern that has a conflict: the result would depend on which path was tried first. A wrong table shows up as a mismatch in the differential test or the `fill_slots` assert.
- **Skips**: the choice is in `dfa_skip`; its thresholds are the constants above it (`SINGLE_SHARE`, `SKIP_SHARE_LIMIT`, `FINGERPRINT_DEPTH`). A skip may only move to positions where no match can start: if unsure, return the skip with `state = UNKNOWN`. `Skip.exact` must stay false unless the fingerprint alone decides the match.
- **Engines**: any change must keep `tests/differential.jai` passing. For a deeper check, generate a larger corpus with a new seed and compare. To exercise the fallbacks, temporarily set `DEFAULT_STATE_LIMIT` to 3, which forces the Pike VM path.
- **Unicode version**: see [Regex Unicode tables](../tools/regex-unicode-tables.md).

Gotchas:

- Runtime `Regex` values share a mutable DFA cache. Don't use one from two threads; compile one per thread. Frozen `Static` engines are read-only, but a lazily built one is shared through a global cache.
- `Groups` enum names are capitalized group names. A clash with another field gets a `_Value` suffix.
- Words that spell the same member (`dog|Dog|DOG`) fold into one. If that leaves fewer than two members, the group stays a `string`. Under `(?i)` a word matched in another case still maps to its member. One matched through a non-ASCII fold (the Kelvin sign for `k`) makes the call return `false`.

## Configuration

- `Options` (run time): `case_insensitive`, `multi_line`, `dot_all`, `swap_greed`, `ignore_space`, `crlf`, `unicode` (default on). The same flags work inline: `(?imsxRU-u)`.
- `DEFAULT_STATE_LIMIT` (`dfa.jai`) and `STATIC_STATE_LIMIT` (`generate.jai`): DFA size limits. `ONEPASS_STATE_LIMIT` and `ONEPASS_STATIC_STATE_LIMIT` (`onepass.jai`): the same for the one-pass table.
- `MAX_NESTING` (250) in `syntax.jai`. Repetition counts go up to `u32`, as in Rust. A program that grows too big is an error.

## Tests

- `stdlib/Extensions/Regex/tests/matching.jai`: the run-time API, errors, options and linear time.
- `tests/typed.jai`: `Static`, the type inference, `parse_into`.
- `tests/differential.jai`: 313 random pattern/text pairs, with the matches and capture spans that Rust's regex 1.13.1 found (`tests/data/differential_cases.jai`).
- `tests/unicode.jai`: the tables.
- `tests/corpus/negative/regex-*.jai`: the compile errors.

To regenerate the differential corpus, build the oracle (a 40-line Rust program, outside the workspace) and run the generator:

```
cargo build --release --manifest-path tools/regex_oracle/Cargo.toml
python3 tools/gen_regex_corpus.py --oracle tools/regex_oracle/target/release/regex-oracle
```

Throughput (`-O2`, Apple M5, 16 MB of mixed ASCII and Unicode text, `Regex.Static`, MB/s), against Rust `regex`:

| Pattern | Jai | Rust |
| --- | ---: | ---: |
| `[0-9]{4}-[0-9]{2}-[0-9]{2}` | 15,000-18,000 | 17,000-20,000 |
| `\w+@\w+\.com` | 12,000-13,500 | 22,600 |
| `\p{Greek}+` | 3,400 | 2,150 |
| `zzz` (no match) | 88,000-95,000 | 53,000 |
| `(?i)HELLO` | 14,000 | 12,600 |
| `\b\w+\b` | 96 | 136 |
| `Sherlock\|Holmes\|...` | 10,700 | 11,700 |
| `the` | 4,600 | 5,600 |
| `[a-zA-Z]+ing` | 920 | 730 |

Dense matches (`the`, `\b\w+\b`) are bound by the cost per match. The regex benchmark compares eight engines and records the numbers in `benchmarks/results/regex.md`.

## Dependencies

- `Basic`, `String`, `Hash_Table`, `Sort` and `Compiler` (for `compiler_report`).
- [Byte_Search](byte-search.md) for the skips.
- The Unicode tables in `unicode_data.jai`, generated from UCD 17 by `tools/gen_regex_unicode.py`.
- Test-only: Rust's `regex` crate in `tools/regex_oracle`.
