# Unicode text

`Extensions/Unicode_Text` implements NFC, NFD, NFKC and NFKD normalization and the default extended grapheme, word and sentence rules for UCD 17.0.0. It uses the [normalization](https://www.unicode.org/reports/tr15/tr15-57.html) and [segmentation](https://www.unicode.org/reports/tr29/tr29-47.html) specifications. Line breaking, locale tailoring, dictionary segmentation, collation, case folding, BiDi and IDNA are outside this module.

```jai
#import "Basic";
#import "Extensions/Unicode_Text";

builder: String_Builder;
defer free_buffers(*builder);
error := normalize("é", .NFC, *builder);
assert(error.kind == .NONE);
text := builder_to_string(*builder);
defer free(text);
assert(text == "é");

iter := segments("hello world", .WORD);
while true {
    start, end, more := next_segment(*iter);
    if !more break;
    print("%..%\n", start, end); // 0..5, 5..6, 6..11
}
assert(iter.error.kind == .NONE);
```

## How it works

Normalization expands generated recursive decompositions, handles Hangul algorithmically, stably orders combining runs with counting sort, then composes eligible pairs. Even a disordered run of thousands of marks takes linear work. `is_normalized` uses the form's quick-check bits and canonical ordering; `Maybe` cases use exact normalization and comparison.

`segments(text, kind)` borrows the input and allocates nothing. `next_segment` returns half-open byte offsets. Check the iterator's `error` after EOF: malformed UTF-8 may be detected during lookahead before the current segment is returned. Word segments include whitespace and punctuation. Sentence rules include closing punctuation and trailing whitespace; they do not interpret abbreviations semantically. Sentence lookahead is cached so long whitespace and punctuation runs remain linear.

`next_grapheme` accepts a known boundary; `previous_grapheme` backs up to a guaranteed boundary and uses the same forward rules; regional-indicator runs can require context back to their start. Both return an offset and error. Text_Buffer re-exports these helpers and `grapheme_count`; UI uses that shared implementation. UI hit testing retains replacement-byte rendering for malformed text.

Errors contain a kind, static message and byte offset; they need no freeing. Normalization appends only after input and output validation, so malformed input and limit errors leave the destination unchanged. A builder allocation failure is reported and can leave partial output. Builder buffers and strings taken from it remain the caller's responsibility.

## Configuration

`Unicode_Text_Options` defaults to 16 MiB of input and 64 MiB of output. `max_output` bounds both intermediate decomposed scalar count and final byte count, excluding bytes already in the builder. Negative limits and invalid enum values return `INVALID_ARGUMENT`.

Ordinary normalization preserves canonical equivalence, including arbitrarily long combining sequences within those limits. Set `stream_safe = true` on `normalize` to produce UAX #15 Stream-Safe Text Format: before a scalar's NFKD expansion would exceed 30 consecutive nonstarters, insert U+034F CGJ. This intentionally changes the text's canonical equivalence. `is_normalized` checks the selected normalization form, independently of the stream-safe option.

## How to change it

Keep rule changes in `segmentation.jai` and normalization in `module.jai`. Generated files use deduplicated 256-code-point blocks, a first-stage block index and second-stage payload. Composition pairs are sorted for binary search. The shared Regex generator's `open_source` owns UCD URL and SHA-256 pinning; do not introduce another downloader.

Regenerate the tables and fetch the conformance suites (written to `target/unicode-text-conformance/`), then test:

```sh
export PYTHONPATH=tools
python3 tools/generators/gen_unicode_text.py --formatter target/jaifmt
python3 tools/unicode_text_conformance/fetch.py   # also part of tools/portable/build_portable.py data
jacket test --path stdlib/Extensions/Unicode_Text
```

Missing suites fail the conformance tests with the command to run. The tests check every row of all four suites, the normalization-form invariants, quick-check answers, and unchanged scalars omitted from normalization Part 1 (surrogates excepted: they are not valid UTF-8). Safety tests cover malformed input, limits, long sequences, stream-safe idempotence and allocation balance.

Run `jacket bench --path stdlib/Extensions/Unicode_Text` for separate NFC and grapheme cases over the generated mixed-script corpus. `bench/reference.py` uses Python's `unicodedata` on the same corpus. Run measurements after builds finish.

## Dependencies

Runtime: `Basic` for builders, arrays and allocation; `Unicode` for strict UTF-8 decoding and encoding into caller storage. No external runtime libraries or changes to `stdlib/Unicode` are required. Generation and suite extraction use Python's standard library, `tools/toolkit/download.py` through the Regex generator, and the pinned UCD archive. Unicode data notices are in `stdlib/Extensions/Unicode_Text/LICENSE.txt`. Downloads live in the toolkit cache (`XDG_CACHE_HOME`); the suites live under `target/` (or `JACKET_OUTPUT_DIR`).
