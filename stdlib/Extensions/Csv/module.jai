// Reads and writes CSV with explicit limits. A record read from a [Csv_Reader] is borrowed until
// the next call; copy it to keep it. Typed rows own their strings.
//
// ```jai
// #import "Extensions/Csv";
//
// r := csv_reader(cast([] u8) "name,age\nada,36\n", .{ header = true });
// defer csv_reader_free(*r);
// fields, present, error := csv_next(*r);
// assert(present && error.kind == .NONE);
// assert(fields[0] == "ada" && fields[1] == "36");
// ```

// How a CSV document is read and written.
Csv_Options :: struct {
    delimiter: u8 = #char ","; // an ASCII byte other than the quote, NUL, CR or LF
    quote: u8 = #char "\"";
    header: bool; // treat the first record as column names; it is not returned
    trim: bool; // drop spaces around unquoted fields
    lax_quotes: bool; // allow a quote inside an unquoted field
    require_equal_width: bool; // fail with `.WIDTH` when a record's field count differs from the first
    max_field_bytes: int = 1 << 20;
    max_fields: int = 16384; // per record
    max_record_bytes: int = 16 << 20;
    line_ending := Csv_Line_Ending.CRLF; // used when writing; reading accepts all three
}

// The record terminator written by [csv_write_record].
Csv_Line_Ending :: enum {
    CRLF;
    LF;
    CR;
}

// Why reading, writing or decoding a CSV failed.
Csv_Error_Kind :: enum {
    NONE;
    OPTIONS; // invalid [Csv_Options] or a missing source callback
    QUOTE; // malformed quoting
    FIELD_LIMIT; // a field exceeds `max_field_bytes`
    FIELD_COUNT; // a record has more than `max_fields` fields, or none when writing
    RECORD_LIMIT; // a record exceeds `max_record_bytes`
    WIDTH; // a ragged record with `require_equal_width`
    SOURCE; // the chunk source failed
    TYPE; // a typed column is empty or does not parse
    HEADER; // a duplicate or missing header column
}

// The outcome of a CSV operation; `kind` is `.NONE` on success. Positions are 1-based, `offset` is
// a byte offset.
Csv_Error :: struct {
    kind: Csv_Error_Kind;
    message: string;
    line: int;
    column: int;
    offset: int;
    column_name: string; // the header column involved, for `.HEADER` and `.TYPE`
}

// A CSV document that arrives in pieces: `fill` puts the next piece in `buffer` and returns how
// many bytes it wrote (0 at the end), or a non-empty string when the source failed. `Json_Source`
// is the same type.
Csv_Source :: Pieces.Byte_Source;

// A reader over CSV bytes or a chunk source. Create it with [csv_reader], advance it with
// [csv_next] and release it with [csv_reader_free]. The fields are internal state.
Csv_Reader :: struct {
    source: Csv_Source;
    chunk: [8192] u8;
    eof: bool;
    options: Csv_Options;
    input: [] u8;
    position: int;
    offset: int;
    line: int = 1;
    line_start: int;
    previous_cr: bool;
    started: bool;
    header_ready: bool;
    header: [] string;
    header_index: Hash_Map.Map(string, int);
    columns: [..] Csv_Column;
    width: int = -1;
    buffer: [..] u8;
    cells: [..] Csv_Cell;
    fields: [..] string;
    stops: [256] u8;
    error: Csv_Error;
    record_offset: int;
    record_line: int;
    record_line_start: int;
}

// Creates a reader borrowing input; release its storage with csv_reader_free.
// Example: r := csv_reader(cast([] u8) "one,two\n", .{header = true}).
csv_reader :: (input: [] u8, options := Csv_Options.{}) -> Csv_Reader {
    r: Csv_Reader;
    r.input = input;
    r.options = options;
    r.error = validate_options(options);
    r.stops = make_stops(options);

    return r;
}

// Creates a chunked reader using Json's existing fill-buffer source convention.
// Example: r := csv_reader(Csv_Source.{*state, fill}); csv_reader_free(*r).
csv_reader :: (source: Csv_Source, options := Csv_Options.{}) -> Csv_Reader {
    r: Csv_Reader;
    r.source = source;
    r.options = options;
    r.error = validate_options(options);
    r.stops = make_stops(options);
    if !source.fill r.error = .{ kind = .OPTIONS, message = "missing source callback", line = 1, column = 1 };

    return r;
}

// Returns the next data record, or false with NONE at EOF. Header mode consumes one header.
// Example: fields, present, error := csv_next(*reader).
csv_next :: (r: *Csv_Reader) -> [] string, bool, Csv_Error {
    if r.error.kind != .NONE return .[], false, r.error;

    if r.options.header && !r.header_ready {
        header, ok := read_record(r);
        if !ok return .[], false, r.error;

        r.header = csv_record_copy(header);
        r.header_ready = true;
        for r.header {
            _, replaced := Hash_Map.insert(*r.header_index, it, it_index);
            if replaced {
                fail(r, .HEADER, "duplicate header");
                r.error.column_name = it;
                locate_error(r, it_index);
                return .[], false, r.error;
            }
        }
    }

    row, ok := read_record(r);

    return row, ok, r.error;
}

// Copies a borrowed record into independently owned fields.
// Example: owned := csv_record_copy(fields); csv_record_free(owned).
csv_record_copy :: (fields: [] string) -> [] string {
    result: [] string;
    result.count = fields.count;
    result.data = cast(*string) alloc(fields.count * size_of(string));
    for fields result[it_index] = copy_string(it);

    return result;
}

// Releases a copied record, including its field strings.
// Example: csv_record_free(owned).
csv_record_free :: (fields: [] string) {
    for fields if it.data free(it.data);
    if fields.data free(fields.data);
}

// Releases reader storage; input and source state stay caller-owned.
// Example: csv_reader_free(*reader).
csv_reader_free :: (r: *Csv_Reader) {
    array_free(r.buffer);
    array_free(r.cells);
    array_free(r.columns);
    array_free(r.fields);
    Hash_Map.deinit(*r.header_index);
    csv_record_free(r.header);
    r.header = .[];
}

// Appends one record atomically after checking the configured field and record limits.
// Example: error := csv_write_record(*builder, string.["a,b", "two"]).
csv_write_record :: (out: *String_Builder, fields: [] string, options := Csv_Options.{}) -> Csv_Error {
    error := validate_options(options);
    if error.kind != .NONE return error;
    if fields.count == 0 || fields.count > options.max_fields return .{ kind = .FIELD_COUNT, message = "invalid field count" };

    total := fields.count - 1;
    if total > options.max_record_bytes return .{ kind = .RECORD_LIMIT, message = "record exceeds limit" };

    // Which of the first fields need quotes, so the writing pass does not scan them again.
    stops := make_stops(options);
    flags: [64] bool = ---;

    for fields {
        if it.count > options.max_field_bytes return .{ kind = .FIELD_LIMIT, message = "field exceeds limit" };

        quoted, quotes := quote_need(it, stops.data, options);
        if it_index < flags.count flags[it_index] = quoted;

        extra := it.count + ifx quoted then 2 + quotes else 0;

        if extra > options.max_record_bytes - total return .{ kind = .RECORD_LIMIT, message = "record exceeds limit" };

        total += extra;
    }

    ending := ifx options.line_ending == .CRLF then "\r\n" else ifx options.line_ending == .LF then "\n" else "\r";
    if ending.count > options.max_record_bytes - total return .{ kind = .RECORD_LIMIT, message = "record exceeds limit" };

    // The record's exact size is known, so it is built in place in one piece of the builder.
    memory := cast(*u8) ensure_contiguous_space(out, total + ending.count);
    if !memory return .{};

    at := memory;

    for fields {
        if it_index {
            at.* = options.delimiter;
            at += 1;
        }

        quoted: bool;
        if it_index < flags.count quoted = flags[it_index];
        else quoted = quote_need(it, stops.data, options);

        if quoted at = copy_quoted(at, it, options.quote);
        else {
            memcpy(at, it.data, it.count);
            at += it.count;
        }
    }

    memcpy(at, ending.data, ending.count);
    advance_through_ensured_space(out, at + ending.count - memory);

    return .{};
}

#scope_module
#import "Basic";
#import "String";
#import "Compiler";
Pieces :: #import "internal/byte-source";
Hash_Map :: #import "Extensions/Hash_Map";
#load "../../internal/derive-notes/notes.jai";
#load "typed.jai";
#load "generate.jai";

// One field of the current record. `start`..`end` is where its bytes sit in `buffer`; a field read
// straight from a borrowed input is `text` instead and leaves the buffer alone. `offset`, `line`
// and `line_start` say where the field began, for typed errors.
Csv_Cell :: struct {
    offset: int;
    line: int;
    line_start: int;
    start: int;
    end: int;
    text: string;
}

// What a typed member last found in the header: the member's name and its column (-1: none).
Csv_Column :: struct {
    name: string;
    index: int;
}

// Bits of `Csv_Reader.stops`: the bytes that end a run of plain bytes in an unquoted or a quoted field.
STOP_UNQUOTED :: 1;
STOP_QUOTED :: 2;

make_stops :: (o: Csv_Options) -> [256] u8 {
    stops: [256] u8;

    stops[o.delimiter] |= STOP_UNQUOTED;
    stops[10] |= STOP_UNQUOTED | STOP_QUOTED;
    stops[13] |= STOP_UNQUOTED | STOP_QUOTED;
    stops[o.quote] |= STOP_QUOTED;
    if !o.lax_quotes stops[o.quote] |= STOP_UNQUOTED;

    return stops;
}

validate_options :: (o: Csv_Options) -> Csv_Error {
    if o.delimiter == o.quote || o.delimiter == 0 || o.quote == 0 || o.delimiter >= 128 || o.quote >= 128 ||
       o.delimiter == 10 || o.delimiter == 13 || o.quote == 10 || o.quote == 13 ||
       o.max_field_bytes < 0 || o.max_fields <= 0 || o.max_record_bytes <= 0 {
        return .{ kind = .OPTIONS, message = "invalid CSV options", line = 1, column = 1 };
    }

    return .{};
}

// Whether `field` must be quoted, and how many quote bytes it holds (each is doubled when quoted).
// `stops` is `make_stops` for the options: any entry set marks a byte that forces quoting.
quote_need :: (field: string, stops: *u8, o: Csv_Options) -> bool, int {
    hit: u8;
    quotes := 0;

    for c: field {
        hit |= stops[c];
        quotes += cast(int) (c == o.quote);
    }

    quoted := hit != 0 || (o.trim && field.count && (space(field[0]) || space(field[field.count - 1])));

    return quoted, quotes;
}

// Writes `field` in quotes at `at`, doubling each quote byte and copying the stretches between them
// whole, and returns where the next byte goes.
copy_quoted :: (at: *u8, field: string, quote: u8) -> *u8 {
    at.* = quote;
    at += 1;

    from := 0;

    for c, i: field {
        if c != quote continue;

        run := i + 1 - from;

        memcpy(at, field.data + from, run);
        at += run;
        at.* = quote;
        at += 1;
        from = i + 1;
    }

    memcpy(at, field.data + from, field.count - from);
    at += field.count - from;
    at.* = quote;

    return at + 1;
}

space :: (c: u8) -> bool {
    return c == 32 || c == 9;
}

fail :: (r: *Csv_Reader, kind: Csv_Error_Kind, message: string) -> bool {
    if r.error.kind == .NONE r.error = .{ kind = kind, message = message, line = r.line, column = r.offset - r.line_start + 1, offset = r.offset };
    return false;
}

peek :: inline (r: *Csv_Reader, ahead: int = 0) -> int {
    if r.position + ahead < r.input.count return r.input[r.position + ahead];
    return fill_to(r, ahead);
}

// The slow half of `peek`: refills the window from the source until it holds the byte `ahead`, or returns -1.
fill_to :: (r: *Csv_Reader, ahead: int) -> int {
    while r.position + ahead >= r.input.count {
        if !r.source.fill || r.eof || r.error.kind != .NONE return -1;

        left := r.input.count - r.position;
        for 0..left - 1 r.chunk[it] = r.input[r.position + it];

        r.position = 0;
        target: [] u8 = .{ r.chunk.count - left, r.chunk.data + left };
        count, error := r.source.fill(r.source.data, target);

        if error.count || count < 0 || count > target.count {
            fail(r, .SOURCE, "source read failed or returned an invalid count");
            return -1;
        }

        r.input = .{ left + count, r.chunk.data };
        if !count r.eof = true;
    }

    return r.input[r.position + ahead];
}

// Takes one byte, counting a line break when it is CR or LF (CRLF is one). Runs of other bytes go through `span`.
take :: inline (r: *Csv_Reader) -> int {
    c := peek(r);
    if c < 0 return c;

    if r.offset - r.record_offset >= r.options.max_record_bytes {
        fail(r, .RECORD_LIMIT, "record exceeds limit");
        return -1;
    }

    r.position += 1;
    r.offset += 1;

    if c == 13 {
        r.line += 1;
        r.line_start = r.offset;
        r.previous_cr = true;
    } else if c == 10 {
        if !r.previous_cr r.line += 1;
        r.line_start = r.offset;
        r.previous_cr = false;
    } else r.previous_cr = false;

    return c;
}

// The first byte in the window at or after the read position whose `stops` entry has a `mask` bit, or the window's end.
scan :: inline (r: *Csv_Reader, mask: u8) -> int {
    i := r.position;
    count := r.input.count;
    data := r.input.data;
    stops := r.stops.data;

    while i < count && !(stops[data[i]] & mask) i += 1;

    return i;
}

field_length :: inline (r: *Csv_Reader, cell: *Csv_Cell) -> int {
    return r.buffer.count - cell.start + cell.text.count;
}

append_bytes :: (r: *Csv_Reader, data: *u8, count: int) {
    needed := r.buffer.count + count;
    if needed > r.buffer.allocated array_reserve(*r.buffer, max(needed, max(64, r.buffer.allocated * 2)));

    memcpy(r.buffer.data + r.buffer.count, data, count);
    r.buffer.count = needed;
}

// Moves bytes held uncopied in `cell.text` into the buffer, before anything else is added to the field.
flush_text :: (r: *Csv_Reader, cell: *Csv_Cell) {
    if !cell.text.count return;

    append_bytes(r, cell.text.data, cell.text.count);
    cell.text = "";
}

// Adds the `count` bytes at the read position to the field. They stay in the borrowed input when they
// are all the field has; a chunked source overwrites its window, so those bytes are copied.
add_run :: inline (r: *Csv_Reader, cell: *Csv_Cell, count: int) {
    if !r.source.fill && !cell.text.count && r.buffer.count == cell.start {
        cell.text = .{ count, r.input.data + r.position };
        return;
    }

    copy_run(r, cell, count);
}

copy_run :: (r: *Csv_Reader, cell: *Csv_Cell, count: int) {
    if count <= 0 return;

    flush_text(r, cell);
    append_bytes(r, r.input.data + r.position, count);
}

consume :: inline (r: *Csv_Reader, count: int) {
    r.position += count;
    r.offset += count;
    r.previous_cr = false;
}

// Takes `count` bytes with no CR or LF in them into the field, failing where taking them one at a
// time would: the record limit on the byte past it, the field limit on the byte that does not fit.
span :: inline (r: *Csv_Reader, cell: *Csv_Cell, count: int) -> bool {
    record_room := r.options.max_record_bytes - (r.offset - r.record_offset);
    field_room := r.options.max_field_bytes - field_length(r, cell);

    if count <= record_room && count <= field_room {
        add_run(r, cell, count);
        consume(r, count);
        return true;
    }

    return span_at_limit(r, cell, record_room, field_room);
}

// `span` for a run that does not fit: take what does, then fail.
span_at_limit :: (r: *Csv_Reader, cell: *Csv_Cell, record_room: int, field_room: int) -> bool {
    if record_room <= field_room {
        add_run(r, cell, record_room);
        consume(r, record_room);
        return fail(r, .RECORD_LIMIT, "record exceeds limit");
    }

    add_run(r, cell, field_room);
    consume(r, field_room + 1);

    return fail(r, .FIELD_LIMIT, "field exceeds limit");
}

put :: (r: *Csv_Reader, cell: *Csv_Cell, c: u8) -> bool {
    if field_length(r, cell) >= r.options.max_field_bytes return fail(r, .FIELD_LIMIT, "field exceeds limit");

    flush_text(r, cell);
    array_add(*r.buffer, c);

    return true;
}

read_field :: (r: *Csv_Reader) -> bool {
    cell := Csv_Cell.{ offset = r.offset, line = r.line, line_start = r.line_start, start = r.buffer.count };
    quote := r.options.quote;
    quoted := peek(r) == quote;

    if quoted {
        if take(r) < 0 return false;

        while true {
            if r.position >= r.input.count && peek(r) < 0 {
                fail(r, .QUOTE, "unterminated quoted field");
                return false;
            }

            end := scan(r, STOP_QUOTED);
            if end > r.position {
                if !span(r, *cell, end - r.position) return false;
                if r.position >= r.input.count continue;
            }

            c := r.input[r.position];

            if take(r) < 0 return false;

            if c == quote {
                if peek(r) != quote break;
                if take(r) < 0 return false;
            }

            if !put(r, *cell, c) return false;
        }

        if r.options.trim while peek(r) != r.options.delimiter && space(cast(u8) max(0, peek(r))) {
            if take(r) < 0 return false;
        }

        c := peek(r);
        if c >= 0 && c != r.options.delimiter && c != 10 && c != 13 {
            fail(r, .QUOTE, "data after closing quote");
            return false;
        }
    } else {
        if r.options.trim while true {
            c := peek(r);
            if c < 0 || (r.stops[c] & STOP_UNQUOTED) || !space(cast(u8) c) break;
            if take(r) < 0 return false;
        }

        while !(r.position >= r.input.count && peek(r) < 0) {
            end := scan(r, STOP_UNQUOTED);
            if end > r.position {
                if !span(r, *cell, end - r.position) return false;
                if r.position >= r.input.count continue;
            }

            if r.input[r.position] == quote && !r.options.lax_quotes {
                fail(r, .QUOTE, "quote in unquoted field");
                return false;
            }

            break;
        }

        if r.options.trim {
            while cell.text.count && space(cell.text[cell.text.count - 1]) cell.text.count -= 1;
            while r.buffer.count > cell.start && space(r.buffer[r.buffer.count - 1]) r.buffer.count -= 1;
        }
    }

    if r.error.kind != .NONE return false;

    cell.end = r.buffer.count;

    if r.cells.count >= r.cells.allocated array_reserve(*r.cells, max(16, r.cells.allocated * 2));

    r.cells.data[r.cells.count] = cell;
    r.cells.count += 1;

    return true;
}

read_record :: (r: *Csv_Reader) -> [] string, bool {
    r.buffer.count = 0;
    r.cells.count = 0;
    r.fields.count = 0;
    if !r.started {
        r.started = true;
        if peek(r) == 239 && peek(r, 1) == 187 && peek(r, 2) == 191 {
            r.position = 3;
            r.offset = 3;
            r.line_start = 3;
        }
    }

    r.record_offset = r.offset;
    r.record_line = r.line;
    r.record_line_start = r.line_start;
    if peek(r) < 0 return .[], false;

    while true {
        if r.cells.count >= r.options.max_fields {
            fail(r, .FIELD_COUNT, "too many fields");
            return .[], false;
        }

        if !read_field(r) return .[], false;

        delimiter := peek(r);
        if delimiter == r.options.delimiter {
            if r.offset - r.record_offset >= r.options.max_record_bytes {
                fail(r, .RECORD_LIMIT, "record exceeds limit");
                return .[], false;
            }

            consume(r, 1);
            continue;
        }

        if delimiter == 10 || delimiter == 13 {
            if take(r) < 0 return .[], false;
            if delimiter == 13 && peek(r) == 10 if take(r) < 0 return .[], false;
        }

        break;
    }

    if r.error.kind != .NONE return .[], false;

    array_reserve(*r.fields, r.cells.count);
    r.fields.count = r.cells.count;

    // Quoted whitespace is data: trimming applies only while reading unquoted fields.
    for r.cells {
        if it.end > it.start r.fields[it_index] = .{ it.end - it.start, r.buffer.data + it.start };
        else r.fields[it_index] = it.text;
    }

    if r.width < 0 r.width = r.fields.count;
    else if r.options.require_equal_width && r.width != r.fields.count {
        fail(r, .WIDTH, "ragged record");
        return .[], false;
    }

    return r.fields, true;
}
