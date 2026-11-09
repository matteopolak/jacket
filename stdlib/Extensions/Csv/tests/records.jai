#import,file "../../test-support/counting-allocator.jai";
#import "Basic";
#import "String";
#import "Extensions/Csv";
#import "Extensions/Testing";

records :: () {
    reader := csv_reader(cast([] u8) "\xef\xbb\xbfa,b\r\n\"x,y\",\"a\"\"b\"\r\n\"one\rtwo\nthree\",z\r");

    defer csv_reader_free(*reader);
    expected := string.["a", "b", "x,y", "a\"b", "one\rtwo\nthree", "z"];

    for 0..2 {
        row, ok, error := csv_next(*reader);

        require(ok && error.kind == .NONE);
        expect_equal(row.count, 2);
        expect_equal(row[0], expected[it * 2]);
        expect_equal(row[1], expected[it * 2 + 1]);
    }

    _, ok, error := csv_next(*reader);

    expect(!ok && error.kind == .NONE);
} @test

roundtrip :: () {
    fields := string.["", "plain", " leading", "trailing ", "a,b", "a\"b", "\r\n", "é"];

    for trim: bool.[false, true] {
        builder: String_Builder;

        defer free_buffers(*builder);
        error := csv_write_record(*builder, fields, .{ trim = trim });

        require(error.kind == .NONE);
        text := builder_to_string(*builder);

        defer free(text);
        reader := csv_reader(cast([] u8) text, .{ trim = trim });

        defer csv_reader_free(*reader);
        row, ok, e := csv_next(*reader);

        require(ok && e.kind == .NONE);
        expect_equal(row.count, fields.count);
        for row expect_equal(it, fields[it_index]);

        copy := csv_record_copy(row);

        defer csv_record_free(copy);
        csv_next(*reader);
        for copy expect_equal(it, fields[it_index]);
    }
} @test

quote_runs :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    error := csv_write_record(*builder, string.["\"", "a\"\"b", "\"x", "x\"", "p,q\"r\"s", "plain"], .{ line_ending = .LF });

    require(error.kind == .NONE);
    text := builder_to_string(*builder);

    defer free(text);
    expect_equal(text, "\"\"\"\",\"a\"\"\"\"b\",\"\"\"x\",\"x\"\"\",\"p,q\"\"r\"\"s\",plain\n");
} @test

reject :: (text: string, kind: Csv_Error_Kind, options := Csv_Options.{}) {
    reader := csv_reader(cast([] u8) text, options);

    defer csv_reader_free(*reader);

    while true {
        _, ok, error := csv_next(*reader);
        if !ok {
            expect_equal(error.kind, kind);
            break;
        }
    }
}

errors :: () {
    reject("a\"b", .QUOTE);
    reject("\"a", .QUOTE);
    reject("\"a\"x", .QUOTE);
    reject("abc", .FIELD_LIMIT, .{ max_field_bytes = 2 });
    reject("a,b", .FIELD_COUNT, .{ max_fields = 1 });
    reject("a,b", .RECORD_LIMIT, .{ max_record_bytes = 2 });
    reject("a,b\nx", .WIDTH, .{ require_equal_width = true });
    reject("", .OPTIONS, .{ delimiter = #char "\"" });
    reject("", .OPTIONS, .{ max_fields = 0 });
    reader := csv_reader(cast([] u8) "a\r\nb\"c");

    defer csv_reader_free(*reader);
    csv_next(*reader);
    _, ok, error := csv_next(*reader);

    expect(!ok && error.kind == .QUOTE);
    expect_equal(error.offset, 4);
    expect_equal(error.line, 2);
    expect_equal(error.column, 2);
} @test

dialects :: () {
    reader := csv_reader(cast([] u8) " a ;'b;b'; c'd\n", .{ delimiter = #char ";", quote = #char "'", trim = true, lax_quotes = true });

    defer csv_reader_free(*reader);
    row, ok, error := csv_next(*reader);

    require(ok && error.kind == .NONE);
    expect_equal(row[0], "a");
    expect_equal(row[1], "b;b");
    expect_equal(row[2], "c'd");
    empty := csv_reader(cast([] u8) "\n\n");

    defer csv_reader_free(*empty);

    for 0..1 {
        r, valid := csv_next(*empty);
        require(valid);
        expect_equal(r.count, 1);
        expect_equal(r[0], "");
    }

    _, valid := csv_next(*empty);

    expect(!valid);
} @test

bounded_quote_suffix :: () {
    reject("\"x\"  ", .RECORD_LIMIT, .{ trim = true, max_record_bytes = 3 });
} @test @timeout(2)

writer_limits :: () {
    cases := Csv_Options.[.{ max_field_bytes = 1 }, .{ max_fields = 1 }, .{ max_record_bytes = 2 }, .{ quote = 10 }];
    kinds := Csv_Error_Kind.[.FIELD_LIMIT, .FIELD_COUNT, .RECORD_LIMIT, .OPTIONS];

    for options: cases {
        builder: String_Builder;

        defer free_buffers(*builder);
        append(*builder, "prefix");
        error := csv_write_record(*builder, string.["ab", "cd"], options);

        expect_equal(error.kind, kinds[it_index]);
        text := builder_to_string(*builder);

        defer free(text);
        expect_equal(text, "prefix");
    }

    for ending: Csv_Line_Ending.[.LF, .CR, .CRLF] {
        builder: String_Builder;

        defer free_buffers(*builder);
        e := csv_write_record(*builder, string.[""], .{ line_ending = ending });

        require(e.kind == .NONE);
        text := builder_to_string(*builder);

        defer free(text);
        expect_equal(text, ifx ending == .LF then "\n" else ifx ending == .CR then "\r" else "\r\n");
    }
} @test

lax_and_bounds :: () {
    reader := csv_reader(cast([] u8) "a\"b,x\r\n", .{ lax_quotes = true, max_field_bytes = 3, max_fields = 2, max_record_bytes = 8 });

    defer csv_reader_free(*reader);
    row, ok, error := csv_next(*reader);

    require(ok && error.kind == .NONE);
    expect_equal(row[0], "a\"b");
    reject("a\"b,x\r\n", .RECORD_LIMIT, .{ lax_quotes = true, max_record_bytes = 6 });
    reject("\"abc\"", .FIELD_LIMIT, .{ max_field_bytes = 2 });
} @test

space_delimiter :: () {
    reader := csv_reader(cast([] u8) "\"a\" \"b\"\n", .{ delimiter = 32, trim = true });

    defer csv_reader_free(*reader);
    row, ok, error := csv_next(*reader);

    require(ok && error.kind == .NONE);
    expect_equal(row.count, 2);
    expect_equal(row[1], "b");
} @test

malformed_ownership :: () {
    text := "a,b\r\n\"one\"\"two\nthree\",end\n";
    for length: 0..text.count {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            reader := csv_reader(cast([] u8) slice(text, 0, length));

            defer csv_reader_free(*reader);

            while true {
                fields, ok := csv_next(*reader);
                if !ok break;
                owned := csv_record_copy(fields);
                csv_record_free(owned);
            }
        }

        expect_equal(tally.outstanding, 0);
    }
} @test

duplicate_headers :: () {
    reject("a,a\n1,2", .HEADER, .{ header = true });
} @test
