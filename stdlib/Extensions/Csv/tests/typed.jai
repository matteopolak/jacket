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
        reader := csv_reader(cast([] u8) "age,full-name,score,active,nick\n42,Ada,1.25,true,Al\n7,Bob,2,false,\n", .{ header = true });

        defer csv_reader_free(*reader);
        a, ok, e := csv_next_row(Person, *reader);

        require(ok && e.kind == .NONE);
        defer csv_row_free(*a);
        expect_equal(a.name, "Ada");
        expect_equal(a.age, 42);
        expect_equal(a.score, 1.25);
        require(a.nick != null);
        expect_equal(a.nick.*, "Al");
        b, valid := csv_next_row(Person, *reader);

        require(valid);
        defer csv_row_free(*b);
        expect(b.nick == null);
        builder: String_Builder;

        defer free_buffers(*builder);
        error := csv_write_rows(*builder, Person.[a, b]);

        require(error.kind == .NONE);
        text := builder_to_string(*builder);

        defer free(text);
        again := csv_reader(cast([] u8) text, .{ header = true });

        defer csv_reader_free(*again);
        row, present, err := csv_next_row(Person, *again);

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
            reader := csv_reader(cast([] u8) text);

            defer csv_reader_free(*reader);
            row, ok, e := csv_next_row(Person, *reader);

            expect(!ok && e.kind == .TYPE);
            expect(e.column_name.count > 0);
            csv_row_free(*row);
        }

        expect_equal(tally.outstanding, 0);
    }

    reader := csv_reader(cast([] u8) "age,age\n1,2", .{ header = true });

    defer csv_reader_free(*reader);
    _, ok, error := csv_next_row(Person, *reader);

    expect(!ok && error.kind == .HEADER);
} @test

Defaults :: struct {
    name := "guest";
    age: int;
}

default_ownership :: () {
    reader := csv_reader(cast([] u8) "wrong,age\nx,no", .{ header = true });

    defer csv_reader_free(*reader);
    row, ok, error := csv_next_row(Defaults, *reader);

    expect(!ok && error.kind == .HEADER);
    expect_equal(row.name, "guest");
    csv_row_free(*row);
} @test

Numbers :: struct {
    unsigned: u64;
    real: float64;
}

numbers :: () {
    reader := csv_reader(cast([] u8) "18446744073709551615,1.2345678901234567");

    defer csv_reader_free(*reader);
    row, ok, e := csv_next_row(Numbers, *reader);

    require(ok && e.kind == .NONE);
    expect_equal(row.unsigned, cast(u64) 0xffffffffffffffff);
    expect_equal(row.real, 1.2345678901234567);
    builder: String_Builder;

    defer free_buffers(*builder);
    error := csv_write_rows(*builder, Numbers.[row]);

    require(error.kind == .NONE);
    text := builder_to_string(*builder);

    defer free(text);
    again := csv_reader(cast([] u8) text, .{ header = true });

    defer csv_reader_free(*again);
    decoded, valid, err := csv_next_row(Numbers, *again);

    require(valid && err.kind == .NONE);
    expect_equal(decoded.real, row.real);
} @test

typed_location :: () {
    reader := csv_reader(cast([] u8) "1,2\r\n4,no");

    defer csv_reader_free(*reader);
    csv_next_row(Numbers, *reader);
    _, ok, error := csv_next_row(Numbers, *reader);

    expect(!ok && error.kind == .TYPE);
    expect_equal(error.line, 2);
    expect_equal(error.column, 3);
    expect_equal(error.offset, 7);
} @test

Small :: struct {
    number: float32;
}

small_rounding :: () {
    reader := csv_reader(cast([] u8) "1.0000000596046447753906250000000000000000000000000001");

    defer csv_reader_free(*reader);
    value, ok, error := csv_next_row(Small, *reader);

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
            reader := csv_reader(cast([] u8) input);

            defer csv_reader_free(*reader);
            row, ok, error := csv_next_row(Optional_Number, *reader);

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
    reader := csv_reader(cast([] u8) text, .{ header = true });

    defer csv_reader_free(*reader);
    row, ok, e := csv_next_row(Named, *reader);

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

Right :: struct {
    a: int;
    c: int;
}

// A reader remembers which header column each member read, but one reader may decode different structs.
reader_shared_by_types :: () {
    reader := csv_reader(cast([] u8) "a,b,c\n1,2,3\n4,5,6\n7,8,9\n", .{ header = true });

    defer csv_reader_free(*reader);
    first, ok, e := csv_next_row(Left, *reader);

    require(ok && e.kind == .NONE);
    expect_equal(first.b, 2);
    expect_equal(first.a, 1);
    second, valid, e2 := csv_next_row(Right, *reader);

    require(valid && e2.kind == .NONE);
    expect_equal(second.a, 4);
    expect_equal(second.c, 6);
    third, again, e3 := csv_next_row(Left, *reader);

    require(again && e3.kind == .NONE);
    expect_equal(third.b, 8);
    expect_equal(third.a, 7);
} @test
