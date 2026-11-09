// Run with jacket bench --path stdlib/Extensions/Postgres.
// `fetch_all` on a result built in memory (text format), so no server is needed: decoding the
// rows and copying them, with their strings and arrays, into one block.
#import "Basic";
#import "Extensions/Testing";
Fetch_Pg :: #import "Extensions/Postgres";

Bench_Row :: struct {
    id: s64;
    email: string;
    name: Fetch_Pg.Nullable(string);
    tags: [] string;
}

ROWS :: 200;

// `ROWS` rows of id, email, name and tags, as the server would send them in text format.
make_result :: () -> Fetch_Pg.Result {
    result: Fetch_Pg.Result;
    result.allocator = context.allocator;
    result.row_count = ROWS;

    names := string.["id", "email", "name", "tags"];
    oids := Fetch_Pg.Oid.[.INT8, .TEXT, .TEXT, .TEXT];

    result.columns = NewArray(4, Fetch_Pg.Column);

    for 0..3 {
        result.columns[it].name = copy_string(names[it]);
        result.columns[it].type_oid = xx oids[it];
        result.columns[it].format = .TEXT;
    }

    // The tags column is a text[] in the real thing; the text form of one is read the same way.
    result.columns[3].type_oid = 1009;

    for row: 0..ROWS - 1 {
        add_value(*result, tprint("%", row + 1));
        add_value(*result, tprint("user%@example.com", row));
        if row % 3 == 0 array_add(*result.values, .{ result.data.count, -1 });
        else add_value(*result, tprint("User number %", row));

        add_value(*result, "{alpha,beta,gamma}");
    }

    return result;
}

add_value :: (result: *Fetch_Pg.Result, text: string) {
    array_add(*result.values, .{ result.data.count, text.count });
    for text array_add(*result.data, it);
}

fetch_all_rows :: (b: *Bench) {
    rows: Fetch_Pg.Rows(Bench_Row);
    rows.result = make_result();
    defer Fetch_Pg.free_rows(*rows);

    set_items(b, ROWS);
    reset_timer(b);

    for 0..b.iterations - 1 {
        items := Fetch_Pg.fetch_all(*rows, temp);

        assert(items.count == ROWS && items[1].email == "user1@example.com");
        black_box(items);

        // The tags decoded into the result: give them back, or they pile up over the iterations.
        for rows.result.allocations free(it,, rows.result.allocator);

        rows.result.allocations.count = 0;

        reset_temporary_storage();
    }
} @bench
