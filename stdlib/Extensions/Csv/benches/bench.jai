#import "Basic";
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

benchmark_input :: () -> string {
    base := environment_variable("JACKET_OUTPUT_DIR");
    if !base.count base = tprint("%../../../../target", #filepath);

    data, ok := read_entire_file(tprint("%/csv-conformance/bench.csv", base));

    assert(ok, "run PYTHONPATH=tools python3 tools/portable/build_portable.py bench");

    return data;
}

read_records :: (b: *Bench) {
    data := benchmark_input();

    defer free(data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        reader := csv_reader(cast([] u8) black_box(data), .{ header = true });
        count := 0;

        while true {
            row, ok, error := csv_next(*reader);

            assert(error.kind == .NONE);
            if !ok break;

            count += row.count;
            black_box(row);
        }

        black_box(count);
        csv_reader_free(*reader);
    }
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
