# Text buffer (`Extensions/Text_Buffer`)

## What it is

An editable UTF-8 text buffer for editors: a piece table with line lookup, undo and redo, and cursor-movement units (grapheme clusters and words). It is the text model of the UI library's text editing widgets and of the example editor.

```jai
#import "Extensions/Text_Buffer";

buffer: Text_Buffer;
defer free_buffer(*buffer);
set_text(*buffer, "hello\nworld");
insert(*buffer, 5, ", there");     // "hello, there\nworld"
line := line_text(*buffer, 1);     // "world"
undo(*buffer);                     // "hello\nworld"
```

## How it works

`module.jai` keeps the loaded text in `original` (never modified) and everything typed in `added` (append only). `pieces` is the document: ordered `{from_added, start, length}` runs. An edit splits at most two pieces and inserts or removes whole ones; typing right after the previous added piece just extends it. `line_starts` is a sorted array of line start offsets, shifted on each edit, so `line_of_offset` is a binary search and `line_text` is a view into one piece (a temporary copy only when a line crosses a piece boundary).

`history.jai` records every `insert`/`erase` as `(offset, removed, inserted, group)`. Typing a character after the previous typed character, or backspacing/deleting next to the previous deletion, merges into one undo step; pastes, cuts and breaks stand alone. `begin_group`/`end_group` make several edits one step (`replace` does), and `seal_undo` stops merging, which an editor calls when the cursor moves. `undo` and `redo` return where the cursor belongs.

`graphemes.jai` works on a `string` and byte index, so a caller passes a line: `next_grapheme`, `previous_grapheme`, `next_word`, `previous_word`. Clusters follow UAX 29 for what editors meet (CR LF, combining marks, ZWJ sequences, variation selectors, skin tones, flags, Hangul jamo). Indic spacing marks are split at the mark; that is the known gap.

## How to change it

- Offsets are bytes. Callers keep cursors on boundaries by moving with the grapheme functions.
- `line_text` and `text_range` results are invalid after the next edit.
- A new merge rule belongs in `merge_into` (`history.jai`); keep `mergeable` in step with it.
- Lines are found by `\n`; a trailing `\r` is trimmed from `line_end`.

## Configuration

None.

## Dependencies

`Basic` only. Tests: `stdlib/Extensions/Text_Buffer/tests/buffer.jai` (a property test against a byte-array model, undo/redo, graphemes, words), run with `jacket test --path stdlib/Extensions/Text_Buffer`.
