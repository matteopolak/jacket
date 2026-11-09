#import "Basic";
#import "String";
#import "File";
#import "Extensions/Environment";
#import "Extensions/Testing";
#import "Extensions/Csv";

Row :: struct {
    id: int;
    name: string;
    value: float64;
    active: bool;
    note: string;
}

// Hands a document to a reader in pieces of at most `size` bytes.
Pieces :: struct {
    text: string;
    at: int;
    size: int;
}

next_piece :: (data: *void, buffer: [] u8) -> s64, string {
    pieces := cast(*Pieces) data;
    count := min(buffer.count, min(pieces.size, pieces.text.count - pieces.at));

    if count > 0 memcpy(buffer.data, pieces.text.data + pieces.at, count);

    pieces.at += count;

    return count, "";
}

NAMES :: string.["Ada", "Grace", "Margaret", "Katherine", "Alan", "Edsger", "Barbara", "Donald", "Radia", "Linus", "Dennis", "Frances", "Tim", "Hedy", "Claude", "Shafi"];
NOTES :: string.["paid in full", "ships next week", "needs review", "returned by customer", "standard delivery", "gift wrapped", "back ordered", "approved"];

// Exported-looking data of a million rows: varied ids, names, numbers and notes, with a quoted name
// holding a comma in one row of twenty.
plain_input :: () -> string {
    builder: String_Builder;

    append(*builder, "id,name,value,active,note\n");

    for 0..999_999 {
        name := NAMES[(it * 7) % NAMES.count];
        note := NOTES[(it * 13) % NOTES.count];

        print_to_builder(*builder, "%,", 1_000_003 + it * 7);

        if it % 20 == 19 print_to_builder(*builder, "\"%, Jr.\",", name);
        else print_to_builder(*builder, "%,", name);

        print_to_builder(*builder, "%.%,%,% order %\n", (it * 31) % 9973, 1 + it % 9, ifx it % 3 == 0 then "true" else "false", note, it % 1000);
    }

    return builder_to_string(*builder);
}

// Reads every record of `data` (from a source that gives `size` bytes at a time when `size` is
// positive) and returns the number of fields.
count_fields :: (data: string, options: Csv_Options, size: int) -> int {
    pieces := Pieces.{ text = data, size = size };
    reader: Csv_Reader;

    if size > 0 reader = csv_reader(Csv_Source.{ *pieces, next_piece }, options);
    else reader = csv_reader(cast([] u8) black_box(data), options);

    defer csv_reader_free(*reader);
    count := 0;

    while true {
        row, ok, error := csv_next(*reader);

        assert(error.kind == .NONE);
        if !ok break;

        count += row.count;
        black_box(row);
    }

    return count;
}

benchmark_input :: () -> string {
    data, ok := read_entire_file(output_path("csv-conformance/bench.csv"));

    assert(ok, "run `PYTHONPATH=tools python3 tools/portable/build_portable.py bench` first");

    return data;
}

read_records :: (b: *Bench) {
    data := benchmark_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 black_box(count_fields(data, .{ header = true }, 0));
} @bench @samples(10)

read_records_plain :: (b: *Bench) {
    data := plain_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 black_box(count_fields(data, .{ header = true }, 0));
} @bench @samples(10)

read_records_plain_trim :: (b: *Bench) {
    data := plain_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 black_box(count_fields(data, .{ header = true, trim = true }, 0));
} @bench @samples(10)

read_typed :: (b: *Bench) {
    data := benchmark_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        reader := csv_reader(Row, cast([] u8) black_box(data), .{ header = true });
        count := 0;

        while true {
            row, ok, error := csv_next(*reader);

            assert(error.kind == .NONE);
            if !ok break;

            count += row.id;
            black_box(row);
            csv_row_free(*row);
        }

        black_box(count);
        csv_reader_free(*reader);
    }
} @bench @samples(10)

// Typed rows with `borrow`, decoded into temporary storage that is emptied after each row.
read_typed_borrowed :: (b: *Bench) {
    data := benchmark_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        reader := csv_reader(Row, cast([] u8) black_box(data), .{ header = true, borrow = true });
        count := 0;

        while true {
            row, ok, error := csv_next(*reader,, temp);

            assert(error.kind == .NONE);
            if !ok break;

            count += row.id;
            black_box(row);
            reset_temporary_storage();
        }

        black_box(count);
        csv_reader_free(*reader);
    }
} @bench @samples(10)

// The records of `read_records` and `read_records_plain`, from a chunk source that gives 4096 bytes at a time.
read_records_chunked :: (b: *Bench) {
    data := benchmark_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 black_box(count_fields(data, .{ header = true }, 4096));
} @bench @samples(10)

read_records_plain_chunked :: (b: *Bench) {
    data := plain_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 black_box(count_fields(data, .{ header = true }, 4096));
} @bench @samples(10)

read_records_plain_trim_chunked :: (b: *Bench) {
    data := plain_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 black_box(count_fields(data, .{ header = true, trim = true }, 4096));
} @bench @samples(10)
