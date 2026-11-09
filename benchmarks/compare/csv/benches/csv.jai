// Csv against rluba/jai-csv on the ~50 MB mixed file `build_portable.py bench` writes for the
// stdlib benchmark: one header, then a row of an integer, a quoted field with a comma, a float, a
// word and a quoted field with doubled quotes, about a million times. The text is rebuilt here
// (same row, LF endings) because the benchmark runs from a scratch copy that cannot read target/.
//
// jai-csv ends a record only at LF and has no record reader: "records" below is its public
// `csv_consume_col` called field by field, and "typed" is its `csv_parse`, which takes a header and
// supports only string, integer and float members, so the typed struct has no bool and `active` is
// skipped through the column callback. Both libraries read the same LF text and fill the same struct.
//
// The community library is imported from corpus/upstream through `-import_dir`. Run through
// tools/bench/library_compare.py, which keeps it inside the sandbox.
//
// Every iteration allocates from a bump pool that is emptied afterwards, for both libraries; jai-csv's
// temporary strings go to the temporary storage, which is reset per record (records) or per iteration.
#import "Basic";
#import "String";
#import "Flat_Pool";
#import "Extensions/Csv";
#import "Extensions/Testing";

jaicsv :: #import "rluba--jai-csv";

Row :: struct {
    id: int;
    name: string;
    value: float64;
    note: string;
}

ROWS :: 1_000_000;
WRITE_ROWS :: 100_000;
COLUMNS :: 5;

text: string;
pool: Flat_Pool;
prepared := false;

// What the writers append: the first WRITE_ROWS records, decoded once by Csv.
written: [..][] string;
written_text: string;

// A library that rejects the input or disagrees with the other would make its benchmark measure a
// failure path.
require :: (ok: bool, what: string) {
    if !ok {
        print("csv comparison: %\n", what);
        exit(1);
    }
}

make_text :: () -> string {
    builder: String_Builder;

    append(*builder, "id,name,value,active,note\n");

    for 0..ROWS - 1 {
        append(*builder, "12345,\"Ada, Lovelace\",3.14159,true,\"a \"\"quoted\"\" line\"\n");
    }

    return builder_to_string(*builder);
}

jacket_count :: (input: string) -> int, int {
    reader := csv_reader(cast([] u8) input, .{ header = true });
    records := 0;
    fields := 0;

    while true {
        row, ok, error := csv_next(*reader);

        require(error.kind == .NONE, "Csv rejected the benchmark file");
        if !ok break;

        records += 1;
        fields += row.count;
    }

    csv_reader_free(*reader);

    return records, fields;
}

// One record at a time through `csv_consume_col`, the only record-level view jai-csv offers.
jaicsv_count :: (input: string) -> int, int {
    remaining := input;
    records := 0;
    fields := 0;

    while true {
        _, ok, last := jaicsv.csv_consume_col(*remaining, #char ",");

        require(ok, "jai-csv rejected the header");
        if last break;
    }

    while remaining.count {
        while true {
            _, ok, last := jaicsv.csv_consume_col(*remaining, #char ",");

            require(ok, "jai-csv rejected the benchmark file");
            fields += 1;
            if last break;
        }

        records += 1;
        reset_temporary_storage();
    }

    return records, fields;
}

name_of :: (name: string, index: int, data: int) -> string {
    if name == "active" return "";
    return name;
}

jaicsv_typed :: (input: string) -> [..] Row, bool {
    return jaicsv.csv_parse(input, Row, 0, name_of);
}

write_jacket :: (builder: *String_Builder) {
    for written {
        error := csv_write_record(builder, it, .{ line_ending = .LF });
        require(error.kind == .NONE, "Csv rejected a record to write");
    }
}

write_jaicsv :: (builder: *String_Builder) {
    for record: written {
        for field, index: record {
            if index append(builder, #char ",");
            jaicsv.append_csv_escaped(builder, field);
        }

        append(builder, #char "\n");
    }
}

prepare :: () {
    if prepared return;

    init(*pool, 1024 * 1024 * 1024);
    text = make_text();

    // Both libraries must see the same records and fields, the same typed rows and write the same bytes.
    jacket_records, jacket_fields := jacket_count(text);
    jaicsv_records, jaicsv_fields := jaicsv_count(text);

    require(jacket_records == ROWS && jacket_fields == ROWS * COLUMNS, "Csv counted the wrong records");
    require(jaicsv_records == jacket_records && jaicsv_fields == jacket_fields, "the libraries disagree on the record counts");

    reader := csv_reader(cast([] u8) text, .{ header = true });

    for 0..WRITE_ROWS - 1 {
        row, ok, error := csv_next(*reader);

        require(ok && error.kind == .NONE, "Csv could not read a record to write");
        array_add(*written, csv_record_copy(row));
    }

    csv_reader_free(*reader);

    jaicsv_rows, jaicsv_ok := jaicsv_typed(text);

    require(jaicsv_ok && jaicsv_rows.count == ROWS, "jai-csv could not parse the typed rows");
    reader = csv_reader(cast([] u8) text, .{ header = true });

    for jaicsv_rows {
        row, ok, error := csv_next_row(Row, *reader);

        require(ok && error.kind == .NONE, "Csv could not read a typed row");
        require(row.id == it.id && row.name == it.name && row.value == it.value && row.note == it.note, "the libraries disagree on a typed row");
        csv_row_free(*row);
    }

    csv_reader_free(*reader);

    jacket_builder: String_Builder;
    jaicsv_builder: String_Builder;

    write_jacket(*jacket_builder);
    write_jaicsv(*jaicsv_builder);
    written_text = builder_to_string(*jacket_builder);
    require(written_text == builder_to_string(*jaicsv_builder), "the libraries write different bytes");

    reset_temporary_storage();
    prepared = true;
}

// ---------------------------------------------------------------------------------- Jacket

records__jacket :: (b: *Bench) {
    prepare();
    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        records, fields := jacket_count(black_box(text));

        black_box(records);
        black_box(fields);
        reset(*pool);
    }
} @bench @samples(10)

typed__jacket :: (b: *Bench) {
    prepare();
    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        reader := csv_reader(cast([] u8) black_box(text), .{ header = true });
        count := 0;

        while true {
            row, ok, error := csv_next_row(Row, *reader);

            require(error.kind == .NONE, "Csv rejected a typed row");
            if !ok break;

            count += 1;
            black_box(row);
        }

        black_box(count);
        reset(*pool);
    }
} @bench @samples(10)

write__jacket :: (b: *Bench) {
    prepare();
    set_bytes(b, written_text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        builder: String_Builder;

        write_jacket(*builder);
        black_box(builder_to_string(*builder));
        reset(*pool);
    }
} @bench @samples(10)

// ---------------------------------------------------------------------------------- jai-csv

records__jaicsv :: (b: *Bench) {
    prepare();
    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        records, fields := jaicsv_count(black_box(text));

        black_box(records);
        black_box(fields);
        reset(*pool);
    }
} @bench @samples(10)

typed__jaicsv :: (b: *Bench) {
    prepare();
    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        rows, ok := jaicsv_typed(black_box(text));

        require(ok, "jai-csv rejected the typed rows");
        black_box(rows);
        reset_temporary_storage();
        reset(*pool);
    }
} @bench @samples(10)

write__jaicsv :: (b: *Bench) {
    prepare();
    set_bytes(b, written_text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        push_allocator(flat_pool_allocator_proc, *pool);
        builder: String_Builder;

        write_jaicsv(*builder);
        black_box(builder_to_string(*builder));
        reset(*pool);
    }
} @bench @samples(10)
