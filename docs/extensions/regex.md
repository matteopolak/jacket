# Regex (`Extensions/Regex`)

## What it is

Regular expressions with the syntax and matching rules of Rust's `regex` crate: full Unicode, linear-time matching, no backreferences or look-around. Compile at run time (`Regex.compile`) or while the program compiles (`Regex.Static`), where named groups become typed fields.

```jai
Regex :: #import "Extensions/Regex";

Date :: Regex.Static("(?<year>[0-9]{4})-(?<month>[0-9]{2})-(?<day>[0-9]{2})");   // bad pattern = compile error
Pet  :: Regex.Static("(?<kind>cat|dog) (?<age>\\d+)");

date, ok := Regex.parse(Date, "2026-10-11");     // Date.Groups; date.year is u16
pet := Regex.captures(Pet, "my dog 7");           // pet.kind == .DOG

re, compiled, error := Regex.compile("\\p{Greek}+");       // run time
for Regex.find_all(*re, "abc αβγ δ") print("[%] ", it.text);
Regex.free_regex(*re);
```

API (both runtime `*Regex` and `Static` types unless noted): `is_match`, `find`, `find_all`, `captures`, `captures_all`, `replace`/`replace_all`/`replace_n` (`$1`, `${name}`, `$$`), `split`, `escape`; `Static` only: `parse` (whole text must match), `parse_into`/`captures_into` (fill your struct by field name); `Captures` only: `group`, `span`. Results are views into `text`; arrays and replaced strings use `context.allocator`.

## How it works

- **Syntax** (`syntax.jai`) is Rust's: nested classes with `&&`/`--`/`~~`, `\p{..}`, flags `imsUxRu`, `(?<n>..)`, `\b{start}`. Errors carry a byte offset.
- **Programs** (`program.jai`): parse tree to a byte-level NFA (forward and reversed); classes become UTF-8 byte tries.
- **Search** (`search.jai`): a lazy forward DFA finds the match end, a reverse DFA finds the start; both cache states on demand. Captures come from the found span via the first applicable of: one-pass table (`onepass.jai`), bounded backtracker (`nfa.jai`), Pike VM (when the span exceeds the backtracker's 2M-bit visited set, or a DFA passes `DEFAULT_STATE_LIMIT` = 10,000 states). Fixed-length patterns skip the reverse DFA. Everything is O(pattern x text).
- **Skipping ahead** (`dfa_skip`): in an unanchored start state a `Byte_Search` call jumps to the next place a match could begin, using rare bytes (`BYTE_SHARE` frequency table), byte sets at fixed distances, or a required byte. The first 8 positions' byte sets (`fingerprint`) filter candidates; a pure chain pattern (`the`, `(?i)hello`) is decided by the fingerprint alone (`Skip.exact`).
- **One-pass** patterns (each byte leaves at most one way forward) capture in one table walk; looks become edge conditions. Refused when a look could change which instruction wins, or over `ONEPASS_STATE_LIMIT` (1,000). Unicode classes often are not one-pass (`(\w+)\s(\w+)`); ASCII-only/`(?-u)` are. `Regex.is_one_pass` reports it.
- **Matching rules:** leftmost-first like Rust and Perl; `find_all` skips an empty match at the previous end and never splits UTF-8; a group in a repetition keeps its last iteration.
- **Unicode `\b` in the DFA:** non-ASCII bytes are `QUIT` in the base column; the search decodes the character and steps through `CHARACTER_COLUMNS` extra column copies, so `(?-u:\b)` in the same pattern still sees them as non-word.
- **`Static`:** the engine is built in a `#run` constant (parse, compile, `dfa_build_all`, `dfa_freeze`, one-pass table up to 256 states) so search needs no setup. If a DFA needs more than `STATIC_STATE_LIMIT` (256) states (big Unicode classes like `\w`), freezing is skipped and the engine is built lazily on first use (`cached_engine`); freezing would cost about a second of compile time per pattern. Procedures taking a `Static` reject other types in `#modify` (`is_static`).
- **Typed groups** (`R.Groups`, from the parse tree): optional sign and digits become the smallest integer type for the digit count (`u8`..64-bit); digits `.` digits `float64`; an alternation of 2-64 identifier words an enum named after the group (`kind` becomes `Kind`, `hotDog` becomes `HOT_DOG`); anything else `string`. A number that does not fit makes the call return `false`. Optional groups get a `bool` in `has`. Field/group mismatches in `parse_into` are compile errors. Words that spell one member (`dog|Dog`) fold; if fewer than two members remain the group stays a `string`; a match through a non-ASCII fold returns `false`.

## How to change it

- **Syntax:** `parse_*` in `syntax.jai`; add cases to `tests/matching.jai` and the corpus generator's atoms.
- **New look:** add to `Look`, then `look_holds` (`nfa.jai`), `look_holds_between` (`dfa.jai`), and byte-class boundaries in `compute_byte_classes`. One-pass needs no change.
- **Type inference:** `infer_kind`, `number_shape`, `groups_members`, `groups_fill`, `into_fill` in `generate.jai` (emitted code is plain Jai text; print it to debug).
- **One-pass refusal rules:** `explore`, `fill_state`. Never accept a conflicting pattern (the result would depend on try order); a wrong table shows up in the differential test or the `fill_slots` assert.
- **Skips:** `dfa_skip` and its constants (`SINGLE_SHARE`, `SKIP_SHARE_LIMIT`, `FINGERPRINT_DEPTH`). A skip may only move to positions where no match can start; if unsure return `state = UNKNOWN`. `Skip.exact` stays false unless the fingerprint decides the match.
- **Engines:** keep `tests/differential.jai` passing (313 pattern/text pairs checked against Rust regex 1.13.1, `tests/data/differential_cases.jai`); set `DEFAULT_STATE_LIMIT` to 3 temporarily to force the Pike VM. Regenerate the corpus with `tools/gen_regex_corpus.py` (it needs a small Rust reference program at `tools/regex_oracle/`; see the script's `--help`). Unicode data: [Regex Unicode tables](../tools/regex-unicode-tables.md) (`tools/gen_regex_unicode.py`, UCD 17).
- Gotchas: runtime `Regex` values share a mutable DFA cache, so use one per thread; lazily built `Static` engines share a global cache. A `Groups` enum member clashing with a field gets a `_Value` suffix.

## Configuration

`Options` (run time): `case_insensitive`, `multi_line`, `dot_all`, `swap_greed`, `ignore_space`, `crlf`, `unicode` (default on); inline flags `(?imsxRU-u)` also work. Limits: `DEFAULT_STATE_LIMIT` (`dfa.jai`), `STATIC_STATE_LIMIT` (`generate.jai`), `ONEPASS_STATE_LIMIT`, `ONEPASS_STATIC_STATE_LIMIT` (`onepass.jai`), `MAX_NESTING` (250).

## Dependencies and tests

`Basic`, `String`, `Hash_Table`, `Sort`, `Compiler`, [Byte_Search](byte-search.md), generated `unicode_data.jai`. Tests in `stdlib/Extensions/Regex/tests/`: `matching`, `typed`, `differential`, `unicode`, plus `tests/corpus/negative/regex-*.jai`. Throughput is on par with Rust's `regex` for most patterns and lower for dense short matches (`\b\w+\b`); numbers are in `benchmarks/results/regex.md`.
