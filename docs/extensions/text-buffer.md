# Text buffer

An editable UTF-8 buffer for editors: a piece table with line lookup, undo/redo, and grapheme and word movement. It is the text model of the UI text widgets and the example editor.

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

- `module.jai`: loaded text lives in `original` (never modified), typing in `added` (append only). `pieces` are ordered `{from_added, start, length}` runs; an edit splits at most two. `line_starts` is a sorted offset array, so `line_of_offset` is a binary search.
- `history.jai`: each edit is `(offset, removed, inserted, group)`. Adjacent typing or deletion merges into one undo step; `begin_group`/`end_group` make several edits one step; `seal_undo` stops merging (call it when the cursor moves).
- `graphemes.jai`: `next_grapheme`, `next_word` and friends take a line and byte index. Clusters follow UAX 29 for what editors meet; Indic spacing marks are split at the mark (known gap).

## How to change it

- Offsets are bytes; keep cursors on boundaries by moving with the grapheme functions.
- `line_text` and `text_range` results are invalid after the next edit.
- New merge rule: `merge_into` in `history.jai`, and keep `mergeable` in step.
- Tests: `jacket test --path stdlib/Extensions/Text_Buffer`.
