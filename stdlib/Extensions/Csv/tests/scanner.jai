// The reader finds delimiters, quotes and line breaks 64 bytes at a time. These tests put every
// kind of byte on both sides of a 64-byte boundary, at every offset, and check that the result does
// not depend on where the window starts: slice input and chunked input (whose windows start
// elsewhere) must agree on records, error kind, offset, line and column, and with a reader that
// takes every record byte by byte (`bytewise`), which is the reference for the block scanner.
#import "Basic";
#import "String";
#import "Extensions/Csv";
#import "Extensions/Testing";

Feed :: struct {
    text: string;
    at: int;
    size: int;
    fail_after: int = -1; // the source fails once it has given this many bytes (-1: never)
}

feed_fill :: (data: *void, buffer: [] u8) -> s64, string {
    feed := cast(*Feed) data;
    count := min(buffer.count, min(feed.size, feed.text.count - feed.at));

    if feed.fail_after >= 0 {
        if feed.at >= feed.fail_after return 0, "the source failed";

        count = min(count, feed.fail_after - feed.at);
    }

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

slice_outcome :: (text: string, options: Csv_Options, bytewise := false) -> string {
    reader := csv_reader(cast([] u8) text, options);

    defer csv_reader_free(*reader);
    reader.bytewise = bytewise;

    return outcome(*reader);
}

chunk_outcome :: (text: string, options: Csv_Options, size: int, bytewise := false) -> string {
    feed := Feed.{ text = text, size = size };
    reader := csv_reader(Csv_Source.{ *feed, feed_fill }, options);

    defer csv_reader_free(*reader);
    reader.bytewise = bytewise;

    return outcome(*reader);
}

// A source that fails after `fail_after` bytes: the records before the failure, and where the
// failure is reported, are the same whether the block scanner or the byte-by-byte path reads them.
failing_outcome :: (text: string, options: Csv_Options, size: int, fail_after: int, bytewise: bool) -> string {
    feed := Feed.{ text = text, size = size, fail_after = fail_after };
    reader := csv_reader(Csv_Source.{ *feed, feed_fill }, options);

    defer csv_reader_free(*reader);
    reader.bytewise = bytewise;

    return outcome(*reader);
}

source_failure_positions :: () {
    push_allocator(temp);

    texts := string.[
        "a,b\nc,d\ne,f\n",
        "a,\"b\r\nc\",d\r\ne,\"f\"\"g\"\r\n",
        "one,two\nthree,\"four\nfive\nsix\"\nseven",
        tprint("%,%\n\"%\",x\n", repeat(#char "m", 70), repeat(#char "n", 130), repeat(#char "q", 100)),
    ];

    for text: texts {
        for options: Csv_Options.[.{}, .{ trim = true }] {
            for size: int.[1, 7, 64, 8192] {
                for fail_after: 0..text.count {
                    want := failing_outcome(text, options, size, fail_after, true);
                    got := failing_outcome(text, options, size, fail_after, false);

                    if got != want {
                        print("failing after % of % in pieces of %\n--- bytewise\n%\n--- blocks\n%\n", fail_after, text, size, want, got);
                        expect(false);
                        return;
                    }
                }
            }
        }
    }
} @test

// Windows of one byte, around a block, and larger than the reader's own 8192-byte window.
CHUNK_SIZES :: int.[1, 5, 63, 64, 65, 8192, 100000];

expect_same_everywhere :: (text: string, options := Csv_Options.{}) {
    want := slice_outcome(text, options);
    reference := slice_outcome(text, options, true);

    if reference != want {
        print("bytewise differs on %\n--- slice\n%\n--- bytewise\n%\n", text, want, reference);
        expect(false);
        return;
    }

    for size: int.[1, 65, 8192] {
        got := chunk_outcome(text, options, size, true);

        if got != want {
            print("bytewise chunks of % differ on %\n--- slice\n%\n--- chunks\n%\n", size, text, want, got);
            expect(false);
            return;
        }
    }

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

// Spaces and tabs around fields, before and after closing quotes, at every offset around a block
// boundary, and with a tab as the delimiter.
trimmed_fields :: () {
    push_allocator(temp);

    for pad: 0..140 {
        run := repeat(#char " ", pad);

        for text: string.[
            tprint("%a%,\t%b\t,%\"c d\"%,%\n  \n%,%\r\n", run, run, run, run, run, run, run),
            tprint("\"x\"%\t,%y\"z\"\n", run, run),
            tprint("%\"q\"%x\n", run, run),
            tprint("a%\t%\tb\t%\n", run, run, run),
            tprint("%\"\"\"\" %,%", run, run, run),
        ] {
            expect_same_everywhere(text, .{ trim = true });
            expect_same_everywhere(text, .{ trim = true, lax_quotes = true });
            expect_same_everywhere(text, .{ trim = true, delimiter = 9 });
            expect_same_everywhere(text, .{ trim = true, max_field_bytes = pad / 2 + 1 });
        }
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

        expect_equal(slice_outcome(text, .{}, true), want);
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

// The window grown for a long record goes back to 8192 bytes once the short records after it
// need a refill (pieces smaller than the records left, so the source is read again).
long_record_window_shrinks :: () {
    push_allocator(temp);

    text: String_Builder;

    for 0..99999 append(*text, "x");

    append(*text, "\n");
    for 0..49999 append(*text, "a,b\n");

    document := builder_to_string(*text);

    for size: int.[7, 4096, 8192] {
        feed := Feed.{ text = document, size = size };
        reader := csv_reader(Csv_Source.{ *feed, feed_fill });

        defer csv_reader_free(*reader);

        fields, present, error := csv_next(*reader);

        expect(present && error.kind == .NONE);
        expect_equal(fields[0].count, 100000);
        expect(reader.window.count > 8192);

        rows := 0;

        while true {
            fields, present, error = csv_next(*reader);
            if !present break;

            expect(error.kind == .NONE && fields.count == 2);
            rows += 1;
        }

        expect_equal(rows, 50000);
        expect_equal(reader.window.count, 8192);
    }
} @test

// A record far longer than the reader's initial 8192-byte window, unquoted and quoted, is read by
// the block scanner (`record_scanned`) from a chunk source of any piece size: the window grows to
// hold it. The fields are those of a slice, and a source that fails inside the record reports the
// same outcome as byte-by-byte reading.
long_records_grow_the_window :: () {
    push_allocator(temp);

    wide: String_Builder;

    for 0..4999 {
        if it append(*wide, ",");

        print_to_builder(*wide, "f%", it);
    }

    append(*wide, "\nnext,row\n");

    quoted: String_Builder;

    append(*quoted, "head,\"");
    for 0..799 append(*quoted, "line of text\r\n with \"\"quotes\"\" and, commas ");

    append(*quoted, "\",\"tail\"\"\"\r\nnext,row\n");

    for text: string.[builder_to_string(*wide), builder_to_string(*quoted)] {
        expect(text.count > 3 * 8192);

        want := slice_outcome(text, .{});

        for size: int.[1, 7, 64, 4096, 8192, 8193, 100000] {
            feed := Feed.{ text = text, size = size };
            reader := csv_reader(Csv_Source.{ *feed, feed_fill });

            defer csv_reader_free(*reader);

            fields, present, error := csv_next(*reader);

            expect(present && error.kind == .NONE);
            expect(reader.record_scanned);
            expect_equal(fields.count, ifx text[0] == #char "h" then 3 else 5000);

            feed2 := Feed.{ text = text, size = size };
            second := csv_reader(Csv_Source.{ *feed2, feed_fill });

            defer csv_reader_free(*second);

            expect_equal(outcome(*second), want);
        }

        for fail_after: int.[100, 8191, 8192, 8193, 9000, 20000, text.count - 12] {
            for size: int.[7, 4096] {
                want_failure := failing_outcome(text, .{}, size, fail_after, true);
                got := failing_outcome(text, .{}, size, fail_after, false);

                if got != want_failure {
                    print("failing after % in pieces of %\n--- bytewise\n%\n--- blocks\n%\n", fail_after, size, want_failure, got);
                    expect(false);
                    return;
                }
            }
        }
    }
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

Quad :: struct {
    a: int;
    b: string;
    c: int;
    d: string;
}

// The rows read before a typed error and where the error is, as one string, from a slice (`size`
// 0) or from a chunk source.
typed_outcome :: (text: string, options: Csv_Options, size: int, bytewise: bool) -> string {
    feed := Feed.{ text = text, size = size };
    reader: Csv_Reader(Quad);

    if size > 0 reader = csv_reader(Quad, Csv_Source.{ *feed, feed_fill }, options);
    else reader = csv_reader(Quad, cast([] u8) text, options);

    defer csv_reader_free(*reader);
    reader.bytewise = bytewise;
    rows := 0;

    while true {
        row, ok, error := csv_next(*reader);

        if !ok return tprint("% rows, % at % line % column % (%)", rows, error.kind, error.offset, error.line, error.column, error.column_name);

        csv_row_free(*row);
        rows += 1;
    }

    return "";
}

// A typed error is placed by the same offset, line and column however the record was read: from a
// slice, from chunks of every size, or byte by byte.
expect_typed_error :: (text: string, options: Csv_Options, want: string) {
    got := typed_outcome(text, options, 0, false);

    if got != want {
        print("typed error in % is [%], want [%]\n", text, got, want);
        expect(false);
        return;
    }

    for CHUNK_SIZES {
        got = typed_outcome(text, options, it, false);

        if got != want {
            print("chunks of % give [%] on %, want [%]\n", it, got, text, want);
            expect(false);
            return;
        }
    }

    for size: int.[0, 1, 64, 8192] {
        got = typed_outcome(text, options, size, true);

        if got != want {
            print("bytewise reading in pieces of % gives [%] on %, want [%]\n", size, got, text, want);
            expect(false);
            return;
        }
    }
}

// The failing field comes after quoted fields holding line breaks of every kind and doubled quotes,
// in the first record and after earlier records.
typed_error_positions :: () {
    push_allocator(temp);

    // After one quoted CRLF.
    expect_typed_error("1,\"q\r\nr\",x\n", .{}, "0 rows, TYPE at 9 line 2 column 4 (c)");

    // The second record, after a quoted LF in the first and a quoted CRLF in itself.
    expect_typed_error("1,\"a\nb\",2,z\n3,\"c\r\nd\",x,z\n", .{}, "1 rows, TYPE at 21 line 4 column 4 (c)");

    // Quoted CRs alone are line breaks.
    expect_typed_error("1,\"a\rb\rc\",x", .{}, "0 rows, TYPE at 10 line 3 column 4 (c)");

    // Doubled quotes and a quoted LF, then the failing field.
    expect_typed_error("1,\"a\"\"\nb\"\"\",x\n", .{}, "0 rows, TYPE at 12 line 2 column 6 (c)");

    // The failing field is the first one, in a later record.
    expect_typed_error("1,\"a\nb\",2,z\nx,b,3,z\n", .{}, "1 rows, TYPE at 12 line 3 column 1 (a)");

    // Trimmed fields: the failing field starts before its leading spaces.
    expect_typed_error("1 ,\"q\r\nr\" ,  x ,z\n", .{ trim = true }, "0 rows, TYPE at 11 line 2 column 5 (c)");
    expect_typed_error("1,\"a\"\"\r\n\" ,  y\n", .{ trim = true }, "0 rows, TYPE at 11 line 2 column 4 (c)");

    // A record with too few fields is placed at its start.
    expect_typed_error("1,a,2,z\n5,\"q\r\n\"\n", .{}, "1 rows, HEADER at 8 line 2 column 1 (c)");
} @test

// A record shorter or longer than the reader's first 8192-byte window is read by the block scanner;
// the error is placed the same way as when it is read byte by byte.
typed_error_positions_in_long_records :: () {
    push_allocator(temp);

    for pad: int.[0, 1, 63, 64, 65, 4000, 4090, 4100, 9000] {
        run := repeat(#char "k", pad);

        for breaks: string.["\n", "\r\n", "\r"] {
            text := tprint("1,\"%%%\"\"%\",x,z\n7,s,8,t\n", run, breaks, run, breaks, run);

            for options: Csv_Options.[.{}, .{ trim = true }] {
                expect_typed_error(text, options, typed_outcome(text, options, 0, true));
            }
        }
    }
} @test

// Random documents with a bad number in a random record: every way of reading them places the
// error at the same byte.
random_typed_errors :: () {
    push_allocator(temp);

    rng: Rng;

    rng.state = 424242;

    parts := string.[",", "\"", "\r", "\n", "\r\n", " ", "a", "\"\"", "\t"];

    for 0..299 {
        records: [..][] string;
        count := 1 + rng_below(*rng, 5);
        bad := rng_below(*rng, count);

        for row: 0..count - 1 {
            fields: [..] string;

            array_add(*fields, ifx rng_below(*rng, 6) == 0 then random_text(*rng, parts, 3) else "5");
            array_add(*fields, random_text(*rng, parts, rng_below(*rng, 6)));
            array_add(*fields, ifx row == bad then "no" else "6");
            array_add(*fields, random_text(*rng, parts, rng_below(*rng, 6)));
            array_add(*records, fields);
        }

        ending := ifx rng_below(*rng, 3) == 0 then "\r\n" else ifx rng_below(*rng, 2) == 0 then "\n" else "\r";
        text := serialize(records, ending, true, rng_below(*rng, 2) == 0);

        for options: Csv_Options.[.{}, .{ trim = true }] {
            expect_typed_error(text, options, typed_outcome(text, options, 0, true));
        }
    }
} @test
