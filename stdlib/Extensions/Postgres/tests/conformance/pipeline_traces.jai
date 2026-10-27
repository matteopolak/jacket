// test: skip_on(.WASM)
// Replays the scenarios of PostgreSQL's libpq_pipeline test (src/test/modules/libpq_pipeline) on
// this client and compares the client's message trace with the .trace file libpq wrote for each.
//
// Runs against a live server and skips without PG_PIPELINE_TRACES, the traces directory of the
// fetched sources (tools/pg_conformance/run.py sets it). The server comes from PGHOST (a host, or
// a Unix socket directory), PGPORT and PGUSER.
#import "Basic";
#import "String";
#import "File";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

INT4 :: 23;
TEXT :: 25;
NUMERIC :: 1700;
INTERVAL :: 1186;

traces_dir: string;
failures: s64;

Capture :: struct {
    lines: [..] string;
    notices: s64;
}

main :: () {
    traces_dir = environment_variable("PG_PIPELINE_TRACES");
    if !traces_dir {
        print("skipped: set PG_PIPELINE_TRACES to libpq_pipeline's traces directory\n");
        return;
    }

    Io.io_run(() {
        run("disallowed_in_pipeline", test_disallowed_in_pipeline);
        run("multi_pipelines", test_multi_pipelines);
        run("nosync", test_nosync);
        run("pipeline_abort", test_pipeline_abort);
        run("pipeline_idle", test_pipeline_idle);
        run("prepared", test_prepared);
        run("simple_pipeline", test_simple_pipeline);
        run("singlerow", test_singlerow);
        run("transaction", test_transaction);
    });

    if failures {
        print("% trace(s) differ\n", failures);
        exit(1);
    }

    print("ok\n");
}

run :: (name: string, scenario: (db: *Pg.Connection)) {
    capture: Capture;
    config := server_config();

    config.user_data = *capture;
    config.on_notice = (c: *Pg.Connection, notice: *Pg.Pg_Error) {
        (cast(*Capture) c.user_data).notices += 1;
    };

    db, error := Pg.connect(config);

    check(error, "connect");

    // libpq_pipeline sets these before it starts the trace.
    check(Pg.execute_script(db, "SET lc_messages TO \"C\""), "SET lc_messages");

    ignored := Pg.execute_script(db, "SET debug_parallel_query = off");

    Pg.free_error(*ignored);

    // The scenarios drop their tables first, and the trace holds the notice only if they are gone.
    check(Pg.execute_script(db, "DROP TABLE IF EXISTS pq_pipeline_demo, pq_pipeline_tst"), "drop tables");
    capture.notices = 0;

    Pg.set_trace(db, (c: *Pg.Connection, line: string) {
        capture := cast(*Capture) c.trace_data;
        array_add(*capture.lines, copy_string(line));
    }, *capture);

    scenario(db);
    Pg.close(db);

    compare(name, capture.lines);
}

server_config :: () -> Pg.Config {
    config: Pg.Config;
    config.host = environment_variable("PGHOST");
    if !config.host config.host = "localhost";

    port_text := environment_variable("PGPORT");
    if port_text {
        port, valid := string_to_int(port_text);
        assert(valid, "bad PGPORT");
        config.port = cast(u16) port;
    }

    config.user = environment_variable("PGUSER");
    if !config.user config.user = "postgres";

    config.database = "postgres";
    config.ssl_mode = .DISABLE;

    return config;
}

compare :: (name: string, lines: [] string) {
    path := join(traces_dir, "/", name, ".trace");
    text, ok := read_entire_file(path);

    assert(ok, tprint("cannot read %", path));

    expected := split(text, "\n");
    if expected.count && !expected[expected.count - 1] expected.count -= 1;

    count := min(expected.count, lines.count);

    for 0..count - 1 {
        if expected[it] == lines[it] continue;

        print("% differs at line %:\n  libpq: %\n  ours:  %\n", name, it + 1, expected[it], lines[it]);
        failures += 1;
        return;
    }

    if expected.count != lines.count {
        print("% has % lines, libpq's has %\n", name, lines.count, expected.count);
        failures += 1;
        return;
    }

    print("% ok (% messages)\n", name, lines.count);
}

check :: (error: Pg.Pg_Error, what: string, location := #caller_location) {
    if error.kind == .NONE return;

    print("%:%: %: %\n", location.fully_pathed_filename, location.line_number, what, Pg.error_text(error));
    assert(false);
}

expect :: (condition: bool, what: string, location := #caller_location) {
    if condition return;

    print("%:%: %\n", location.fully_pathed_filename, location.line_number, what);
    assert(false);
}

// The call must fail with a usage error that starts with `message`.
check_refused :: (error: Pg.Pg_Error, message: string, location := #caller_location) {
    expect(error.kind == .USAGE && begins_with(error.message, message), tprint("expected a refusal \"%\", got %", message, Pg.error_text(error)), location);
}

enter :: (db: *Pg.Connection) -> *Pg.Pipeline {
    p, error := Pg.pipeline_enter(db);
    check(error, "pipeline_enter");

    return p;
}

next :: (p: *Pg.Pipeline, kind: Pg.Pipeline_Kind, location := #caller_location) -> Pg.Pipeline_Result {
    item, error := Pg.pipeline_next_result(p);
    check(error, "pipeline_next_result", location);
    expect(item.kind == kind, tprint("expected a % result, got % (%)", kind, item.kind, Pg.error_text(item.error)), location);

    return item;
}

skip :: (p: *Pg.Pipeline, kind: Pg.Pipeline_Kind, location := #caller_location) {
    item := next(p, kind, location);
    Pg.free_pipeline_result(*item);
}

// A failed statement with this SQLSTATE.
skip_error :: (p: *Pg.Pipeline, state: string, location := #caller_location) {
    item := next(p, .ERROR, location);
    expect(Pg.sqlstate_text(item.error.code) == state, tprint("expected error %, got %", state, Pg.error_text(item.error)), location);
    Pg.free_pipeline_result(*item);
}

only_value :: (item: *Pg.Pipeline_Result) -> string {
    expect(item.result.row_count == 1, tprint("expected one row, got %", item.result.row_count));
    value, error := Pg.get(*item.result, 0, 0, string);
    check(error, "get");

    return copy_string(value,, temp);
}

query :: (p: *Pg.Pipeline, sql: string, types: [] u32 = .[], values: [] string = .[], location := #caller_location) {
    check(Pg.pipeline_query_text(p, sql, types, values), sql);
}

// The first column of each row a simple query returns.
Rows :: struct {
    hooks: Pg.Script_Hooks;
    values: [..] string;
}

script_rows :: (db: *Pg.Connection, sql: string) -> [] string {
    rows: Rows;
    rows.hooks.user_data = *rows;
    rows.hooks.on_result = (hooks: *Pg.Script_Hooks, result: *Pg.Result) {
        rows := cast(*Rows) hooks.user_data;

        for 0..result.row_count - 1 {
            value, error := Pg.get(result, it, 0, string);
            check(error, "get");
            array_add(*rows.values, copy_string(value,, temp));
        }
    };

    check(Pg.run_script(db, sql, *rows.hooks), sql);

    return rows.values;
}

script :: (db: *Pg.Connection, sql: string) {
    check(Pg.execute_script(db, sql), sql);
}

test_simple_pipeline :: (db: *Pg.Connection) {
    p := enter(db);

    query(p, "SELECT $1", u32.[INT4], string.["1"]);
    check_refused(Pg.pipeline_exit(p), "cannot exit pipeline mode", #location());
    check(Pg.pipeline_sync(p), "sync");

    item := next(p, .RESULT);

    expect(only_value(*item) == "1", "the first result");
    Pg.free_pipeline_result(*item);

    // The sync point is still to come.
    check_refused(Pg.pipeline_exit(p), "cannot exit pipeline mode", #location());

    skip(p, .SYNC);
    skip(p, .NONE);
    expect(p.active, "left pipeline mode");
    check(Pg.pipeline_exit(p), "exit");
}

test_multi_pipelines :: (db: *Pg.Connection) {
    p := enter(db);

    query(p, "SELECT $1", u32.[INT4], string.["1"]);
    check(Pg.pipeline_sync(p), "sync");
    query(p, "SELECT $1", u32.[INT4], string.["1"]);
    check(Pg.pipeline_queue_sync(p), "queue sync"); // no flush
    query(p, "SELECT $1", u32.[INT4], string.["1"]);
    check(Pg.pipeline_sync(p), "sync");

    skip(p, .RESULT);
    check_refused(Pg.pipeline_exit(p), "cannot exit pipeline mode", #location());
    skip(p, .SYNC);

    skip(p, .RESULT);
    check_refused(Pg.pipeline_exit(p), "cannot exit pipeline mode", #location());
    skip(p, .SYNC);

    skip(p, .RESULT);
    skip(p, .SYNC);
    skip(p, .NONE);
    check(Pg.pipeline_exit(p), "exit");
}

test_nosync :: (db: *Pg.Connection) {
    QUERIES :: 10;

    p := enter(db);

    for 1..QUERIES {
        query(p, "SELECT repeat('xyzxz', 12)");
        check(Pg.pipeline_flush(p), "flush");
    }

    // Tells the server to send what it has.
    check(Pg.pipeline_flush_request(p), "flush request");

    for 1..QUERIES {
        item := next(p, .RESULT);
        expect(only_value(*item) == "xyzxzxyzxzxyzxzxyzxzxyzxzxyzxzxyzxzxyzxzxyzxzxyzxzxyzxzxyzxz", "a repeat() result");
        Pg.free_pipeline_result(*item);
    }

    // Nothing was synced, and nothing is waiting.
    check(Pg.pipeline_exit(p), "exit");
}

test_pipeline_abort :: (db: *Pg.Connection) {
    INSERT :: "INSERT INTO pq_pipeline_demo(itemno) VALUES ($1)";

    script(db, "DROP TABLE IF EXISTS pq_pipeline_demo");
    script(db, "CREATE UNLOGGED TABLE pq_pipeline_demo(id serial primary key, itemno integer,int8filler int8);");

    p := enter(db);

    // The second statement fails, so the third is skipped, and the sync point ends the batch.
    query(p, INSERT, u32.[INT4], string.["1"]);
    query(p, "SELECT no_such_function($1)", u32.[INT4], string.["1"]);
    query(p, INSERT, u32.[INT4], string.["2"]);
    check(Pg.pipeline_sync(p), "sync");
    query(p, INSERT, u32.[INT4], string.["3"]);
    check(Pg.pipeline_sync(p), "sync");

    skip(p, .RESULT);
    expect(!p.aborted, "aborted too early");
    skip_error(p, "42883");
    expect(p.aborted, "the batch should be aborted");
    skip(p, .ABORTED);
    expect(p.aborted, "the batch should be aborted");
    skip(p, .SYNC);
    expect(!p.aborted, "the sync point should end the abort");
    expect(p.active, "left pipeline mode");

    skip(p, .RESULT);
    skip(p, .SYNC);

    // Two commands in one statement.
    query(p, "SELECT 1; SELECT 2");
    check(Pg.pipeline_sync(p), "sync");
    skip_error(p, "42601");
    skip(p, .SYNC);

    // Rows one at a time, then an error part-way.
    query(p, "SELECT 1.0/g FROM generate_series(3, -1, -1) g");
    check(Pg.pipeline_sync(p), "sync");
    expect(Pg.pipeline_single_row(p), "single row mode");

    rows := 0;

    while true {
        item, error := Pg.pipeline_next_result(p);

        check(error, "next");
        defer Pg.free_pipeline_result(*item);

        if item.kind == .ROWS {
            rows += 1;
            continue;
        }

        expect(item.kind == .ERROR && Pg.sqlstate_text(item.error.code) == "22012", "division by zero");
        break;
    }

    expect(rows == 3, "three rows");
    skip(p, .SYNC);
    check(Pg.pipeline_exit(p), "exit");

    // The first batch rolled back; only the second one's insert stayed.
    values := script_rows(db, "SELECT itemno FROM pq_pipeline_demo");

    expect(values.count == 1 && values[0] == "3", "only the insert of 3 remains");
}

test_pipeline_idle :: (db: *Pg.Connection) {
    capture := cast(*Capture) db.user_data;
    p := enter(db);

    query(p, "SELECT 1");
    check(Pg.pipeline_flush_request(p), "flush request");
    skip(p, .RESULT);

    query(p, "SELECT 2");
    check_refused(Pg.pipeline_exit(p), "cannot exit pipeline mode", #location());
    check(Pg.pipeline_flush_request(p), "flush request");
    skip(p, .RESULT);
    check(Pg.pipeline_exit(p), "exit");
    expect(capture.notices == 0, "no notices so far");

    // A warning in the middle of a result.
    p = enter(db);
    query(p, "SELECT pg_catalog.pg_advisory_unlock(1,1)");
    check(Pg.pipeline_flush_request(p), "flush request");
    skip(p, .RESULT);
    check(Pg.pipeline_exit(p), "exit");
    expect(capture.notices == 1, "the warning");
}

test_prepared :: (db: *Pg.Connection) {
    p := enter(db);

    check(Pg.pipeline_prepare(p, "select_one", "SELECT $1, '42', $1::numeric, interval '1 sec'", u32.[INT4]), "prepare");
    check(Pg.pipeline_describe_statement(p, "select_one"), "describe");
    check(Pg.pipeline_sync(p), "sync");

    skip(p, .RESULT);

    item := next(p, .RESULT);

    expect(item.parameter_types.count == 1 && item.parameter_types[0] == INT4, "the parameter type");

    types := u32.[INT4, TEXT, NUMERIC, INTERVAL];

    expect(item.result.columns.count == types.count, "four columns");
    for types expect(item.result.columns[it_index].type_oid == it, "a column type");

    Pg.free_pipeline_result(*item);
    skip(p, .SYNC);

    check(Pg.pipeline_close_statement(p, "select_one"), "close");
    check(Pg.pipeline_sync(p), "sync");
    skip(p, .RESULT);
    skip(p, .SYNC);
    check(Pg.pipeline_exit(p), "exit");

    // Outside a pipeline, a statement that is gone is an error to describe, not to close.
    info, error := Pg.describe_prepared(db, "select_one");

    expect(error.kind == .SERVER && Pg.sqlstate_text(error.code) == "26000", "describing a missing statement");
    Pg.free_error(*error);
    Pg.free_statement_info(*info);
    check(Pg.close_prepared(db, "select_one"), "close_prepared");

    script(db, "BEGIN");
    script(db, "DECLARE cursor_one CURSOR FOR SELECT 1");

    p = enter(db);
    check(Pg.pipeline_describe_portal(p, "cursor_one"), "describe portal");
    check(Pg.pipeline_sync(p), "sync");

    item = next(p, .RESULT);
    expect(item.result.columns.count == 1 && item.result.columns[0].type_oid == INT4, "the portal's column");
    Pg.free_pipeline_result(*item);
    skip(p, .SYNC);

    check(Pg.pipeline_close_portal(p, "cursor_one"), "close portal");
    check(Pg.pipeline_sync(p), "sync");
    skip(p, .RESULT);
    skip(p, .SYNC);
    check(Pg.pipeline_exit(p), "exit");

    info, error = Pg.describe_portal(db, "cursor_one");
    expect(error.kind == .SERVER && Pg.sqlstate_text(error.code) == "34000", "describing a missing portal");
    Pg.free_error(*error);
    Pg.free_statement_info(*info);
    check(Pg.close_portal(db, "cursor_one"), "close_portal");
}

test_singlerow :: (db: *Pg.Connection) {
    p := enter(db);

    // Three statements; the first two deliver one row per result.
    for 0..2 {
        query(p, "SELECT generate_series(42, $1)", .[], string.[tprint("%", 44 + it)]);
    }

    check(Pg.pipeline_sync(p), "sync");

    for 0..2 {
        if it >= 2 {
            item := next(p, .RESULT);
            expect(item.result.row_count == 5, "five rows in one result");
            Pg.free_pipeline_result(*item);
            continue;
        }

        statement := it;

        expect(Pg.pipeline_single_row(p), "single row mode");

        for 1..3 + statement {
            item := next(p, .ROWS);
            expect(item.result.row_count == 1, "one row");
            Pg.free_pipeline_result(*item);
        }

        // The statement ends with a result that holds no rows.
        last := next(p, .RESULT);

        expect(last.result.row_count == 0 && last.result.command == tprint("SELECT %", 3 + statement), "the end of a single row statement");
        Pg.free_pipeline_result(*last);
    }

    skip(p, .SYNC);

    // Single row mode ends with its statement.
    query(p, "SELECT generate_series(0, 0)");
    check(Pg.pipeline_flush_request(p), "flush request");
    expect(Pg.pipeline_single_row(p), "single row mode");
    skip(p, .ROWS);
    skip(p, .RESULT);

    query(p, "SELECT 1");
    check(Pg.pipeline_flush_request(p), "flush request");
    skip(p, .RESULT);

    // Chunks of three: a full one, the rest, and then the tag.
    query(p, "SELECT generate_series(1, 5)");
    check(Pg.pipeline_flush_request(p), "flush request");
    expect(Pg.pipeline_chunked_rows(p, 3), "chunked rows mode");

    item := next(p, .ROWS);

    expect(item.result.row_count == 3 && item.result.partial, "a chunk of three");
    Pg.free_pipeline_result(*item);

    item = next(p, .ROWS);
    expect(item.result.row_count == 2, "the last two rows");
    Pg.free_pipeline_result(*item);

    item = next(p, .RESULT);
    expect(item.result.row_count == 0 && item.result.command == "SELECT 5", "the tag");
    Pg.free_pipeline_result(*item);

    check(Pg.pipeline_exit(p), "exit");
}

test_transaction :: (db: *Pg.Connection) {
    script(db, "DROP TABLE IF EXISTS pq_pipeline_tst;CREATE TABLE pq_pipeline_tst (id int)");

    p := enter(db);

    check(Pg.pipeline_prepare(p, "rollback", "ROLLBACK"), "prepare");
    query(p, "BEGIN");
    query(p, "SELECT 0/0");

    // Skipped: the batch failed, and its transaction has to end before anything else works.
    check(Pg.pipeline_execute_prepared_text(p, "rollback", .[], .BINARY), "execute");
    query(p, "INSERT INTO pq_pipeline_tst VALUES (1)");
    check(Pg.pipeline_sync(p), "sync");

    // Fails: the transaction is still aborted although the batch has ended.
    query(p, "INSERT INTO pq_pipeline_tst VALUES (2)");
    check(Pg.pipeline_sync(p), "sync");

    // Works: ends the transaction, and the insert after it is a new one.
    check(Pg.pipeline_execute_prepared_text(p, "rollback", .[], .BINARY), "execute");
    query(p, "INSERT INTO pq_pipeline_tst VALUES (3)");
    check(Pg.pipeline_sync(p), "sync");
    check(Pg.pipeline_sync(p), "sync");

    skip(p, .RESULT); // prepare
    skip(p, .RESULT); // BEGIN
    skip_error(p, "22012");
    skip(p, .ABORTED);
    skip(p, .ABORTED);
    skip(p, .SYNC);
    expect(db.status == .FAILED, "the transaction is aborted");

    skip_error(p, "25P02");
    skip(p, .SYNC);

    skip(p, .RESULT);
    skip(p, .RESULT);
    skip(p, .SYNC);
    skip(p, .SYNC);
    expect(db.status == .IDLE, "idle again");

    check(Pg.pipeline_exit(p), "exit");

    values := script_rows(db, "SELECT * FROM pq_pipeline_tst");

    expect(values.count == 1 && values[0] == "3", "only the insert of 3 remains");
}

test_disallowed_in_pipeline :: (db: *Pg.Connection) {
    p := enter(db);

    // Requests that wait for their own answer do not mix with a pipeline.
    result, error := Pg.query_dynamic(db, "SELECT 1");

    Pg.free_result(*result);
    check_refused(error, "synchronous command execution functions are not allowed in pipeline mode", #location());

    check_refused(Pg.execute_script(db, "SELECT 1"), "synchronous command execution", #location());

    // Entering again changes nothing.
    expect(enter(db) == p, "entering twice");
    check(Pg.pipeline_exit(p), "exit");
    expect(!p.active, "pipeline mode did not end");

    // Leaving when not in pipeline mode is fine.
    check(Pg.pipeline_exit(p), "exit again");

    values := script_rows(db, "SELECT 1");

    expect(values.count == 1 && values[0] == "1", "a query after the pipeline");
}
