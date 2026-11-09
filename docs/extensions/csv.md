# Csv

`#import "Extensions/Csv";` provides bounded RFC 4180 record reading, writing and
compile-time struct codecs. Delimiter and quote are single ASCII bytes; LF, CRLF
and CR input endings are accepted, and an initial UTF-8 BOM is skipped.

## Reading records

```jai
r := csv_reader(cast([] u8) "name,age\r\nAda,36\r\n", .{header = true});
defer csv_reader_free(*r);
fields, present, error := csv_next(*r);
assert(present && error.kind == .NONE && fields[0] == "Ada");
```

The reader keeps one decoded record in reusable storage. Fields and their array
are views valid until the next call; `csv_record_copy` makes independent strings
and an array, released with `csv_record_free`. The input slice remains borrowed.
A final newline does not produce another record; an actual blank line is one
empty field. Header mode owns a copy of the first record, exposes it as
`reader.header`, and rejects duplicate names.

Chunked input uses `Csv_Source`, an alias of Json's existing source type:
`fill(data: *void, buffer: [] u8) -> s64, string`. Fill the supplied buffer,
return the byte count (zero for EOF) and an empty error string on success.
A nonempty error, negative count or count larger than the buffer gives `SOURCE`.
Deflate in this checkout has no chunked input interface; reusing Json avoids
introducing a second callback convention.

```jai,ignore
r := csv_reader(Csv_Source.{*state, fill}, .{header = true});
```

The reader requests at most 8192 bytes at once and handles tokens split at any
byte boundary. Keep its address stable after reading starts: its input can point
into its own chunk buffer. It neither closes nor frees the source. Memory is
bounded by one record, field metadata, the optional header and one chunk;
the complete input is never gathered.

## Options and errors

`Csv_Options` defaults to comma, double quote, no header, no trimming, strict
quotes, unconstrained row width and CRLF output. Strict quotes are the default
because stray quotes usually indicate damaged data. Set `lax_quotes = true`
to treat quotes inside unquoted fields literally, matching Python csv's default
handling of those fields. Text after a closing quote remains an error.

`trim = true` strips ASCII spaces and tabs from unquoted fields and accepts them
after a closing quote. Quoted whitespace remains data. `require_equal_width`
compares all records with the first record, including a header.

Limits are `max_field_bytes = 1 << 20` (decoded bytes),
`max_fields = 16384` and `max_record_bytes = 16 << 20` (wire bytes including
delimiters, quotes and the line ending, excluding the initial BOM). Trimming can
discard leading whitespace before the decoded-field limit; the record limit
still bounds that work. Delimiter and quote must differ and cannot be NUL, CR,
LF or non-ASCII. Limits must be nonnegative, with positive field count and record
size. Input is UTF-8 bytes; no encoding conversion or Unicode normalization occurs.

`Csv_Error` contains a kind, static message, zero-based byte offset and one-based
line and byte column. CRLF counts as one line break, including inside quotes.
Typed errors also name the column. Error text is borrowed; header-derived
column names last until the reader is freed. EOF is `present = false` with
`NONE`; failures are terminal for that reader.

Kinds are `OPTIONS`, `QUOTE`, `FIELD_LIMIT`, `FIELD_COUNT`, `RECORD_LIMIT`,
`WIDTH`, `SOURCE`, `TYPE` and `HEADER`. No errors own storage.

## Writing and typed rows

`csv_write_record(*builder, fields, options)` validates before appending. It
quotes fields containing the delimiter, quote, CR or LF, doubles quotes, and
protects surrounding whitespace when trimming is enabled. `line_ending` selects
`CRLF`, `LF` or `CR`. An empty record array is rejected; one empty field is
written as an empty line.

```jai
Person :: struct {
    name: string; @csv(name=full-name)
    age: int;
    nickname: *string;
    local: int; @csv(skip)
}
r := csv_reader(cast([] u8) "full-name,age,nickname\nAda,36,\n", .{header = true});
defer csv_reader_free(*r);
person, present, error := csv_next_row(Person, *r);
assert(present && error.kind == .NONE && person.nickname == null);
csv_row_free(*person);
```

Without a header, fields follow declaration order, excluding constants and
skipped members. With a header, matching uses a hash map built once per reader.
Extra columns are permitted; missing required columns are errors. Integers are
range-checked, floats retain round-trip precision when written, booleans use
`true`/`false`, and decoded strings are owned. Pointers to these scalar types
are optional: an empty cell becomes null. An empty nonoptional number fails.
Declared initializers are preserved when failure occurs before a field is read;
`csv_row_free` leaves default storage alone and resets decoded members.

`csv_write_rows(*builder, rows, options)` always emits a header, then structs.
It appends records individually, so failure may leave earlier records in the
builder. Each typed value and optional pointer is released with `csv_row_free`;
skipped fields remain caller-owned.

## Changing and checking Csv

The record parser, options and writer live in `module.jai`; scalar conversions
and typed ownership live in `typed.jai`. `generate.jai` emits per-struct field
accesses through `#insert`, reusing the shared derive-note splitter for
`@csv(name=...)` and `@csv(skip)`. Extend the supported-type check, scalar
codec, freer and regression tests together when adding a field type.
Dependencies are Basic, String, Compiler, Json and Extensions/Hash_Map;
there are no external runtime libraries.

```sh
PYTHONPATH=tools python3 tools/portable/build_portable.py data
PYTHONPATH=tools python3 tools/portable/build_portable.py bench
jacket test --path stdlib/Extensions/Csv
jacket test --path stdlib/Extensions/Csv --interp
jacket bench --path stdlib/Extensions/Csv
PYTHONPATH=tools python3 tools/csv_conformance/bench.py
```

The portable data step calls `tools/csv_conformance/fetch.py`, verifies a pinned
csv-spectrum archive and extracts CSV/JSON fixtures under
`target/csv-conformance`; the bench step writes a ~50 MB CSV beside them.
`JACKET_OUTPUT_DIR` relocates fixtures and output; `XDG_CACHE_HOME`
relocates the download cache. Benchmarks are in `benches/bench.jai`.

The reader scans runs of plain bytes in the current window with a 256-entry table
(`Csv_Reader.stops`, built from the options) and handles only delimiters, quotes
and line breaks one byte at a time. A run is checked against the field and record
limits once, and fails at the same byte a byte-by-byte reader would. Line and
column are not tracked per byte: a reader keeps the line number and where the
line began, and each field remembers both for typed errors. When the input is a
slice, a field with no doubled quotes is a view of the input; otherwise, and for
every chunked source, its bytes are copied into one reused buffer. The writer
checks each field once, sizes the record exactly and builds it in place in the
builder. Change the scanning in `read_field`, `span` and `take` together; the
chunk and limit tests in `tests/streaming.jai` compare slice and chunked input
byte for byte, including error positions.

The spectrum test compares all twelve CSV/JSON pairs with no skipped cases.
At the pinned revision, `location_coordinates.json` gives a different phone
number than its CSV, so `fetch.py` corrects that one value. Unit tests cover strict rejection, lax quotes, limits, round trips,
chunk boundaries, typed conversions and allocation balance.

## Compared with the community libraries

`benchmarks/compare/csv/benches/csv.jai` (`tools/bench/library_compare.py csv`, see [library comparisons](../tools/library-comparisons.md)) runs Csv and [rluba/jai-csv](https://github.com/rluba/jai-csv) on the ~50 MB mixed file (about a million rows, LF endings so jai-csv can read it) and on 100,000 records written (Apple M5, one thread):

| Workload | Csv | jai-csv |
| --- | --- | --- |
| records (parse all) | 75 ms, 734 MB/s | 80 ms, 689 MB/s |
| typed rows | 129 ms, 427 MB/s | 122 ms, 451 MB/s |
| write | 7.0 ms, 788 MB/s | 8.2 ms, 668 MB/s |

Measured with a one-minute load average of about 9 on the 10 cores, both libraries in the same run; the spread within a run was under 1%. Csv parses records about 7% faster and writes about 15% faster; typed rows are about 5% slower; the reader is not the difference there, decoding the members is. jai-csv does less: it keeps no position, has no limits and takes the whole input at once. jai-csv has no record reader, so "records" calls its `csv_consume_col` per field, and "typed" uses `csv_parse` into the same struct without the `bool` column, which it can't read. Both libraries produce the same counts, rows and written bytes, checked before the timer.

| Feature | Csv | jai-csv |
| --- | --- | --- |
| Streaming input | yes, `Csv_Source` chunks | no, whole string |
| Quoted newlines | yes | yes |
| CRLF and CR endings | yes | no, only LF (`\r` stays in the last field) |
| BOM | skipped | kept in the first field |
| Limits | field, field count, record | none |
| Error positions | kind, offset, line, column | log message only |
| Typed rows | all scalars, `bool`, optional, rename and skip notes | string, integer and float via a column callback |
| Writer | records and typed rows, line ending choice, whitespace protection | `csv_escape` per field, no record writer, `\r` not quoted |
| Lax quotes | option | no, stray quotes fail |

On the csv-spectrum cases Csv matches all twelve (stray-quote `location_coordinates` with `lax_quotes`). jai-csv, read through `csv_consume_col`, matches eight: it fails on `empty_crlf` and `location_coordinates` (quote errors) and leaves `\r` in the last field of `simple_crlf` and `newlines_crlf`.

Extend `csv.jai` by adding a `WORKLOAD__LIBRARY` pair and a matching check in `prepare`.
