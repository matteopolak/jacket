// Run with jacket bench --path stdlib/Extensions/Postgres.
// `copy_in` of structs: encoding each row's members for a binary COPY. A trigger drops the rows
// on the server, so the time is mostly the client's. It uses the server in JAI_PG_TEST_PORT (see
// tests/start_server.sh) when that is set, and a managed server otherwise.
#import "Extensions/Testing";
#import "Basic";
#import "Extensions/Environment";
Copy_Io :: #import "Extensions/Io";
Copy_Pg :: #import "Extensions/Postgres";

Copy_Bench_Row :: struct {
    id: s64;
    score: s32;
    level: s16;
    name: string;
    ratio: float64;
    flag: bool;
    at: Copy_Pg.Timestamp;
    note: Copy_Pg.Nullable(string);
}

COPY_ROWS :: 5000;

Copy_Bench :: struct {
    b: *Bench;
}

copy_in_rows :: (b: *Bench) {
    state := Copy_Bench.{ b };

    Copy_Io.io_run((state: *Copy_Bench) {
        b := state.b;
        config: Copy_Pg.Config;
        server: Copy_Pg.Managed_Server;
        port := environment_variable("JAI_PG_TEST_PORT");

        if port.count {
            config.host = "127.0.0.1";
            value, valid := string_to_int(port);

            assert(valid, "bad JAI_PG_TEST_PORT");
            config.port = cast(u16) value;
            config.user = "postgres";
            config.database = "postgres";
            config.ssl_mode = .DISABLE;
        } else {
            base := environment_variable("JACKET_OUTPUT_DIR");

            if !base.count base = tprint("%../../../../target", #filepath);

            managed: Copy_Pg.Managed_Settings;
            managed.cache_dir = tprint("%/postgres-cache", base);
            managed.data_dir = tprint("%/copy-in-bench", base);

            start_error: string;
            server, start_error = Copy_Pg.start_managed(managed);

            assert(!start_error.count, "%", start_error);
            config = server.config;
        }

        defer if !port.count assert(!Copy_Pg.stop_managed(*server).count);

        db, error := Copy_Pg.connect(config);

        assert(error.kind == .NONE, "%", Copy_Pg.error_text(error));
        defer Copy_Pg.close(db);

        error = Copy_Pg.execute_script(db, #string SQL
create temp table copy_bench (id int8, score int4, level int2, name text, ratio float8, flag bool, at timestamptz, note text);
create function pg_temp.copy_bench_drop() returns trigger as 'begin return null; end' language plpgsql;
create trigger copy_bench_drop before insert on copy_bench for each row execute function pg_temp.copy_bench_drop();
SQL);
        assert(error.kind == .NONE, "%", Copy_Pg.error_text(error));

        rows := NewArray(COPY_ROWS, Copy_Bench_Row);

        defer array_free(rows);

        for *rows {
            it.id = it_index;
            it.score = cast(s32) (it_index * 7);
            it.level = cast(s16) (it_index % 100);
            it.name = "benchmark row name";
            it.ratio = 0.5 * it_index;
            it.flag = it_index % 2 == 0;
            it.at.microseconds = it_index * 1_000_000;

            if it_index % 4 == 0 it.note.null = true;
            else it.note.value = "a note that is a little longer";
        }

        set_items(b, COPY_ROWS);
        reset_timer(b);

        for 0..b.iterations - 1 {
            stored, copy_error := Copy_Pg.copy_in(db, "copy copy_bench from stdin (format binary)", rows);

            assert(copy_error.kind == .NONE, "%", Copy_Pg.error_text(copy_error));
            black_box(stored);
            reset_temporary_storage();
        }
    }, *state);
} @bench
