// Extensions/Postgres against rluba/jai-postgres (libpq bindings) on a local server: point selects,
// bulk inserts and the fetch of wide rows. Both clients connect over TCP to 127.0.0.1, send
// parameters in binary, read results in binary and decode them into structs. Neither uses a
// prepared statement unless the benchmark name says so.
//
// The server and its tables are set up by tools/bench/library_compare.py (setup.sql); the address is
// in PG_BENCH_URL (default postgres://postgres@127.0.0.1:54329/bench). Run through that tool: the
// sandbox it keeps jai-postgres in allows the loopback interface only, for this comparison.
//
//   point_select    one `select ... from items where id = $1` (100,000 rows, primary key) per iteration
//   bulk_insert     truncate a temporary table, then 1,000 single-row inserts inside one transaction
//   wide_fetch      `select * from wide` (1,000 rows of 24 columns) decoded into structs
//   point_select_tls  point_select over TLS (sslmode=require)
//
// Only Jacket has: point_select_prepared (a named prepared statement), bulk_insert_pipeline (the same
// inserts queued in a pipeline) and bulk_insert_copy (one binary COPY of the 1,000 rows).
#import "Basic";
#import "String";
#import "Flat_Pool";
#import "Extensions/Environment";
#import "Extensions/Testing";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

libpq :: #import "rluba--jai-postgres";

DEFAULT_URL :: "postgres://postgres@127.0.0.1:54329/bench";
ROWS :: 1000;

Item :: struct {
    id: int;
    name: string;
    qty: int;
    price: float64;
    active: bool;
    note: string;
}

Wide :: struct {
    id: int;
    i1: int;
    i2: int;
    i3: int;
    b1: s64;
    b2: s64;
    b3: s64;
    r1: float32;
    r2: float32;
    d1: float64;
    d2: float64;
    d3: float64;
    f1: bool;
    f2: bool;
    t1: string;
    t2: string;
    t3: string;
    t4: string;
    t5: string;
    t6: string;
    t7: string;
    t8: string;
    n1: string;
    n2: string;
}

Sink :: struct {
    id: s32;
    name: string;
    qty: s32;
    price: float64;
}

pool: Flat_Pool;
pool_ready := false;
seed: u64 = 424242;

next :: (range: int) -> int {
    seed = seed * 6364136223846793005 + 1442695040888963407;
    return cast(int) ((seed >> 33) % cast(u64) range);
}

url_for :: (tls: bool) -> string {
    url := environment_variable("PG_BENCH_URL");
    if !url url = DEFAULT_URL;
    return tprint("%?sslmode=%", url, ifx tls then "require" else "disable");
}

require_ok :: (ok: bool, what: string) {
    if !ok {
        print("% failed\n", what);
        exit(1);
    }
}

require_pg :: (error: Pg.Pg_Error) {
    if error.kind != .NONE {
        print("%\n", Pg.error_text(error));
        exit(1);
    }
}

// What the benchmark of Jacket does inside the fiber that Io.io_run makes; the body reads these.
bench_in_fiber: *Bench;
fiber_work: (b: *Bench, db: *Pg.Connection);
fiber_tls := false;

run_fiber :: (b: *Bench, tls: bool, work: (b: *Bench, db: *Pg.Connection)) {
    bench_in_fiber = b;
    fiber_work = work;
    fiber_tls = tls;

    Io.io_run(() {
        config, config_error := Pg.parse_url(url_for(fiber_tls));

        require_pg(config_error);

        db, error := Pg.connect(config);

        require_pg(error);
        defer Pg.close(db);

        fiber_work(bench_in_fiber, db);
    });
}

run_libpq :: (b: *Bench, tls: bool, work: (b: *Bench, conn: *libpq.PGconn)) {
    if !pool_ready {
        init(*pool, 256 * 1024 * 1024);
        pool_ready = true;
    }

    // The connection is made with libpq's blocking call; the wrapper's own connect spins on
    // PQconnectPoll without waiting for the socket.
    url := to_c_string(url_for(tls));
    conn := libpq.PQconnectdb(url);

    if libpq.PQstatus(conn) != libpq.ConnStatusType.CONNECTION_OK {
        print("libpq connect: %\n", to_string(libpq.PQerrorMessage(conn)));
        exit(1);
    }

    defer libpq.disconnect(conn);

    work(b, conn);
}

// ----------------------------------------------------------------------------- point selects

SELECT_ITEM :: "select id, name, qty, price, active, note from items where id = $1";

point_select__jacket :: (b: *Bench) {
    run_fiber(b, false, (b: *Bench, db: *Pg.Connection) {
        set_items(b, 1);
        reset_timer(b);

        for 0..b.iterations - 1 {
            result, error := Pg.query_binary(db, SELECT_ITEM, 1 + next(100000));

            require_pg(error);
            item: Item;
            item.id, error = Pg.get(*result, 0, 0, s32);
            item.name, error = Pg.get(*result, 0, 1, string);
            item.qty, error = Pg.get(*result, 0, 2, s32);
            item.price, error = Pg.get(*result, 0, 3, float64);
            item.active, error = Pg.get(*result, 0, 4, bool);
            item.note, error = Pg.get(*result, 0, 5, string);
            require_pg(error);
            black_box(item);
            Pg.free_result(*result);
        }
    });
} @bench

point_select_prepared__jacket :: (b: *Bench) {
    run_fiber(b, false, (b: *Bench, db: *Pg.Connection) {
        set_items(b, 1);
        reset_timer(b);

        for 0..b.iterations - 1 {
            result, error := Pg.query_cached(db, SELECT_ITEM, 1 + next(100000), binary = true);

            require_pg(error);
            item: Item;
            item.id, error = Pg.get(*result, 0, 0, s32);
            item.name, error = Pg.get(*result, 0, 1, string);
            item.qty, error = Pg.get(*result, 0, 2, s32);
            item.price, error = Pg.get(*result, 0, 3, float64);
            item.active, error = Pg.get(*result, 0, 4, bool);
            item.note, error = Pg.get(*result, 0, 5, string);
            require_pg(error);
            black_box(item);
            Pg.free_result(*result);
        }
    });
} @bench

point_select_tls__jacket :: (b: *Bench) {
    run_fiber(b, true, (b: *Bench, db: *Pg.Connection) {
        set_items(b, 1);
        reset_timer(b);

        for 0..b.iterations - 1 {
            result, error := Pg.query_binary(db, SELECT_ITEM, 1 + next(100000));

            require_pg(error);
            name, name_error := Pg.get(*result, 0, 1, string);

            require_pg(name_error);
            black_box(name);
            Pg.free_result(*result);
        }
    });
} @bench

point_select__libpq :: (b: *Bench) {
    run_libpq(b, false, (b: *Bench, conn: *libpq.PGconn) {
        set_items(b, 1);
        reset_timer(b);

        for 0..b.iterations - 1 {
            push_allocator(flat_pool_allocator_proc, *pool);
            items, ok := libpq.execute(conn, Item, SELECT_ITEM, 1 + next(100000));

            require_ok(ok && items.count == 1, "libpq point select");
            black_box(items);
            reset(*pool);
        }
    });
} @bench

point_select_tls__libpq :: (b: *Bench) {
    run_libpq(b, true, (b: *Bench, conn: *libpq.PGconn) {
        set_items(b, 1);
        reset_timer(b);

        for 0..b.iterations - 1 {
            push_allocator(flat_pool_allocator_proc, *pool);
            items, ok := libpq.execute(conn, Item, SELECT_ITEM, 1 + next(100000));

            require_ok(ok && items.count == 1, "libpq point select");
            black_box(items);
            reset(*pool);
        }
    });
} @bench

// ----------------------------------------------------------------------------- bulk insert

INSERT_SINK :: "insert into sink (id, name, qty, price) values ($1, $2, $3, $4)";

// The inserts go into a temporary table named like the real one, which shadows it: the numbers then
// measure the client and the protocol, not the server's write-ahead log and its fsync.
TEMP_SINK :: "create temp table sink (id int not null, name text not null, qty int not null, price float8 not null)";

bulk_insert__jacket :: (b: *Bench) {
    run_fiber(b, false, (b: *Bench, db: *Pg.Connection) {
        set_items(b, ROWS);
        require_pg(Pg.execute_script(db, TEMP_SINK));
        reset_timer(b);

        for 0..b.iterations - 1 {
            require_pg(Pg.execute_script(db, "truncate sink; begin"));

            for row: 0..ROWS - 1 {
                result, error := Pg.query_binary(db, INSERT_SINK, row, "name-value", row % 100, row * 1.5);
                require_pg(error);
                Pg.free_result(*result);
            }

            require_pg(Pg.execute_script(db, "commit"));
        }
    });
} @bench

bulk_insert_pipeline__jacket :: (b: *Bench) {
    run_fiber(b, false, (b: *Bench, db: *Pg.Connection) {
        set_items(b, ROWS);
        require_pg(Pg.execute_script(db, TEMP_SINK));
        reset_timer(b);

        for 0..b.iterations - 1 {
            require_pg(Pg.execute_script(db, "truncate sink"));

            p, error := Pg.pipeline_enter(db);

            require_pg(error);

            for row: 0..ROWS - 1 {
                require_pg(Pg.pipeline_query_binary(p, INSERT_SINK, row, "name-value", row % 100, row * 1.5));

                // A sync every 200 rows sends the batch and lets its answers be read, so they
                // never fill the socket.
                if row % 200 == 199 {
                    require_pg(Pg.pipeline_sync(p));
                    drain(p);
                }
            }

            require_pg(Pg.pipeline_exit(p));
        }
    });
} @bench

drain :: (p: *Pg.Pipeline) {
    while p.queued > 0 {
        item, error := Pg.pipeline_next_result(p);

        require_pg(error);
        done := item.kind == .SYNC;

        Pg.free_pipeline_result(*item);
        if done break;
    }
}

bulk_insert_copy__jacket :: (b: *Bench) {
    rows: [ROWS] Sink;
    for *rows {
        it.id = xx it_index;
        it.name = "name-value";
        it.qty = xx (it_index % 100);
        it.price = it_index * 1.5;
    }

    bulk_rows = rows;

    run_fiber(b, false, (b: *Bench, db: *Pg.Connection) {
        set_items(b, ROWS);
        require_pg(Pg.execute_script(db, TEMP_SINK));
        reset_timer(b);

        for 0..b.iterations - 1 {
            require_pg(Pg.execute_script(db, "truncate sink"));
            stored, error := Pg.copy_in(db, "copy sink (id, name, qty, price) from stdin (format binary)", bulk_rows);
            require_pg(error);
            require_ok(stored == ROWS, "copy_in");
        }
    });
} @bench

bulk_rows: [ROWS] Sink;

bulk_insert__libpq :: (b: *Bench) {
    run_libpq(b, false, (b: *Bench, conn: *libpq.PGconn) {
        set_items(b, ROWS);
        ok, changed := libpq.execute(conn, TEMP_SINK);

        require_ok(ok, "libpq create temp table");
        reset_timer(b);

        for 0..b.iterations - 1 {
            ok, changed = libpq.execute(conn, "truncate sink");
            require_ok(ok, "libpq truncate");
            ok, changed = libpq.execute(conn, "begin");
            require_ok(ok, "libpq begin");

            for row: 0..ROWS - 1 {
                value: int = row;
                ok, changed = libpq.execute(conn, INSERT_SINK, value, "name-value", value % 100, value * 1.5);
                require_ok(ok && changed == 1, "libpq insert");
            }

            ok, changed = libpq.execute(conn, "commit");
            require_ok(ok, "libpq commit");
        }
    });
} @bench

// ----------------------------------------------------------------------------- wide rows

wide_fetch__jacket :: (b: *Bench) {
    run_fiber(b, false, (b: *Bench, db: *Pg.Connection) {
        set_items(b, ROWS);
        reset_timer(b);

        for 0..b.iterations - 1 {
            result, error := Pg.query_binary(db, "select * from wide");

            require_pg(error);
            require_ok(result.row_count == ROWS, "wide row count");

            rows := NewArray(ROWS, Wide,, temp);

            for row: 0..ROWS - 1 {
                w := *rows[row];
                w.id, error = Pg.get(*result, row, 0, s32);
                w.i1, error = Pg.get(*result, row, 1, s32);
                w.i2, error = Pg.get(*result, row, 2, s32);
                w.i3, error = Pg.get(*result, row, 3, s32);
                w.b1, error = Pg.get(*result, row, 4, s64);
                w.b2, error = Pg.get(*result, row, 5, s64);
                w.b3, error = Pg.get(*result, row, 6, s64);
                w.r1, error = Pg.get(*result, row, 7, float32);
                w.r2, error = Pg.get(*result, row, 8, float32);
                w.d1, error = Pg.get(*result, row, 9, float64);
                w.d2, error = Pg.get(*result, row, 10, float64);
                w.d3, error = Pg.get(*result, row, 11, float64);
                w.f1, error = Pg.get(*result, row, 12, bool);
                w.f2, error = Pg.get(*result, row, 13, bool);
                w.t1, error = Pg.get(*result, row, 14, string);
                w.t2, error = Pg.get(*result, row, 15, string);
                w.t3, error = Pg.get(*result, row, 16, string);
                w.t4, error = Pg.get(*result, row, 17, string);
                w.t5, error = Pg.get(*result, row, 18, string);
                w.t6, error = Pg.get(*result, row, 19, string);
                w.t7, error = Pg.get(*result, row, 20, string);
                w.t8, error = Pg.get(*result, row, 21, string);
                n1, n1_error := Pg.get(*result, row, 22, Pg.Numeric);
                n2, n2_error := Pg.get(*result, row, 23, Pg.Numeric);

                w.n1 = n1.text;
                w.n2 = n2.text;
                require_pg(error);
                require_pg(n1_error);
                require_pg(n2_error);
            }

            black_box(rows);
            Pg.free_result(*result);
            reset_temporary_storage();
        }
    });
} @bench

wide_fetch__libpq :: (b: *Bench) {
    run_libpq(b, false, (b: *Bench, conn: *libpq.PGconn) {
        set_items(b, ROWS);
        reset_timer(b);

        for 0..b.iterations - 1 {
            push_allocator(flat_pool_allocator_proc, *pool);
            rows, ok := libpq.execute(conn, Wide, "select * from wide");

            require_ok(ok && rows.count == ROWS, "libpq wide fetch");
            black_box(rows);
            reset(*pool);
        }
    });
} @bench
