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
- **Engines:** keep `tests/differential.jai` passing (313 pattern/text pairs checked against Rust regex 1.13.1, `tests/data/differential_cases.jai`); set `DEFAULT_STATE_LIMIT` to 3 temporarily to force the Pike VM. Regenerate the corpus with `tools/corpus/gen_regex_corpus.py` (it needs a small Rust reference program at `tools/corpus/regex_oracle/`; see the script's `--help`). Unicode data: [Regex Unicode tables](../tools/regex-unicode-tables.md) (`tools/generators/gen_regex_unicode.py`, UCD 17).
- Gotchas: runtime `Regex` values share a mutable DFA cache, so use one per thread; lazily built `Static` engines share a global cache. A `Groups` enum member clashing with a field gets a `_Value` suffix.

## Configuration

`Options` (run time): `case_insensitive`, `multi_line`, `dot_all`, `swap_greed`, `ignore_space`, `crlf`, `unicode` (default on); inline flags `(?imsxRU-u)` also work. Limits: `DEFAULT_STATE_LIMIT` (`dfa.jai`), `STATIC_STATE_LIMIT` (`generate.jai`), `ONEPASS_STATE_LIMIT`, `ONEPASS_STATIC_STATE_LIMIT` (`onepass.jai`), `MAX_NESTING` (250).

## Dependencies and tests

`Basic`, `String`, `Hash_Table`, `Sort`, `Compiler`, [Byte_Search](byte-search.md), generated `unicode_data.jai`. Tests in `stdlib/Extensions/Regex/tests/`: `matching`, `typed`, `differential`, `unicode`, plus `tests/corpus/negative/regex-*.jai`. Throughput is on par with Rust's `regex` for most patterns and lower for dense short matches (`\b\w+\b`); numbers are in `benchmarks/results/regex.md`.

## Compared with the community libraries

Jacket's Regex next to [`rluba--uniform`](https://github.com/rluba/uniform) (a port of RE2; what `smari--jai-simplehttp` and `rluba--jai-date` use) and [`Lunatoid--Rematch`](https://github.com/Lunatoid/Rematch) (an ECMAScript-style engine), the two regular-expression libraries in the upstream corpus. The cells come from reading each library's source; ⚠ has a note.

| Feature | Jacket Regex | Uniform | Rematch |
| --- | --- | --- | --- |
| How it matches | lazy DFA for the match, reverse DFA for its start, one-pass table, bounded backtracker or Pike VM for captures | Pike VM over the compiled program (RE2's NFA only: no DFA, no one-pass) | backtracking over a tree of steps with an explicit stack |
| Time linear in pattern and text | ✅ | ✅ | ❌ [1] |
| Syntax | Rust `regex` | RE2 / Perl, no backreferences | ECMAScript |
| Backreferences | ❌ | ❌ | ✅ |
| Look-ahead and look-behind | ❌ | ❌ | ✅ [2] |
| Unicode classes (`\p{Greek}`) | ✅ | ✅ | ❌ |
| UTF-8 text: `.` and classes take whole characters | ✅ | ✅ | ✅ |
| Case-insensitive beyond ASCII | ✅ | ✅ | ❌ [3] |
| Named groups | ✅ also as typed fields | ⚠ [4] | ✅ |
| Find all matches | ✅ `find_all` | ❌ [5] | ✅ `GLOBAL` |
| Replace with group references | ✅ `$1`, `${name}` | ❌ | ✅ `$1`, `$<name>`, `$&` |
| Split | ✅ | ❌ | ❌ |
| Groups as typed fields (integers, floats, enums) | ✅ `Static` | ❌ | ❌ |
| Pattern compiled while the program compiles, bad pattern is a compile error | ✅ `Static` | ❌ | ❌ |
| Error for a bad pattern | ✅ byte offset | ⚠ [6] | ⚠ [6] |
| Limits on one match (time, memory) | ❌ [7] | ❌ | ✅ `time_limit_ms`, `memory_limit` |
| Input from a stream | ❌ | ❌ | ✅ `Text_Stream` |
| Anchoring options | ✅ `^`, `\A`, `parse` | ✅ `Anchor` | ✅ `STICKY` |
| Checked against a reference | ✅ 313 pairs against Rust `regex` | ⚠ [8] | ❌ |

1. Rematch limits a match by time and memory instead (`Match_Options`); a pattern such as `(a+)+b` takes exponential time up to that limit.
2. Rematch's look-behind carries a `:LookBehindBandaid` marker in `match.jai`.
3. Rematch's README calls `CASE_INSENSITIVE` "Ignores all ASCII case".
4. Uniform parses `(?P<name>...)` but `match` returns the groups as a plain `[] string`; the names are not available.
5. `match` returns the first match. The benchmark's find-all loops search the rest of the text after each match.
6. A failed `compile` returns `false` and logs the reason; there is no position.
7. The DFA's cache is bounded (`DEFAULT_STATE_LIMIT`), and a pattern over the limit falls back to the Pike VM; the time stays linear, so there is nothing to limit.
8. Uniform has a test suite (`tests/`); the project says it is "minimally tested".

### How it was measured

`benchmarks/compare/regex/benches/regex.jai`, run by `tools/bench/library_compare.py regex` (see [library comparisons](../tools/library-comparisons.md)). Texts are generated with a fixed seed, ASCII only so that the three engines (Unicode-aware, Unicode-aware, ASCII `\w`) agree on every match; every benchmark checks its count against the one Jacket finds.

- **log_parse**: 20,000 access-log lines (2.9 MB), one anchored pattern with six capture groups per line (`^(\S+) \S+ \S+ \[([\w/:]+ \+\d{4})\] "(\w+) (\S+) [^"]*" (\d{3}) (\d+)`).
- **log_filter**: the same lines, which have a 4xx or 5xx status (`" [45]\d\d `), a search without captures.
- **extract_emails**, **extract_urls**: every address and every `http(s)://` URL in 1 MB of prose (about 2,000 of the former), a find-all.
- **search_names**, **search_ing**: every match of an alternation of five names and of `[a-zA-Z]+ing` over the same prose.
- **compile_email**: compiling the e-mail pattern, once per iteration.

`jacket` compiles each pattern at run time, as the others do, before the timer starts; `jacket_static` is `Regex.Static`, compiled while the program compiled. Everything allocates from the default allocator and frees what it was given. Inputs go through `black_box`.

Apple M5 (10 cores: 4 Super and 6 Efficiency), 16 GB, macOS 27.0 arm64. The machine was a working desktop, not an idle one: the load average when the run began was 3.07, so absolute times carry a few percent of noise (the intervals show the part that was measured within one run) and the ratios are the result to read.

Time per iteration with the 95% interval, and the ratio to `jacket_static` (above 1: slower).

| Workload | jacket_static | jacket | uniform | rematch |
| --- | --- | --- | --- | --- |
| log_parse | 10.1 ms (10.1 ms to 10.2 ms) | 9.67 ms (9.62 ms to 9.73 ms) 0.95x | 142 ms (141 ms to 143 ms) 13.99x | 42.9 ms (42.8 ms to 43.1 ms) 4.23x |
| log_filter | 1.24 ms (1.24 ms to 1.24 ms) | 1.2 ms (1.2 ms to 1.2 ms) 0.97x | 10.4 ms (10.4 ms to 10.4 ms) 8.43x | 56.2 ms (56.1 ms to 56.2 ms) 45.41x |
| extract_emails | 381 us (379 us to 384 us) | 377 us (376 us to 378 us) 0.99x | 125 ms (125 ms to 125 ms) 328.39x | 95.1 ms (94.8 ms to 95.3 ms) 249.23x |
| extract_urls | 410 us (409 us to 411 us) | 409 us (409 us to 410 us) 1.00x | 10.9 ms (10.9 ms to 10.9 ms) 26.55x | 17.8 ms (17.8 ms to 17.8 ms) 43.42x |
| search_names | 723 us (722 us to 724 us) | 727 us (725 us to 728 us) 1.00x | 75.6 ms (75.4 ms to 75.9 ms) 104.58x | 258 ms (257 ms to 259 ms) 356.38x |
| search_ing | 1.05 ms (1.05 ms to 1.06 ms) | 1.07 ms (1.06 ms to 1.07 ms) 1.01x | 54.9 ms (54.8 ms to 55 ms) 52.06x | 84.7 ms (84 ms to 85.5 ms) 80.36x |
| compile_email | n/a | 6.18 us (6.18 us to 6.19 us) | 6.62 us (6.61 us to 6.63 us) | 318 ns (316 ns to 320 ns) |

Reading the table:

- **The gap is the engine.** Uniform runs its NFA over every byte of the text, with a copy of every thread's captures; Rematch backtracks from every start position. Jacket's DFA skips to the places a match can start (a rare byte, a required byte), which is why `extract_emails` takes 0.4 ms where the others take about 100. On the log lines, where every line matches from its first byte and six groups are captured, the gap is 4x to 14x on `log_parse` and 8x to 45x on `log_filter`.
- `jacket` against `jacket_static` is the run-time cache against the frozen tables built at compile time; the two are level except in the capture-heavy `log_parse`, where the lazily built DFA has to be filled from the first line.
- Compiling is the one place the others win: Jacket builds the byte-level program and both automata's start states (6 us for the e-mail pattern, level with Uniform); Rematch only parses (0.4 us) and does its work during matching.
