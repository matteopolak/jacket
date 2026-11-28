# Writing stdlib doc comments

How doc comments in `stdlib/` and `stdlib/Extensions` are written. The model is the API docs of well-documented Rust crates (jiff, regex, serde, bitflags, bytes, indexmap). The syntax (what counts as a doc, links, fence options) is in [doc comments](../compiler/doc-comments.md); this page is about content.

## Module comment

The comment that opens `module.jai` is the module's front page.

1. One sentence on what the module is for.
2. The main entry points, linked: `[from_json]`, `[Json_Value]`. A short list when there are several jobs.
3. One runnable example of the common case, ending in an `assert`.
4. Anything that applies to every item: who frees results, which allocator is used, thread safety, limits on untrusted input.

Keep it under about 30 lines. Longer material goes in a guide under `docs/extensions/` and is linked.

## Item comment

Every public procedure, struct, enum and alias that a user calls or names gets a comment in this order:

1. **Summary.** One sentence, third person, present tense: "Returns the byte offset of the first match.", "Splits `s` at every `separator`." It is what lists and search show, so it stands on its own and doesn't repeat the name or the signature.
2. **Behaviour that isn't obvious**, each in a short paragraph or list, only when it applies:
   - *Errors*: what the error or `ok = false` result means and when it happens.
   - *Failure*: what `assert`s or crashes (an index out of range, a null pointer).
   - *Edge cases*: empty input, a missing key, zero, overflow, invalid UTF-8.
   - *Cost*: when it allocates and with which allocator, who frees the result, and the time when it isn't constant (`O(n)` in the length of `s`).
   - *Ownership*: whether the result points into the argument or is a copy.
3. **Example.** A fenced ```` ```jai ```` block that runs and checks its result with `assert`. Show the edge case the prose names, not only the happy path.
4. **Links** to the related items: the allocating or temporary twin (`sprint` and `tprint`), the inverse (`to_json` and `from_json`), the checked or in-place variant.

Parameters are described only when the name and type don't say enough, as a list item (`- limit: the most bytes to read; 0 means no limit.`). Fields and enum members get a trailing comment when their meaning isn't plain from the name.

```jai
// Returns the index of the first `needle` in `haystack` at or after `start`, or -1.
//
// Compares bytes, so a match may start inside a multi-byte character. An empty `needle`
// matches at `start`. Takes `O(n·m)` time in the worst case; see [Byte_Search] for long needles.
//
// ```jai
// assert(find_from("a,b,c", ",", 2) == 3);
// assert(find_from("abc", "z") == -1);
// ```
find_from :: (haystack: string, needle: string, start := 0) -> int { ... }
```

## Examples

- Examples run under `jacket test` and on the site's Run button, so they must pass. Use `assert`, not `print`.
- A block of statements runs with `Basic` and the item's library imported. Add other `#import` lines at the top.
- Keep them short: the setup the call needs, the call, the checks. Free what the example allocates when the item's docs say the caller must.
- Use ```` ```jai,no_run ```` for files outside a temp folder, the network, processes and windows. It is still type-checked. ```` ```jai,ignore ```` is for fragments that can't stand alone; prefer making them stand alone.
- An alias (`print :: Impl.print;`) shows its target's docs. Document the target once, in `stdlib/internal`.

## What to leave out

- Restating the signature ("Takes a string and returns an int").
- History, plans and how the code is implemented, unless it changes what a caller sees.
- Comparisons with other languages or libraries.
- Claims the code doesn't back. Check each sentence against the body before writing it.

## Checking

```sh
jacket doc --check --module Json        # links and types in signatures
jacket test --doc --path stdlib/Extensions/Json   # runs the examples
PYTHONPATH=tools python3 tools/ci/check_stdlib_docs.py --report --examples --module Json
```

The last one lists public items without a doc comment or without an example. [Stdlib doc gaps](../tools/stdlib-doc-gaps.md) explains the CI list of undocumented items; run `--update` after documenting some.
