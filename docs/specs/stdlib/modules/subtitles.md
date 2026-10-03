# Subtitles


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Subtitles :: struct`
- `Subtitles_Section :: struct`
- `Subtitles_Line :: struct`
- `Subtitles_Instant :: struct`
- `Subtitles_Event :: struct`
- `SECTION_VERSION_NUMBER :: 1`
- `load_subtitles_from_file :: (path: string) -> bool, Subtitles`
- `load_subtitles_from_memory :: (data: string, short_name: string) -> bool, Subtitles`
- `serialize :: (using subtitles: *Subtitles) -> string`
- `free :: (using subtitles: *Subtitles)`
- `get_sections_for_characters :: (subtitles: Subtitles, names: .. string) -> [..] *Subtitles_Section`
- `format_sections_as_html :: (builder: *String_Builder, sections: [] *Subtitles_Section, character_to_emphasize := "", name_map: *Table(string, string) = null)`
- `format_section_as_html :: (builder: *String_Builder, section: *Subtitles_Section, character_to_emphasize := "", name_map: *Table(string, string) = null)`
- `csv_line_number: int`
- `csv_line_index_within_section: int`
- `format_sections_as_csv :: (builder: *String_Builder, sections: [] *Subtitles_Section, name_map: *Table(string, string) = null)`
- `format_section_as_csv :: (builder: *String_Builder, section: *Subtitles_Section, name_map: *Table(string, string) = null)`
- `flush_csv_lines :: (builder: *String_Builder, section_name: string, name_map: *Table(string, string), _speaker_name: string, lines: [] string, directions: [] string)`
- `csv_quote :: (s: string) -> string`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Subtitles`

```jai
Subtitles :: struct {
    data: string;
    data_owned: bool;

    sections_by_name: Table(string, *Subtitles_Section);
    all_sections: [..] Subtitles_Section;
}
```
### `Subtitles_Event`

```jai
Subtitles_Event :: struct {
    name: string;
    parameter: string;
    instant_index: s32;
    time_begin: float64 = -1;
    time_end:   float64 = -1;
}
```
### `Subtitles_Instant`

```jai
Subtitles_Instant :: struct {
    lines: [..] Subtitles_Line;
    speaker: string;

    time: float64 = -1;

}
```
### `Subtitles_Line`

```jai
Subtitles_Line :: struct {
    type: enum u8 {
        UNINITIALIZED :: 0;
        SPOKEN        :: 1;
        DIRECTION     :: 2;
    }

    text: string;
}
```
### `Subtitles_Section`

```jai
Subtitles_Section :: struct {
    name:  string;
    embed: string;
    instants: [..] Subtitles_Instant;
    events:   [..] Subtitles_Event;

    flags: enum_flags u32 {
        FINALIZED :: 1;
    }
}
```
## Contracts and constraints

The module loads and serializes a UTF-8 subtitle text format, preserving section order. Memory-backed loading borrows the input string, which must outlive the parsed `Subtitles`; file loading marks the read buffer as owned. `free` releases the section arrays, lookup table, and file-owned source buffer, but does not free memory-backed input. Instants last until the next instant; the last has no explicit end and lasts to the media end. Omitted instant/event times use `-1` sentinels. Explicit parse errors return `false` and an empty value. The parser relies on `Text_File_Handler` for the initial bracketed integer version line and comment handling, but does not compare that parsed integer to `SECTION_VERSION_NUMBER`; malformed-header status is not consistently propagated by the module.

## Text grammar and parse behavior

The first nonblank, noncomment line is a bracketed integer version header such as `[1]`. Section declarations begin with `:` and contain a required one-token section name plus an optional one-token embed name. A doubled marker `::` sets `FINALIZED`. There is no quoting or escaping for section names or embed names.

An instant begins with `=`, followed by a speaker token and optionally a floating-point timestamp. `_` maps to the empty speaker. A following ordinary line is a spoken line and requires a current instant. `++` starts a direction line on the current instant. A single `+` declares an event with a required name, optional parameter, optional begin time, and optional end time. If an event has no explicit begin time, it inherits the current instant's time field (which can still be the `-1` sentinel); a current instant must exist. An event with an explicit begin time may appear before the first instant and receives `instant_index = -1`. The parser does not impose monotonic-time, nonnegative-time, or begin-before-end constraints.

Event fields are positional and tokens are split on spaces; there is no quoting or escape syntax. With no remainder after the name, the parameter is empty and the begin time is inherited from the current instant. If a token follows the name, it is always consumed as the parameter before the parser looks for times. Therefore `+ fade 1.5 2.0` means parameter `1.5` and begin time `2.0`; it does not mean an empty parameter with begin/end times. To parse explicit begin/end times, a parameter token must occupy the parameter position. The format has no syntax for an empty parameter followed by explicit times.

Blank lines are skipped. Lines beginning with `#` are comments, and the default text handler strips a `#` and the remainder from any line before parsing it. Parsed text lines are trimmed at their edges by the text handler. Duplicate section names fail loading. `serialize` emits a fresh allocated string with a `[1]` header, sections in stored order, finalized markers, instants, events, spoken lines and `++` directions; the caller releases the returned string. It does not preserve comments or original whitespace. It writes events associated with an instant before that instant's lines and emits times only when they are nonnegative. For an event, it writes the parameter if nonempty, then writes the begin time only when `time_begin >= 0`, and writes the end only when both the begin and end are nonnegative. Thus a negative begin with a nonnegative end serializes without either time. An empty parameter with a nonnegative begin serializes the begin as the first token after the event name, which the parser reads as the parameter; such values do not round-trip through this text grammar.

## Rendering and output behavior

`get_sections_for_characters` returns each matching section once, in input section order, when any instant's speaker matches one of the requested names.

HTML formatting emits one title and speaker/dialogue groups per section. Empty-speaker instants are omitted. A supplied `name_map` changes displayed speaker names; `character_to_emphasize` selects the bold speaker/dialogue style. Spoken lines are grouped with line breaks; direction lines use a separate italic style. The document formatter wraps sections in a head/body and supplies inline styles, while the single-section formatter emits a keep-together section container. Events, timestamps, and embed names are not rendered. Text and mapped names are inserted without HTML escaping, so callers must not treat this output as safe for untrusted markup.

CSV formatting starts with the columns `Line #,Conversation,Character,Dialogue,Directions,Wav Filename,Conversation Filename`. It skips empty-speaker instants and combines adjacent instants from the same speaker into a row; spoken and direction text are newline-joined in separate columns. The optional name map changes the displayed speaker. `format_sections_as_csv` resets the public global `csv_line_number` to zero, so its first emitted data row is numbered 262; `format_section_as_csv` resets `csv_line_index_within_section` but does not reset `csv_line_number`, so direct calls continue from its current global value (initially zero). The per-section audio row index starts at 01; generated filenames are `<section>_<index>.wav` and `<section>.wav`. Sections with no nonempty spoken lines produce no data row, even if a speaker or direction lines are present. `csv_quote` allocates and returns a string for the caller to release; it surrounds a value with quotes and doubles embedded quotes, retaining embedded newlines. Speaker, dialogue, and direction values use this quoting; section and generated filename columns are written raw, so the formatter does not guarantee valid CSV for delimiters in those values. Exact HTML whitespace/style declarations are presentation details, not a serialized data format. The module depends on Basic, File, Hash_Table, Text_File_Handler, and String.
