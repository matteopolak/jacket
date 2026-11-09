#import,file "../../test-support/counting-allocator.jai";
#import "Basic";
#import "Extensions/Csv";
#import "Extensions/Testing";

Person :: struct {
    name: string; @csv(name=full-name)
    age: s8;
    score: float64;
    active: bool;
    nick: *string;
    ignored: int; @csv(skip)
}

typed_rows :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);
        reader := csv_reader(Person, cast([] u8) "age,full-name,score,active,nick\n42,Ada,1.25,true,Al\n7,Bob,2,false,\n", .{ header = true });

        defer csv_reader_free(*reader);
        a, ok, e := csv_next(*reader);

        require(ok && e.kind == .NONE);
        defer csv_row_free(*a);
        expect_equal(a.name, "Ada");
        expect_equal(a.age, 42);
        expect_equal(a.score, 1.25);
        require(a.nick != null);
        expect_equal(a.nick.*, "Al");
        b, valid := csv_next(*reader);

        require(valid);
        defer csv_row_free(*b);
        expect(b.nick == null);
        builder: String_Builder;

        defer free_buffers(*builder);
        error := csv_write_rows(*builder, Person.[a, b]);

        require(error.kind == .NONE);
        text := builder_to_string(*builder);

        defer free(text);
        again := csv_reader(Person, cast([] u8) text, .{ header = true });

        defer csv_reader_free(*again);
        row, present, err := csv_next(*again);

        require(present && err.kind == .NONE);
        expect_equal(row.name, "Ada");
        csv_row_free(*row);
    }

    expect_equal(tally.outstanding, 0);
} @test

typed_errors :: () {
    for text: string.["Ada,,1,true,", "Ada,128,1,true,", "Ada,1,wat,true,", "Ada,1,1,yes,"] {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            reader := csv_reader(Person, cast([] u8) text);

            defer csv_reader_free(*reader);
            row, ok, e := csv_next(*reader);

            expect(!ok && e.kind == .TYPE);
            expect(e.column_name.count > 0);
            csv_row_free(*row);
        }

        expect_equal(tally.outstanding, 0);
    }

    reader := csv_reader(Person, cast([] u8) "age,age\n1,2", .{ header = true });

    defer csv_reader_free(*reader);
    _, ok, error := csv_next(*reader);

    expect(!ok && error.kind == .HEADER);
} @test

Defaults :: struct {
    name := "guest";
    age: int;
}

default_ownership :: () {
    reader := csv_reader(Defaults, cast([] u8) "wrong,age\nx,no", .{ header = true });

    defer csv_reader_free(*reader);
    row, ok, error := csv_next(*reader);

    expect(!ok && error.kind == .HEADER);
    expect_equal(row.name, "guest");
    csv_row_free(*row);
} @test

Numbers :: struct {
    unsigned: u64;
    real: float64;
}

numbers :: () {
    reader := csv_reader(Numbers, cast([] u8) "18446744073709551615,1.2345678901234567");

    defer csv_reader_free(*reader);
    row, ok, e := csv_next(*reader);

    require(ok && e.kind == .NONE);
    expect_equal(row.unsigned, cast(u64) 0xffffffffffffffff);
    expect_equal(row.real, 1.2345678901234567);
    builder: String_Builder;

    defer free_buffers(*builder);
    error := csv_write_rows(*builder, Numbers.[row]);

    require(error.kind == .NONE);
    text := builder_to_string(*builder);

    defer free(text);
    again := csv_reader(Numbers, cast([] u8) text, .{ header = true });

    defer csv_reader_free(*again);
    decoded, valid, err := csv_next(*again);

    require(valid && err.kind == .NONE);
    expect_equal(decoded.real, row.real);
} @test

typed_location :: () {
    reader := csv_reader(Numbers, cast([] u8) "1,2\r\n4,no");

    defer csv_reader_free(*reader);
    csv_next(*reader);
    _, ok, error := csv_next(*reader);

    expect(!ok && error.kind == .TYPE);
    expect_equal(error.line, 2);
    expect_equal(error.column, 3);
    expect_equal(error.offset, 7);
} @test

Small :: struct {
    number: float32;
}

small_rounding :: () {
    reader := csv_reader(Small, cast([] u8) "1.0000000596046447753906250000000000000000000000000001");

    defer csv_reader_free(*reader);
    value, ok, error := csv_next(*reader);

    require(ok && error.kind == .NONE);
    expect_equal(value.number, cast(float32) 1.00000011920928955078125);
} @test

Optional_Number :: struct {
    text: string;
    number: *s32;
}

optional_number_ownership :: () {
    for input: string.["a,123", "a,", "a,bad"] {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            reader := csv_reader(Optional_Number, cast([] u8) input);

            defer csv_reader_free(*reader);
            row, ok, error := csv_next(*reader);

            expect_equal(ok, input != "a,bad");
            expect_equal(error.kind, ifx input == "a,bad" then Csv_Error_Kind.TYPE else .NONE);
            csv_row_free(*row);
            csv_row_free(*row);
        }

        expect_equal(tally.outstanding, 0);
    }
} @test

Named :: struct {
    value: string; @csv(name="first,last")
    ignored: int; @csv(skip)
}

quoted_note :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    error := csv_write_rows(*builder, Named.[.{ "Ada", 7 }]);

    require(error.kind == .NONE);
    text := builder_to_string(*builder);

    defer free(text);
    reader := csv_reader(Named, cast([] u8) text, .{ header = true });

    defer csv_reader_free(*reader);
    row, ok, e := csv_next(*reader);

    require(ok && e.kind == .NONE);
    expect_equal(row.value, "Ada");
    expect_equal(row.ignored, 0);
    csv_row_free(*row);
} @test

Escaped :: struct {
    value: string; @csv(name="say \"hi\"\tthen")
}

escaped_note_name :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    error := csv_write_rows(*builder, Escaped.[.{ "x" }]);

    require(error.kind == .NONE);
    text := builder_to_string(*builder);

    defer free(text);
    expect_equal(text, "\"say \"\"hi\"\"\tthen\"\r\nx\r\n");
} @test

Left :: struct {
    b: int;
    a: int;
}

// The reader maps columns once; every row then reads the same columns, whatever the header order.
columns_fixed_per_reader :: () {
    reader := csv_reader(Left, cast([] u8) "a,b,c\n1,2,3\n4,5,6\n7,8,9\n", .{ header = true });

    defer csv_reader_free(*reader);
    first, ok, e := csv_next(*reader);

    require(ok && e.kind == .NONE);
    expect_equal(first.b, 2);
    expect_equal(first.a, 1);
    second, valid, e2 := csv_next(*reader);

    require(valid && e2.kind == .NONE);
    expect_equal(second.b, 5);
    expect_equal(second.a, 4);
    third, again, e3 := csv_next(*reader);

    require(again && e3.kind == .NONE);
    expect_equal(third.b, 8);
    expect_equal(third.a, 7);
    _, more, e4 := csv_next(*reader);

    expect(!more && e4.kind == .NONE);
} @test

// A header that lacks a member is an error on the first row, not on a header-only document.
missing_column :: () {
    only_header := csv_reader(Left, cast([] u8) "a,c\n", .{ header = true });

    defer csv_reader_free(*only_header);
    _, present, error := csv_next(*only_header);

    expect(!present && error.kind == .NONE);

    reader := csv_reader(Left, cast([] u8) "a,c\n1,2\n", .{ header = true });

    defer csv_reader_free(*reader);
    _, ok, e := csv_next(*reader);

    expect(!ok && e.kind == .HEADER);
    expect_equal(e.column_name, "b");
} @test

// `for row: reader` yields decoded rows and stops at the end or on an error, which stays in `error`.
iterate_typed :: () {
    reader := csv_reader(Person, cast([] u8) "full-name,age,score,active,nick\nAda,36,1.5,true,\nBob,41,2,false,Rob\n", .{ header = true });

    defer csv_reader_free(*reader);
    count := 0;

    for row, index: reader {
        defer csv_row_free(*row);
        expect_equal(index, count);
        expect_equal(row.name, ifx index == 0 then "Ada" else "Bob");
        count += 1;
    }

    expect_equal(count, 2);
    expect_equal(reader.error.kind, Csv_Error_Kind.NONE);
} @test

iterate_stops_on_error :: () {
    reader := csv_reader(Numbers, cast([] u8) "1,2\n3,x\n5,6\n");

    defer csv_reader_free(*reader);
    count := 0;

    for row: reader count += 1;

    expect_equal(count, 1);
    expect_equal(reader.error.kind, Csv_Error_Kind.TYPE);
    expect_equal(reader.error.line, 2);
} @test

iterate_untyped :: () {
    reader := csv_reader(cast([] u8) "x,y\n1,2\n3,4\n5,6\n", .{ header = true });

    defer csv_reader_free(*reader);
    total := 0;

    for fields: reader {
        total += fields.count;
        if fields[0] == "3" break;
    }

    expect_equal(total, 4);
} @test

Borrowed :: struct {
    id: int;
    plain: string;
    quoted: string;
    doubled: string;
    nick: *string;
}

inside :: (text: string, input: string) -> bool {
    return text.data >= input.data && text.data + text.count <= input.data + input.count;
}

// With `borrow`, a string that needs no unquoting is a slice of the input and the rest is copied.
borrowed_strings :: () {
    input := "id,plain,quoted,doubled,nick\n7,abc,\"x, y\",\"say \"\"hi\"\"\",nick\n";
    reader := csv_reader(Borrowed, cast([] u8) input, .{ header = true, borrow = true });

    defer csv_reader_free(*reader);
    row, ok, error := csv_next(*reader);

    require(ok && error.kind == .NONE);
    expect_equal(row.plain, "abc");
    expect_equal(row.quoted, "x, y");
    expect_equal(row.doubled, "say \"hi\"");
    expect_equal(row.nick.*, "nick");
    expect(inside(row.plain, input));
    expect(inside(row.quoted, input));
    expect(inside(row.nick.*, input));
    expect(!inside(row.doubled, input));

    // Releasing through the reader frees only what the row owns: the pointer. The copied string is the caller's.
    free(row.doubled.data);
    csv_row_free(*reader, *row);
    expect(row.nick == null);
    expect_equal(row.plain, "");
} @test

// Neither a failed decode nor `csv_row_free(*reader, ...)` frees a borrowed slice.
borrowed_failure_and_free :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);
        reader := csv_reader(Borrowed, cast([] u8) "id,plain,quoted,doubled,nick\nx,abc,def,ghi,jkl\n", .{ header = true, borrow = true });

        defer csv_reader_free(*reader);
        _, ok, error := csv_next(*reader);

        expect(!ok && error.kind == .TYPE);
    }

    expect_equal(tally.outstanding, 0);

    tally = .{};

    {
        push_allocator(tally_proc, *tally);
        reader := csv_reader(Borrowed, cast([] u8) "id,plain,quoted,doubled,nick\n1,a,b,c,d\n", .{ header = true, borrow = true });

        defer csv_reader_free(*reader);
        row, ok, error := csv_next(*reader);

        require(ok && error.kind == .NONE);
        csv_row_free(*reader, *row);
    }

    expect_equal(tally.outstanding, 0);
} @test

// Without `borrow` strings are owned copies.
owned_strings :: () {
    input := "id,plain,quoted,doubled,nick\n7,abc,def,ghi,\n";
    reader := csv_reader(Borrowed, cast([] u8) input, .{ header = true });

    defer csv_reader_free(*reader);
    row, ok, error := csv_next(*reader);

    require(ok && error.kind == .NONE);
    defer csv_row_free(*row);
    expect(!inside(row.plain, input));
} @test

empty_fill :: (data: *void, buffer: [] u8) -> s64, string {
    return 0, "";
}

// A chunk's window is reused, so a borrowed string would dangle: the option is rejected.
borrow_needs_slice_input :: () {
    state: int;
    reader := csv_reader(Numbers, Csv_Source.{ *state, empty_fill }, .{ borrow = true });

    defer csv_reader_free(*reader);
    _, present, error := csv_next(*reader);

    expect(!present && error.kind == .OPTIONS);
} @test

// The reader's own storage (header, column map, buffers) comes from the allocator it was made with,
// whatever allocator a call to csv_next runs under; only the row follows the call.
reader_storage_ignores_call_allocator :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);
        reader := csv_reader(Borrowed, cast([] u8) "id,plain,quoted,doubled,nick\n1,a,b,c,d\n2,e,f,g,\n", .{ header = true, borrow = true });

        defer csv_reader_free(*reader);

        for 0..1 {
            row, ok, error := csv_next(*reader,, temp);

            require(ok && error.kind == .NONE);
            expect_equal(row.id, it + 1);
        }

        reset_temporary_storage();
    }

    expect_equal(tally.outstanding, 0);
} @test
