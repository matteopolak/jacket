#import,file "../../test-support/counting-allocator.jai";
#import "Basic";
#import "String";
#import "Extensions/Csv";
#import "Extensions/Testing";

Chunks :: struct {
    text: string;
    at: int;
    split: int;
    size: int = 1;
    bad: int;
}

fill :: (data: *void, buffer: [] u8) -> s64, string {
    c := cast(*Chunks) data;
    if c.bad == 1 return -1, "";
    if c.bad == 2 return buffer.count + 1, "";
    if c.bad == 3 return 0, "source failed";

    count := min(buffer.count, min(c.size, c.text.count - c.at));
    if c.at < c.split count = min(count, c.split - c.at);
    if count > 0 memcpy(buffer.data, c.text.data + c.at, count);

    c.at += count;

    return count, "";
}

chunk_boundaries :: () {
    text := "\xef\xbb\xbfa,b\r\n\"a\"\"b\",\"x\r\ny\"\rnext,last";
    for split: 0..text.count {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            chunks := Chunks.{ text = text, split = split, size = text.count };
            reader := csv_reader(Csv_Source.{ *chunks, fill });

            defer csv_reader_free(*reader);

            for row_index: 0..2 {
                row, ok, error := csv_next(*reader);

                require(ok && error.kind == .NONE);
                expect_equal(row.count, 2);

                if row_index == 0 {
                    expect_equal(row[0], "a");
                    expect_equal(row[1], "b");
                }

                if row_index == 1 {
                    expect_equal(row[0], "a\"b");
                    expect_equal(row[1], "x\r\ny");
                }

                if row_index == 2 {
                    expect_equal(row[0], "next");
                    expect_equal(row[1], "last");
                }
            }

            _, ok, e := csv_next(*reader);

            expect(!ok && e.kind == .NONE);
        }

        expect_equal(tally.outstanding, 0);
    }
} @test

source_failures :: () {
    for bad: 1..3 {
        chunks := Chunks.{ bad = bad };
        reader := csv_reader(Csv_Source.{ *chunks, fill });

        defer csv_reader_free(*reader);
        _, ok, error := csv_next(*reader);

        expect(!ok && error.kind == .SOURCE);
    }

    missing := csv_reader(Csv_Source.{});

    defer csv_reader_free(*missing);
    _, ok, error := csv_next(*missing);

    expect(!ok && error.kind == .OPTIONS);
} @test

one_byte_and_large :: () {
    for text: string.["\xef", "\xef\xbb", "\xef\xbb\xbf", "a,b\r\n\"x\"\"y\",z\r"] {
        chunks := Chunks.{ text = text };
        reader := csv_reader(Csv_Source.{ *chunks, fill });

        defer csv_reader_free(*reader);
        slice_reader := csv_reader(cast([] u8) text);

        defer csv_reader_free(*slice_reader);

        while true {
            row, ok, e := csv_next(*reader);
            other, valid, error := csv_next(*slice_reader);

            expect_equal(ok, valid);
            expect_equal(e.kind, error.kind);
            if !ok break;

            expect_equal(row.count, other.count);
            for row expect_equal(it, other[it_index]);
        }
    }

    text := alloc_string(20000);

    defer free(text);
    for *text it.* = #char "x";

    chunks := Chunks.{ text = text, size = 997 };
    reader := csv_reader(Csv_Source.{ *chunks, fill });

    defer csv_reader_free(*reader);
    row, ok, error := csv_next(*reader);

    require(ok && error.kind == .NONE);
    expect_equal(row[0], text);
} @test

// The error a reader ends on, or NONE at a clean end.
final_error :: (reader: *Csv_Reader) -> Csv_Error {
    while true {
        _, ok, error := csv_next(reader);
        if !ok return error;
    }

    return .{};
}

// Slice input and chunked input at several sizes must stop on the same error in the same place.
expect_error_at :: (text: string, options: Csv_Options, kind: Csv_Error_Kind, offset: int, line: int, column: int) {
    slice_reader := csv_reader(cast([] u8) text, options);

    defer csv_reader_free(*slice_reader);
    errors := Csv_Error.[final_error(*slice_reader), .{}, .{}, .{}, .{}, .{}, .{}, .{}];

    for size: int.[1, 5, 63, 64, 65, 8192, 100000] {
        chunks := Chunks.{ text = text, size = size };
        reader := csv_reader(Csv_Source.{ *chunks, fill }, options);

        defer csv_reader_free(*reader);
        errors[1 + it_index] = final_error(*reader);
    }

    for errors {
        expect_equal(it.kind, kind, tprint("%: kind", it_index));
        expect_equal(it.offset, offset, tprint("%: offset", it_index));
        expect_equal(it.line, line, tprint("%: line", it_index));
        expect_equal(it.column, column, tprint("%: column", it_index));
    }
}

limits_and_positions_mid_run :: () {
    // The field limit trips on the byte that does not fit, inside a run of plain bytes.
    expect_error_at("ab,abcdefgh\n", .{ max_field_bytes = 5 }, .FIELD_LIMIT, 9, 1, 10);
    expect_error_at("x\n\"abcdefghij\"\n", .{ max_field_bytes = 4 }, .FIELD_LIMIT, 8, 2, 7);

    // The record limit trips before the byte past it.
    expect_error_at("ab,abcdefgh\n", .{ max_record_bytes = 6 }, .RECORD_LIMIT, 6, 1, 7);
    expect_error_at("x\n\"abcdefghij\"\n", .{ max_record_bytes = 5 }, .RECORD_LIMIT, 7, 2, 6);

    // The limit can fall on a delimiter or a line ending.
    expect_error_at("a,b", .{ max_record_bytes = 1 }, .RECORD_LIMIT, 1, 1, 2);
    expect_error_at("ab\r\n", .{ max_record_bytes = 2 }, .RECORD_LIMIT, 2, 1, 3);

    // Where both limits are reached at once, the record limit comes first.
    expect_error_at("abcdef", .{ max_record_bytes = 3, max_field_bytes = 3 }, .RECORD_LIMIT, 3, 1, 4);

    // Line and column after a long run, a quoted line break, and CRLF pairs.
    expect_error_at("\"l1\nl2\",a\nbbbbbbbbbbbb\"c\n", .{}, .QUOTE, 22, 3, 13);
    expect_error_at("a\r\n\"b\r\nc\"\r\nzzzz\"", .{}, .QUOTE, 15, 4, 5);
    expect_error_at("a\r\"b\"x", .{}, .QUOTE, 5, 2, 4);

    // Trimmed leading space still counts toward the record limit.
    expect_error_at("   abc", .{ trim = true, max_record_bytes = 4 }, .RECORD_LIMIT, 4, 1, 5);

    // A run of 20000 bytes crosses chunk after chunk before the limit.
    long := alloc_string(20000);

    defer free(long);
    for *long it.* = #char "y";

    expect_error_at(long, .{ max_field_bytes = 15000 }, .FIELD_LIMIT, 15001, 1, 15002);
    expect_error_at(long, .{ max_record_bytes = 9000 }, .RECORD_LIMIT, 9000, 1, 9001);
} @test

// A mixed document read from one slice and from chunks of many sizes yields the same records.
chunked_documents_match :: () {
    builder: String_Builder;

    state: u32 = 12345;
    pieces := string.["plain", "with space", "x\"y", "a,b", "line\nbreak", "cr\rbreak", "crlf\r\nbreak", "", " pad ", "\t"];

    for row: 0..299 {
        for field: 0..3 {
            if field append(*builder, #char ",");

            state = state * 1664525 + 1013904223;
            pick := cast(int) ((state >> 16) % 10);
            repeat := cast(int) ((state >> 8) % 40);
            text := pieces[pick];
            special := pick >= 2 && pick <= 6;
            quoted := special || (state & 1) != 0;

            if quoted append(*builder, #char "\"");

            for 1..repeat {
                if !quoted {
                    if !special append(*builder, text);
                    continue;
                }

                for c: text {
                    append(*builder, c);
                    if c == #char "\"" append(*builder, c);
                }
            }

            if quoted append(*builder, #char "\"");
        }

        append(*builder, ifx row % 3 == 0 then "\r\n" else "\n");
    }

    text := builder_to_string(*builder);

    defer free(text);

    for option: Csv_Options.[.{}, .{ trim = true }, .{ lax_quotes = true, trim = true }] {
        slice_reader := csv_reader(cast([] u8) text, option);

        defer csv_reader_free(*slice_reader);
        rows: [..][] string;
        ending: Csv_Error;

        defer {
            for rows csv_record_free(it);
            array_free(rows);
        }

        while true {
            row, ok, error := csv_next(*slice_reader);
            if !ok {
                ending = error;
                break;
            }

            array_add(*rows, csv_record_copy(row));
        }

        for size: int.[1, 13, 63, 64, 65, 997, 8192, 100000] {
            chunks := Chunks.{ text = text, size = size };
            reader := csv_reader(Csv_Source.{ *chunks, fill }, option);

            defer csv_reader_free(*reader);
            index := 0;

            while true {
                row, ok, error := csv_next(*reader);
                if !ok {
                    expect_equal(error.kind, ending.kind);
                    expect_equal(error.offset, ending.offset);
                    expect_equal(error.line, ending.line);
                    expect_equal(error.column, ending.column);
                    break;
                }

                require(index < rows.count);
                require(row.count == rows[index].count);

                for field: row expect_equal(field, rows[index][it_index], tprint("size % row % field %", size, index, it_index));

                index += 1;
            }

            expect_equal(index, rows.count);
        }
    }
} @test
