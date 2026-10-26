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

Apple M5 (10 cores: 4 Super and 6 Efficiency), 16 GB, macOS 27.0 arm64. The machine was a working desktop, not an idle one: the load average when the run began was 8.89, so absolute times carry a few percent of noise (the intervals show the part that was measured within one run) and the ratios are the result to read.

Time per iteration with the 95% interval, and the ratio to `jacket_static` (above 1: slower).

| Workload | jacket_static | jacket | uniform | rematch |
| --- | --- | --- | --- | --- |
| log_parse | 9.71 ms (9.61 ms to 9.89 ms) | 13.4 ms (11.7 ms to 15.7 ms) 1.38x | 95.6 ms (93.2 ms to 99.2 ms) 9.85x | 82.3 ms (69.9 ms to 93.7 ms) 8.48x |
| log_filter | 1.55 ms (1.54 ms to 1.56 ms) | 2.2 ms (2.12 ms to 2.29 ms) 1.42x | 12.2 ms (11.7 ms to 12.6 ms) 7.87x | 74.2 ms (70.8 ms to 77.8 ms) 47.96x |
| extract_emails | 793 us (758 us to 836 us) | 790 us (767 us to 813 us) 1.00x | 115 ms (112 ms to 118 ms) 145.28x | 129 ms (125 ms to 133 ms) 162.56x |
| extract_urls | 766 us (747 us to 786 us) | 804 us (788 us to 820 us) 1.05x | 10.3 ms (9.94 ms to 10.8 ms) 13.49x | 19.6 ms (19 ms to 20.2 ms) 25.54x |
| search_names | 1.61 ms (1.57 ms to 1.66 ms) | 1.7 ms (1.65 ms to 1.75 ms) 1.05x | 59.6 ms (58.1 ms to 61.3 ms) 36.89x | 252 ms (247 ms to 259 ms) 156.22x |
| search_ing | 1.51 ms (1.51 ms to 1.52 ms) | 1.51 ms (1.5 ms to 1.51 ms) 1.00x | 44.9 ms (44.5 ms to 45.3 ms) 29.65x | 88.5 ms (87.8 ms to 89.3 ms) 58.48x |
| compile_email | n/a | 5.91 us (5.86 us to 5.97 us) | 5.63 us (5.6 us to 5.66 us) | 384 ns (382 ns to 386 ns) |

Reading the table:

- **The gap is the engine.** Uniform runs its NFA over every byte of the text, with a copy of every thread's captures; Rematch backtracks from every start position. Jacket's DFA skips to the places a match can start (a rare byte, a required byte), which is why `extract_emails` takes 0.8 ms where the others take over 100. On the log lines, where every line matches from its first byte and six groups are captured, the gap is 8 to 10x.
- `jacket` against `jacket_static` is the run-time cache against the frozen tables built at compile time; the two are level except in the capture-heavy `log_parse`, where the lazily built DFA has to be filled from the first line.
- Compiling is the one place the others win: Jacket builds the byte-level program and both automata's start states (6 us for the e-mail pattern, level with Uniform); Rematch only parses (0.4 us) and does its work during matching.
