// pgbench's built-in TPC-B transaction, run by fibers sharing one pool, as a load test of the
// client (docs/postgres/benchmark.md). It loads its own tables, so it needs a database it may
// drop `pgbench_*` tables in.
//
//     DATABASE_URL=postgres://... jaic tpcb.jai -exe tpcb && ./tpcb -clients 8 -seconds 10 -scale 1

#import "Basic";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

Settings :: struct {
    clients := 8;
    seconds := 10;
    scale := 1;
    load := true; // false: reuse the tables of an earlier run
}

settings: Settings;
pool: *Pg.Pool;
stop_at: s64;
done := 0;
transactions := 0;
failures := 0;
first_failure: string;

main :: () {
    args := get_command_line_arguments();

    index := 1;

    while index < args.count {
        flag := args[index];

        index += 1;

        if flag == "-no-load" {
            settings.load = false;
            continue;
        }

        if index >= args.count usage();

        value := number(args[index]);

        index += 1;

        if flag == {
            case "-clients"; settings.clients = value;
            case "-seconds"; settings.seconds = value;
            case "-scale"; settings.scale = value;
            case; usage();
        }
    }

    url := environment_variable("DATABASE_URL");
    if !url fatal("set DATABASE_URL");

    config, config_error := Pg.parse_url(url);
    if config_error.kind != .NONE fatal(Pg.error_text(config_error));

    Io.io_run((config: Pg.Config) {
        error: Pg.Pg_Error;
        pool, error = Pg.create_pool(config, .{ max_size = settings.clients });
        if error.kind != .NONE fatal(Pg.error_text(error));

        defer Pg.close_pool(pool);

        if settings.load load_tables();

        started := now_milliseconds();

        stop_at = started + settings.seconds * 1000;

        for 1..settings.clients Io.spawn(client, it);
        while done < settings.clients Io.sleep(20);

        elapsed := now_milliseconds() - started;

        print("clients %, scale %, % s\n", settings.clients, settings.scale, formatFloat(elapsed / 1000.0, trailing_width = 1, zero_removal = .NO));
        print("transactions %, failed %\n", transactions, failures);
        print("tps %\n", formatFloat(transactions * 1000.0 / elapsed, trailing_width = 1, zero_removal = .NO));
        print("latency average % ms\n", formatFloat(cast(float64) elapsed * settings.clients / max(transactions, 1), trailing_width = 3, zero_removal = .NO));

        if first_failure print("first failure: %\n", first_failure);
    }, config);

    if failures exit(1);
}

usage :: () {
    fatal("usage: tpcb [-clients N] [-seconds N] [-scale N] [-no-load]");
}

number :: (text: string) -> s64 {
    value, ok := string_to_int(text);
    if !ok || value <= 0 fatal(tprint("not a positive number: %", text));
    return value;
}

fatal :: (message: string) {
    print("%\n", message);
    exit(2);
}

now_milliseconds :: () -> s64 {
    return to_milliseconds(current_time_monotonic());
}

// The four tables pgbench -i makes, with the same row counts per scale step.
load_tables :: () {
    c, error := Pg.acquire(pool);
    if error.kind != .NONE fatal(Pg.error_text(error));

    defer Pg.release(pool, c);

    error = Pg.execute_script(c, "set client_min_messages = warning");
    if error.kind != .NONE fatal(Pg.error_text(error));

    error = Pg.execute_script(c, tprint(#string SQL
drop table if exists pgbench_history, pgbench_tellers, pgbench_accounts, pgbench_branches;
create table pgbench_branches (bid int primary key, bbalance int, filler char(88)) with (fillfactor = 100);
create table pgbench_tellers (tid int primary key, bid int, tbalance int, filler char(84)) with (fillfactor = 100);
create table pgbench_accounts (aid int primary key, bid int, abalance int, filler char(84)) with (fillfactor = 100);
create table pgbench_history (tid int, bid int, aid int, delta int, mtime timestamp, filler char(22));
insert into pgbench_branches (bid, bbalance) select b, 0 from generate_series(1, %1) as b;
insert into pgbench_tellers (tid, bid, tbalance) select t, (t - 1) / 10 + 1, 0 from generate_series(1, 10 * %1) as t;
insert into pgbench_accounts (aid, bid, abalance, filler) select a, (a - 1) / 100000 + 1, 0, '' from generate_series(1, 100000 * %1) as a;
SQL, settings.scale));

    if error.kind != .NONE fatal(Pg.error_text(error));

    // A script of several statements runs in one transaction, which VACUUM refuses.
    error = Pg.execute_script(c, "vacuum analyze pgbench_branches, pgbench_tellers, pgbench_accounts, pgbench_history");
    if error.kind != .NONE fatal(Pg.error_text(error));
}

// One client: TPC-B transactions back to back until the time is up.
client :: (seed: s64) {
    state := cast(u64) seed * 0x9E3779B97F4A7C15 + 1;

    while now_milliseconds() < stop_at {
        aid := random_in(*state, 1, 100000 * settings.scale);
        bid := random_in(*state, 1, settings.scale);
        tid := random_in(*state, 1, 10 * settings.scale);
        delta := random_in(*state, -5000, 5000);

        error := run_transaction(aid, bid, tid, delta);

        if error.kind == .NONE {
            transactions += 1;
        } else {
            failures += 1;
            if !first_failure first_failure = copy_string(Pg.error_text(error));
        }

        reset_temporary_storage();
    }

    done += 1;
}

run_transaction :: (aid: s64, bid: s64, tid: s64, delta: s64) -> Pg.Pg_Error {
    c, error := Pg.acquire(pool);
    if error.kind != .NONE return error;

    defer Pg.release(pool, c);

    Statement :: struct {
        sql: string;
        args: [] Any;
    }

    // pgbench sends BEGIN and END too, so the round trips match.
    statements := Statement.[
        .{ "begin", .[] },
        .{ "update pgbench_accounts set abalance = abalance + $1 where aid = $2", .[delta, aid] },
        .{ "select abalance from pgbench_accounts where aid = $1", .[aid] },
        .{ "update pgbench_tellers set tbalance = tbalance + $1 where tid = $2", .[delta, tid] },
        .{ "update pgbench_branches set bbalance = bbalance + $1 where bid = $2", .[delta, bid] },
        .{ "insert into pgbench_history (tid, bid, aid, delta, mtime) values ($1, $2, $3, $4, current_timestamp)", .[tid, bid, aid, delta] },
        .{ "end", .[] },
    ];

    for statements {
        result: Pg.Result;
        result, error = Pg.query_binary(c, it.sql, ..it.args);
        Pg.free_result(*result);

        if error.kind != .NONE {
            if it_index > 0 {
                rollback: Pg.Result;
                rollback = Pg.query_binary(c, "rollback");
                Pg.free_result(*rollback);
            }

            return error;
        }
    }

    return .{};
}

random_in :: (state: *u64, low: s64, high: s64) -> s64 {
    // xorshift64*
    x := state.*;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    state.* = x;

    span := cast(u64) (high - low + 1);

    return low + cast(s64) ((x * 0x2545F4914F6CDD1D) % span);
}
