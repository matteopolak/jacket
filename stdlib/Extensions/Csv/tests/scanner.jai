// The reader finds delimiters, quotes and line breaks 64 bytes at a time. These tests put every
// kind of byte on both sides of a 64-byte boundary, at every offset, and check that the result does
// not depend on where the window starts: slice input and chunked input (whose windows start
// elsewhere) must agree on records, error kind, offset, line and column.
#import "Basic";
#import "String";
#import "Extensions/Csv";
#import "Extensions/Testing";

Feed :: struct {
    text: string;
    at: int;
    size: int;
}

feed_fill :: (data: *void, buffer: [] u8) -> s64, string {
    feed := cast(*Feed) data;
    count := min(buffer.count, min(feed.size, feed.text.count - feed.at));

    if count > 0 memcpy(buffer.data, feed.text.data + feed.at, count);

    feed.at += count;

    return count, "";
}

// Every record and the final error of a reader, as one string.
outcome :: (reader: *Csv_Reader) -> string {
    push_allocator(temp);

    builder: String_Builder;

    while true {
        row, ok, error := csv_next(reader);

        if !ok {
            print_to_builder(*builder, "end % at % line % column %", cast(int) error.kind, error.offset, error.line, error.column);
            break;
        }

        for row print_to_builder(*builder, "[%]", it);

        append(*builder, "\n");
    }

    return builder_to_string(*builder);
}

slice_outcome :: (text: string, options: Csv_Options) -> string {
    reader := csv_reader(cast([] u8) text, options);

    defer csv_reader_free(*reader);

    return outcome(*reader);
}

chunk_outcome :: (text: string, options: Csv_Options, size: int) -> string {
    feed := Feed.{ text = text, size = size };
    reader := csv_reader(Csv_Source.{ *feed, feed_fill }, options);

    defer csv_reader_free(*reader);

    return outcome(*reader);
}

CHUNK_SIZES :: int.[1, 5, 63, 64, 65, 8192];

expect_same_everywhere :: (text: string, options := Csv_Options.{}) {
    want := slice_outcome(text, options);

    for CHUNK_SIZES {
        got := chunk_outcome(text, options, it);

        if got != want {
            print("chunks of % differ on %\n--- slice\n%\n--- chunks\n%\n", it, text, want, got);
            expect(false);
            return;
        }
    }
}

repeat :: (c: u8, count: int) -> string {
    push_allocator(temp);

    text := alloc_string(count);

    for *text it.* = c;

    return text;
}

// A record array as CSV text. A field is quoted when it needs to be or when `quote_all` is set.
serialize :: (records: [][] string, ending: string, quote_all: bool, final_ending: bool) -> string {
    push_allocator(temp);

    builder: String_Builder;

    for record, row: records {
        for field, column: record {
            if column append(*builder, ",");

            needs := quote_all || (record.count == 1 && !field.count);

            for c: field if c == #char "," || c == #char "\"" || c == 13 || c == 10 needs = true;

            if !needs {
                append(*builder, field);
                continue;
            }

            append(*builder, "\"");

            for c: field {
                if c == #char "\"" append(*builder, "\"");
                append(*builder, string.{ 1, *c });
            }

            append(*builder, "\"");
        }

        if row < records.count - 1 || final_ending append(*builder, ending);
    }

    return builder_to_string(*builder);
}

expect_records :: (text: string, records: [][] string) {
    reader := csv_reader(cast([] u8) text);

    defer csv_reader_free(*reader);

    for record: records {
        row, ok, error := csv_next(*reader);

        if !ok || error.kind != .NONE || row.count != record.count {
            print("record mismatch in %\n", text);
            expect(false);
            return;
        }

        for field, i: record {
            if row[i] != field {
                print("field % is [%], want [%] in %\n", i, row[i], field, text);
                expect(false);
                return;
            }
        }
    }

    _, ok, error := csv_next(*reader);

    expect(!ok && error.kind == .NONE);
}

// Quotes, doubled quotes, delimiters and line breaks around byte 63 and 64, at every offset.
straddling_blocks :: () {
    push_allocator(temp);

    for pad: 0..200 {
        run := repeat(#char "x", pad);

        both := tprint("%\"", run);
        records := [] string.[
            string.[run, "q\"q", "plain"],
            string.["multi\r\nline", both, ""],
            string.["", "", "", ""],
            string.["a\"\"", ",", "\"", "\n"],
            string.["x\ry", "\r\n", run],
        ];

        for 0..3 {
            ending := ifx it == 0 then "\r\n" else ifx it == 1 then "\n" else "\r";
            text := serialize(records, ending, it == 3, it != 2);

            expect_records(text, records);

            for size: CHUNK_SIZES {
                feed := Feed.{ text = text, size = size };
                reader := csv_reader(Csv_Source.{ *feed, feed_fill });

                expect_records_from(*reader, records);
                csv_reader_free(*reader);
            }
        }
    }
} @test

expect_records_from :: (reader: *Csv_Reader, records: [][] string) {
    for record: records {
        row, ok, error := csv_next(reader);

        require(ok && error.kind == .NONE && row.count == record.count);

        for field, i: record expect_equal(row[i], field);
    }

    _, ok, error := csv_next(reader);

    expect(!ok && error.kind == .NONE);
}

// A field longer than 64 bytes, a whole block of delimiters and a quoted field spanning blocks.
long_fields_and_delimiter_blocks :: () {
    push_allocator(temp);

    for length: 60..140 {
        run := repeat(#char "y", length);

        text := tprint("%,%\n\"%\",\"%\"\"%\"\n", run, run, run, run, run);
        records := [] string.[string.[run, run], string.[run, tprint("%\"%", run, run)]];

        expect_records(text, records);
    }

    commas := repeat(#char ",", 200);

    row, ok, error := csv_next(*csv_reader(cast([] u8) commas));

    expect(ok && error.kind == .NONE && row.count == 201);

    for lead: 0..70 {
        text := tprint("%%\n%\n", repeat(#char "a", lead), commas, commas);
        reader := csv_reader(cast([] u8) text);

        defer csv_reader_free(*reader);

        first, first_ok, first_error := csv_next(*reader);

        require(first_ok && first_error.kind == .NONE);
        expect_equal(first.count, 201);
        expect_equal(first[0].count, lead);

        second, second_ok, second_error := csv_next(*reader);

        require(second_ok && second_error.kind == .NONE);
        expect_equal(second.count, 201);
    }
} @test

// Error positions around block boundaries.
error_positions :: () {
    push_allocator(temp);

    for pad: 0..140 {
        run := repeat(#char "x", pad);

        // A quote inside an unquoted field.
        text := tprint("a,q%\"", run);
        reader := csv_reader(cast([] u8) text);

        _, ok, error := csv_next(*reader);

        expect(!ok && error.kind == .QUOTE);
        expect_equal(error.offset, 3 + pad);
        expect_equal(error.line, 1);
        expect_equal(error.column, 4 + pad);
        csv_reader_free(*reader);

        // Data after a closing quote, after a CRLF inside the quotes.
        text = tprint("\"a\r\n%\"b", run);
        reader = csv_reader(cast([] u8) text);

        _, ok, error = csv_next(*reader);

        expect(!ok && error.kind == .QUOTE);
        expect_equal(error.offset, pad + 5);
        expect_equal(error.line, 2);
        expect_equal(error.column, pad + 2);
        csv_reader_free(*reader);

        // An unterminated quote at the end.
        text = tprint("\"%\r", run);
        reader = csv_reader(cast([] u8) text);

        _, ok, error = csv_next(*reader);

        expect(!ok && error.kind == .QUOTE);
        expect_equal(error.offset, pad + 2);
        expect_equal(error.line, 2);
        expect_equal(error.column, 1);
        csv_reader_free(*reader);

        expect_same_everywhere(tprint("%\r\n%\"", run, run));
        expect_same_everywhere(tprint("\"%\"\"\r\n%\"\"\"", run, run));
    }
} @test

// Limits reached in the middle of a block stop on the same byte as in a one-byte chunk.
limits_mid_block :: () {
    push_allocator(temp);

    for pad: 0..140 {
        run := repeat(#char "z", pad);

        for text: string.[tprint("%,%\n", run, run), tprint("\"%\"\"%\"\n", run, run), tprint("%,,,,%\n", run, run)] {
            for limit: 0..4 {
                expect_same_everywhere(text, .{ max_field_bytes = pad + limit - 2 });
                expect_same_everywhere(text, .{ max_record_bytes = 2 * pad + limit + 2 });
                expect_same_everywhere(text, .{ max_fields = limit + 1 });
            }
        }
    }
} @test

Rng :: struct {
    state: u64 = 88172645463325252;
}

rng_below :: (r: *Rng, n: int) -> int {
    x := r.state;

    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    r.state = x;

    return cast(int) (x % cast(u64) n);
}

random_text :: (rng: *Rng, parts: [] string, pieces: int) -> string {
    push_allocator(temp);

    builder: String_Builder;

    for 0..pieces - 1 {
        pick := rng_below(rng, parts.count + 1);

        if pick == parts.count {
            run := repeat(#char "x", rng_below(rng, 150));

            append(*builder, run);
        } else append(*builder, parts[pick]);
    }

    return builder_to_string(*builder);
}

// Random well-formed documents read back as the records they were made from.
random_documents :: () {
    push_allocator(temp);

    rng: Rng;
    parts := string.[",", "\"", "\r", "\n", "\r\n", " ", "\xc3\xa9", "a", "\"\""];

    for 0..299 {
        records: [..][] string;

        for 1..1 + rng_below(*rng, 6) {
            fields: [..] string;

            for 1..1 + rng_below(*rng, 5) array_add(*fields, random_text(*rng, parts, rng_below(*rng, 6)));

            array_add(*records, fields);
        }

        ending := ifx rng_below(*rng, 3) == 0 then "\r\n" else ifx rng_below(*rng, 2) == 0 then "\n" else "\r";
        text := serialize(records, ending, rng_below(*rng, 4) == 0, rng_below(*rng, 2) == 0);

        expect_records(text, records);

        want := slice_outcome(text, .{});

        for CHUNK_SIZES expect_equal(chunk_outcome(text, .{}, it), want);
    }
} @test

// Random byte soup, most of it malformed: every option set gives one answer for every window.
random_garbage :: () {
    push_allocator(temp);

    rng: Rng;

    rng.state = 1234567;

    parts := string.[",", "\"", "\r", "\n", "\r\n", " ", "\t", ";", "a", "\"\"", " \"", "\",", "\"\r\n"];
    variants := Csv_Options.[
        .{},
        .{ lax_quotes = true },
        .{ trim = true },
        .{ trim = true, lax_quotes = true },
        .{ max_field_bytes = 5 },
        .{ max_record_bytes = 40 },
        .{ max_fields = 3 },
        .{ delimiter = #char ";" },
        .{ require_equal_width = true },
    ];

    for 0..299 {
        text := random_text(*rng, parts, 1 + rng_below(*rng, 60));

        for variants expect_same_everywhere(text, it);
    }
} @test

// A field and a record much longer than the 8192-byte chunk window, with doubled quotes and line
// breaks inside, read from chunks of an awkward size.
spans_windows :: () {
    push_allocator(temp);

    builder: String_Builder;

    append(*builder, "head,\"");
    for 0..299 append(*builder, "line of text\r\n with \"\"quotes\"\" and, commas ");

    append(*builder, "\",tail\r\nnext,row\n");

    text := builder_to_string(*builder);

    expect_same_everywhere(text);
    expect_same_everywhere(text, .{ max_field_bytes = 5000 });
    expect_same_everywhere(text, .{ max_record_bytes = 9000 });
} @test

Triple :: struct {
    a: int;
    b: string;
    c: int;
}

// A typed error after a line break inside quotes is placed by line and column, for a slice and a
// chunk source alike.
typed_error_after_quoted_break :: () {
    text := "1,\"q\r\nr\",x\n";
    slice := csv_reader(Triple, cast([] u8) text);

    defer csv_reader_free(*slice);
    feed := Feed.{ text = text, size = 3 };
    chunked := csv_reader(Triple, Csv_Source.{ *feed, feed_fill });

    defer csv_reader_free(*chunked);

    _, present, error := csv_next(*slice);

    expect(!present && error.kind == .TYPE);
    expect_equal(error.offset, 9);
    expect_equal(error.line, 2);
    expect_equal(error.column, 4);

    _, present, error = csv_next(*chunked);

    expect(!present && error.kind == .TYPE);
    expect_equal(error.offset, 9);
    expect_equal(error.line, 2);
    expect_equal(error.column, 4);
} @test
