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

A reader's own storage (header, column map, buffers) comes from the allocator that
was current when it was created, and `csv_reader_free` releases it with that one, so
a call to `csv_next(*r,, temp)` only moves the decoded row to another allocator.

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

`borrow = true` makes a typed row's strings (and the strings behind its `*string`
members) slices of the input when the field needed no unquoting, and copies when it
had doubled quotes. It saves one allocation per string field. It needs a slice input:
`csv_reader(Row, source, ...)` with `borrow` fails with `OPTIONS`, because a chunk
buffer is reused. Untyped records are borrowed already, so it changes nothing there.
The input must outlive the rows, and a borrowed row must not go through
`csv_row_free(*row)`: decode into `temp` or an arena and reset that, or release a row
with `csv_row_free(*reader, *row)`, which frees only the optional pointers.

```jai
reader := csv_reader(Person, input, .{header = true, borrow = true});
defer csv_reader_free(*reader);
person, present, error := csv_next(*reader,, temp);
// person.name points into `input`
reset_temporary_storage();
```

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
r := csv_reader(Person, cast([] u8) "full-name,age,nickname\nAda,36,\n", .{header = true});
defer csv_reader_free(*r);
person, present, error := csv_next(*r);
assert(present && error.kind == .NONE && person.nickname == null);
csv_row_free(*person);
```

The row type belongs to the reader: `Csv_Reader(Row)`, made by `csv_reader(Row, input, options)`
or `csv_reader(Row, source, options)`. `csv_next(*r)` then returns `Row, bool, Csv_Error`;
a reader without a row type (`Csv_Reader`, i.e. `Csv_Reader(void)`) returns `[] string, bool, Csv_Error`.
`csv_reader_free` and the writers work for both. A reader is also iterable, and
`reader.error` says afterwards why the loop ended (`.NONE` at the end of the input):

```jai
for person, index: r {
    defer csv_row_free(*person);
    print("%: %\n", index, person.name);
}
if r.error.kind != .NONE print("%\n", r.error.message);
```

`Csv_Reader(Row)` wraps the untyped parser state (`Csv_Core`) through `#as using`, so
its fields (`error`, `header`, `options`) read the same on both kinds.

Without a header, fields follow declaration order, excluding constants and
skipped members. With a header, the reader looks each member up in a hash map
once, when its first row is read, and every later row reads by column number.
A member missing from the header fails with `HEADER` on the first row, not on
a header-only document.
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

A slice input is read a record at a time by `read_record_fast`. Every 64 bytes
of the window get two bitmasks from `load_block` (`Extensions/SIMD`, one 64-byte
compare per byte class): the bytes that end an unquoted field (delimiter, CR, LF,
and the quote unless `lax_quotes`) and those that end a quoted run (quote, CR,
LF). `scan_from` takes the next set bit with a count of trailing zeros, so a
field costs one bit scan instead of a loop over its bytes, and one block serves
about six fields. The masks are cached per block and dropped when a chunked
source refills its window. The simdcsv-style prefix XOR of the quote mask is not
used: it assumes well-formed quotes, and the reader must also report quote
errors by position, honour `lax_quotes` and stop at the exact byte of a limit.
Following the quote state field by field gives the speed without those
exceptions. A vector scan that read 16 bytes per field was slower (about
130 ms against 85 ms for records): fields average ten bytes.

The fast path keeps a field as a view of the input, or decodes it into the reused
buffer when it holds doubled quotes. It leaves the reader untouched until the
whole record is read and within its limits; anything else (a quote error, a
limit, EOF inside quotes) makes `read_record` read the record again byte by
byte, which also reports the error. Chunked sources and `trim` always use that
byte path, with the same masks finding runs. Line and column are not tracked
per byte: a reader keeps the line number and where the line began, advanced at
line breaks (CRLF counts once), and each field remembers both for typed errors.
The fields of a chunked source are copied into one reused buffer. The writer
checks each field once, sizes the record exactly and builds it in place in the
builder. Change `read_record_fast` and `read_field`, `span` and `take` together:
`tests/scanner.jai` puts every byte class on each side of a block boundary and
compares slice input (fast path) with chunked input (byte path) on random
documents and random byte soup, and the chunk and limit tests in
`tests/streaming.jai` compare them byte for byte, including error positions.

The spectrum test compares all twelve CSV/JSON pairs with no skipped cases.
At the pinned revision, `location_coordinates.json` gives a different phone
number than its CSV, so `fetch.py` corrects that one value. Unit tests cover strict rejection, lax quotes, limits, round trips,
chunk boundaries, typed conversions and allocation balance.

## Compared with the community libraries

`benchmarks/compare/csv/benches/csv.jai` (`tools/bench/library_compare.py csv`, see [library comparisons](../tools/library-comparisons.md)) runs Csv and [rluba/jai-csv](https://github.com/rluba/jai-csv) on two ~50 MB files of a million rows (LF endings so jai-csv can read them) and on 100,000 records written (Apple M5, one thread). "Quoted" repeats one row of an integer, a quoted name with a comma, a float, a bool and a quoted field with doubled quotes. "Plain" is closer to exported data: varied ids, names, numbers and notes, with one row in twenty holding a quoted name with a comma.

| Workload | Csv | Csv with `borrow` | jai-csv |
| --- | --- | --- | --- |
| records, quoted | 48 ms, 1140 MB/s | | 83 ms, 666 MB/s |
| typed rows, quoted | 97 ms, 569 MB/s | 92 ms, 601 MB/s | 122 ms, 453 MB/s |
| records, plain | 36 ms, 1420 MB/s | | 53 ms, 974 MB/s |
| typed rows, plain | 88 ms, 590 MB/s | 62 ms, 832 MB/s | 95 ms, 546 MB/s |
| write | 6.6 ms, 839 MB/s | | 7.7 ms, 718 MB/s |

One run of both libraries at a one-minute load average of 5 to 9 on the 10 cores; the spread within a run was under 3% except for typed rows with `borrow`, and jai-csv varied between runs (records 75 to 83 ms on the quoted file). Before the block scanner the same benchmark gave 74 ms (746 MB/s) and 64 ms (805 MB/s) for records, 120 ms and 105 ms for typed rows, so records now parse 1.5x to 1.8x faster than before and 1.5x to 1.7x faster than jai-csv; typed rows are about 1.2x faster than before and 1.1x to 1.3x faster than jai-csv, and `borrow` saves another 5% to 30% (more on the plain file, where fewer strings need copying). Typed rows spend most of their time decoding members, not scanning, so they gained less. Writing is unchanged, 1.2x faster than jai-csv. The reader maps header names once, so a row only decodes its members. jai-csv does less: it keeps no position, has no limits and takes the whole input at once. jai-csv has no record reader, so "records" calls its `csv_consume_col` per field, and "typed" uses `csv_parse` into the same struct without the `bool` column, which it can't read. Both libraries produce the same counts, rows and written bytes on both files, checked before the timer.

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
