# Regex

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

Both runtime and `Static` patterns support `is_match`, `find`, `find_all`, `captures`, `replace` (`$1`, `${name}`, `$$`), `split` and `escape`. `Static` adds `parse` (the whole text must match) and `parse_into`. Results are views into the text; arrays and replaced strings use `context.allocator`.

## How it works

- **Syntax** (`syntax.jai`) is Rust's: nested classes, `\p{..}`, flags, `(?<n>..)`. Errors carry a byte offset.
- **Programs** (`program.jai`): parse tree to a byte-level NFA (forward and reversed); classes become UTF-8 byte tries.
- **Search** (`search.jai`): a lazy forward DFA finds the match end, a reverse DFA the start. Captures come from the found span via the first applicable of a one-pass table (`onepass.jai`), a bounded backtracker (`nfa.jai`), or a Pike VM. Everything is O(pattern x text).
- **Skipping** (`dfa_skip`): in an unanchored start state, a `Byte_Search` call jumps to the next place a match could begin, using rare bytes, byte sets at fixed distances or a required byte.
- **One-pass** patterns (each byte leaves at most one way forward) capture in one table walk. Unicode classes often aren't one-pass (`(\w+)\s(\w+)`); ASCII-only or `(?-u)` ones are. `Regex.is_one_pass` reports it.
- Matching is leftmost-first like Rust and Perl. A group in a repetition keeps its last iteration.
- **`Static`** builds the engine in a `#run` constant so search needs no setup. If a DFA needs more than `STATIC_STATE_LIMIT` (256) states (large Unicode classes like `\w`), the engine is built lazily on first use instead, since freezing would add about a second of compile time per pattern.
- **Typed groups** (`R.Groups`): digits become the smallest integer type that fits, digits `.` digits become `float64`, an alternation of 2-64 words becomes an enum named after the group (`kind` becomes `Kind`), anything else `string`. A number that doesn't fit makes the call return `false`. Optional groups get a `bool` in `has`. Field/group mismatches in `parse_into` are compile errors.

## How to change it

- Syntax: `parse_*` in `syntax.jai`; add cases to `tests/matching.jai` and the corpus generator's atoms.
- New look-around assertion: add to `Look`, then `look_holds` (`nfa.jai`), `look_holds_between` (`dfa.jai`) and `compute_byte_classes`.
- Type inference: `infer_kind`, `number_shape`, `groups_members`, `groups_fill`, `into_fill` in `generate.jai`. The emitted code is plain Jai text; print it to debug.
- One-pass rules: `explore`, `fill_state`. Never accept a conflicting pattern; a wrong table shows up in the differential test or the `fill_slots` assert.
- Skips: `dfa_skip` and its constants. A skip may only move to positions where no match can start; if unsure return `state = UNKNOWN`.
- Keep `tests/differential.jai` passing (`tests/data/differential_cases.jai`); set `DEFAULT_STATE_LIMIT` to 3 temporarily to force the Pike VM. Regenerate the corpus with `tools/corpus/gen_regex_corpus.py` (see `--help`). Unicode data: [Regex Unicode tables](../tools/regex-unicode-tables.md).
- Gotchas: runtime `Regex` values share a mutable DFA cache, so use one per thread. Lazily built `Static` engines share a global cache.

`Options` (run time): `case_insensitive`, `multi_line`, `dot_all`, `swap_greed`, `ignore_space`, `crlf`, `unicode` (default on); inline flags `(?imsxRU-u)` also work. Limits: `DEFAULT_STATE_LIMIT`, `STATIC_STATE_LIMIT`, `ONEPASS_STATE_LIMIT`, `MAX_NESTING` (250). Depends on [Byte_Search](byte-search.md). Tests are in `stdlib/Extensions/Regex/tests/`.

## Compared with the community libraries

`benchmarks/compare/regex/benches/regex.jai` (`tools/bench/library_compare.py regex`, see [library comparisons](../tools/library-comparisons.md)) runs the same patterns over generated ASCII text with Jacket (run-time and `Static`), Uniform (an RE2 port, Pike VM only) and Rematch (an ECMAScript backtracker). Time relative to `Static` (Apple M5; above 1 is slower):

| Workload | Uniform | Rematch |
| --- | --- | --- |
| log parse (six captures per line) | 14x | 4x |
| log filter (no captures) | 8x | 45x |
| extract emails / URLs | 328x / 27x | 249x / 43x |
| search names / `[a-zA-Z]+ing` | 105x / 52x | 356x / 80x |

The gap is the engine: Uniform runs its NFA over every byte and Rematch backtracks from every start, while Jacket's DFA skips to where a match can start. Run-time and `Static` are level except in capture-heavy work. Compiling is the one place the others win or tie: 6 us for an e-mail pattern, level with Uniform; Rematch only parses.

Rematch has backreferences, look-around, match limits and stream input, which Jacket doesn't. Jacket has Unicode classes, linear time, split, typed groups and compile-time patterns; Uniform lacks named groups, find-all and replace.
