# String


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `compare :: (a: string, b: string) -> int`
- `compare_nocase :: (a: string, b: string) -> int`
- `equal :: (a: string, b: string) -> bool`
- `equal_nocase :: (a: string, b: string) -> bool`
- `contains :: (s: string, substring: string) -> (found: bool, remainder: string)`
- `contains_nocase :: (s: string, substring: string) -> (found: bool, remainder: string)`
- `contains :: (s: string, char: u8) -> (found: bool, remainder: string)`
- `contains_any_character :: (s: string, chars: string) -> (found: bool, remainder: string)`
- `contains_any_string :: (s: string, substrings: .. string) -> (found: bool, remainder: string)`
- `begins_with :: (s: string, prefix: string) -> bool`
- `ends_with :: (s: string, suffix: string) -> bool`
- `begins_with_nocase :: (s: string, prefix: string) -> bool`
- `ends_with_nocase :: (s: string, suffix: string) -> bool`
- `index_of_string_nocase :: (s: string, substring: string) -> int, found:bool`
- `replace_chars :: (s: string, chars: string, replacement: u8)`
- `wildcard_match :: (s: string, pat: string) -> bool`
- `scan :: (format: string, text: string) -> bool, [] Any`
- `scan2 :: (text: string, format: string, args: ..Any) -> bool`
- `replace :: (s: string, old: string, new: string) -> string, ocurrences:int`
- `to_lower_in_place :: ($$ s: string)`
- `to_upper_in_place :: ($$ s: string)`
- `to_lower_copy :: (s: string) -> string`
- `to_upper_copy :: (s: string) -> string`
- `slice :: inline (s: string, index: s64, count: s64) -> string`
- `join :: (inputs: .. string, separator := "", before_first := false, after_last := false) -> string`
- `find_index_from_left :: (s: string, substring: string, start_index := 0) -> s64`
- `find_index_from_right :: (s: string, substring: string) -> s64`
- `trim_left :: (s: string, chars := DEFAULT_SPACES) -> string`
- `trim_right :: (s: string, chars := DEFAULT_SPACES) -> string`
- `trim :: (s: string, chars := DEFAULT_SPACES) -> string`
- `normalize_line_endings :: (s: *string)`
- `normalize_line_endings :: (s: string) -> string`
- [OS == .WINDOWS] `PATH_SEPARATOR :: #char "\\"`
- [else branch] `PATH_SEPARATOR :: #char "/"`
- `decomp_filename :: path_decomp`
- `path_decomp :: (name: string) -> (path: string, basename: string, extension: string, basename_with_extension: string)`
- `path_filename :: (path: string) -> string`
- `path_extension :: (path: string) -> (extension: string, found_extension: bool)`
- `path_strip_filename :: (path: string) -> string`
- `path_strip_extension :: (path: string) -> string`
- `path_overwrite_separators :: (path: string, separator: u8 = PATH_SEPARATOR)`
- `basename :: (s: string) -> string`
- `path_basename :: basename`
- `path_ensure_extension :: (path: string, extension: string) -> string`
- [OS != .NN_SWITCH2] `is_absolute_path :: (path: string) -> bool`
- `path_join :: (components: ..string, separator: u8 = PATH_SEPARATOR) -> string`
- `DEFAULT_DELIMITERS :: ";=()"`
- `DEFAULT_SPACES :: " \t\r\n"`
- `parse_bool :: (line: *string, delimiters:=DEFAULT_DELIMITERS, spaces:=DEFAULT_SPACES) -> value:bool, success:bool`
- `parse_token :: (line: *string, delimiters:=DEFAULT_DELIMITERS, spaces:=DEFAULT_SPACES) -> value:string, success:bool`
- `match_token :: (line: *string, match: string, delimiters:=DEFAULT_DELIMITERS, spaces:=DEFAULT_SPACES) -> value:string, success:bool`
- `match_token_nocase :: (line: *string, match: string, delimiters:=DEFAULT_DELIMITERS, spaces:=DEFAULT_SPACES) -> value:string, success:bool`
- `parse_enum :: (line: *string, $T : Type, delimiters:=DEFAULT_DELIMITERS, spaces:=DEFAULT_SPACES) -> value:T, success:bool`
- `parse_float :: (line: *string, delimiters:=DEFAULT_DELIMITERS, spaces:=DEFAULT_SPACES) -> value:float, success:bool`
- `parse_int :: (line: *string, $T : Type = int, spaces:=" \t", delimiters:=DEFAULT_DELIMITERS) -> value:T, success:bool`
- `split_from_left :: (s: string, byte: u8) -> (found: bool, left: string, right: string)`
- `split_from_right :: (s: string, byte: u8) -> (found: bool, left: string, right: string)`
- `split :: (s: string, separator: $T/.[string, u8]) -> [] string`
- `split_from_left_by_any :: (s: string, bytes: string) -> (found: bool, left: string, right: string)`
- `split_from_right_by_any :: (s: string, bytes: string) -> (found: bool, left: string, right: string)`
- `split_from_left :: (s: string, separator: string) -> (found: bool, left: string, right: string)`
- `split_from_right :: (s: string, separator: string) -> (found: bool, left: string, right: string)`
- `find_index_from_left :: (s: string, byte: u8, start_index := 0) -> s64 #no_abc`
- `find_index_from_right :: (s: string, byte: u8) -> s64 #no_abc`
- `find_index_of_any_from_left :: (s: string, bytes: string, start_index := 0) -> s64 #no_abc`
- `find_index_of_any_from_right :: (s: string, bytes: string) -> s64 #no_abc`
- `stop_at_any :: (s: string, bytes: string) -> string`
- `starts_with :: begins_with`
- `eat_trailing_spaces :: (_s: string) -> string`
- `eat_until_space :: (_s: string) -> string`
- `is_any :: (c: u8, chars:string) -> bool`
- `string_tests :: ()`


## Behavioral contract

The module complements basic string operations with byte-oriented search, comparison, slicing, replacement, splitting, trimming, case conversion, path manipulation, and token parsing. Case-insensitive routines are ASCII-only. In-place transforms mutate the supplied buffer and must not be used on immutable literal storage. Search results such as `remainder` and path component results are often views into the original input; treat them as tied to that allocation unless a routine explicitly returns a new string. Searches proceed left-to-right; replacement does not match overlapping occurrences. `string_tests` is an exported assertion-based exercise of the string routines.

### Operation-level behavior

| Operation group | Observable behavior |
| --- | --- |
| `compare`, `equal`, prefix/suffix checks | Compare bytes, with case-sensitive variants exact and `_nocase` variants ASCII-folded. Ordering compares lexicographically and returns negative, zero, or positive. Empty prefixes/suffixes match. |
| `split_from_left`, `split_from_right` (and the `_by_any` forms) | Every overload returns `found = false`, the whole input as `left` and an empty `right` when there is no match. |
| `contains`, `find_index_*` | Indices are byte offsets. Missing matches return `-1`; `contains` reports `found = false` and an empty remainder. On success, its remainder begins immediately after the matched byte/string. Empty-string searches are treated as absent by substring search. A negative `start_index` is treated as 0. Substring search skips to offsets holding the substring's first byte before comparing. |
| `wildcard_match` | Supports `*`, `?`, and bracket character sets/ranges over bytes. No Unicode character interpretation is implied. |
| `replace_chars`, case conversion | Mutating forms change the supplied bytes in place; copy forms allocate a separate string. Case mapping is ASCII. `replace` replaces non-overlapping occurrences and returns the replacement count; an empty old string produces no replacements. |
| `slice` | Negative index/count assert. An index at or beyond the end returns empty; an overlong count is clipped. The result is a view into the input. |
| `join` | Concatenates values with the separator and optional leading/trailing separators. Empty input produces empty output. The returned storage is newly allocated. |
| `trim_*` | Return views after removing all edge bytes found in the supplied set; default set is `DEFAULT_SPACES`. |
| `split` | Preserves empty fields at the beginning, between repeated separators, and at the end. Empty input yields one empty field; an empty string separator yields the original string as one field. Returned fields are views. On no match, left splitters return `(false, s, "")`; right split by byte returns `(false, "", s)`, while right split by string or by any byte returns `(false, s, "")`. |
| `normalize_line_endings` | Converts CRLF pairs to LF; bare CR bytes remain unchanged. The pointer form mutates the string and adjusts its count; the value form returns newly allocated storage. |
| `parse_token` and token parsers | Leading configured whitespace is skipped. A delimiter at the beginning is itself a one-byte token; otherwise token ends at whitespace or delimiter. Successful parsing advances past leading whitespace and the token, leaving trailing whitespace for the next call. Failed token extraction restores the original input view. `match_token*` and typed parsers consume an extracted token even when its value does not match or convert. `parse_bool` accepts case-insensitive `true`/`false`; `parse_enum` requires an exact member name. `parse_int` parses to `int` then casts to `$T` without checking representability, so a narrowing target can lose information. |
| `scan`, `scan2` | `scan` recognizes `%f`, `%d`/`%i`, `%b`, and quoted `%s`; it returns success plus matched values in temporary storage, retaining values read before a later failure. In both, `%%` matches and consumes a literal `%`. `scan2` writes through pointer-valued `Any` arguments. Both match ordinary literal bytes exactly. `scan2` may have already written earlier outputs when a later match fails; unused extra arguments are ignored. Neither quoted-string parser processes escapes. |
| Path decomposition | Components are byte views. `path_filename` is empty after a trailing separator; `path_strip_filename` includes the final separator and is empty when none exists. Extension omits the dot; leading dotfiles are not treated as extensions. `path_ensure_extension` compares extension text without a dot and returns the original view when it already matches. |
| `is_absolute_path` | On Windows, requires a drive-letter form with `:` followed by a slash or backslash. Elsewhere, requires a leading `/`. |

Not every parser conversion failure or out-of-range search start has a uniform error contract. Callers should pass valid byte offsets and use the returned success flag rather than infer conversion success from the output value.

`DEFAULT_DELIMITERS` is the string `";=()"`. `DEFAULT_SPACES` is `" \t\r\n"`, including ordinary space, tab, carriage return, and line feed. Token parsers use these defaults except `parse_int`, whose default whitespace set is only space and tab; callers can supply an explicit set.

`PATH_SEPARATOR` is `\` on Windows and `/` elsewhere. Path operations recognize both slash forms when locating separators, while `path_overwrite_separators` rewrites either form in place to the requested byte (OS separator by default). `path_join` removes one trailing chosen separator from each component before joining components with that separator; callers needing another convention should pass it explicitly. `is_absolute_path` is unavailable on Nintendo Switch 2. The module depends on Basic string allocation and byte operations.


Platform coverage note: Nintendo branches exist, but their declarations are not specified here; exports for those targets are unspecified.
