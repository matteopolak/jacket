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
