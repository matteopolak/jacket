// Run with jacket bench --path stdlib/Extensions/Postgres.
#import "Extensions/Testing";
#import "Basic";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

settings: Settings;
pool: *Pg.Pool;

Settings :: struct {
    clients := 8;
    scale := 1;
    prepared: bool;
}

run_tpcb :: (b: *Bench, prepared: bool) {
    settings = .{ prepared = prepared };
    state := Pg_Bench.{ b };

    Io.io_run((state: *Pg_Bench) {
        b := state.b;
        base := environment_variable("JACKET_OUTPUT_DIR");

        if !base.count base = tprint("%../../../../target", #filepath);

        managed: Pg.Managed_Settings;
        managed.cache_dir = tprint("%/postgres-cache", base);
        managed.data_dir = tprint("%/tpcb-bench", base);
        server, start_error := Pg.start_managed(managed);

        assert(!start_error.count, "%", start_error);
        defer assert(!Pg.stop_managed(*server).count);
        error: Pg.Pg_Error;
        pool, error = Pg.create_pool(server.config, .{ max_size = settings.clients });
        assert(error.kind == .NONE, "%", Pg.error_text(error));
        defer Pg.close_pool(pool);
        load_tables();
        group: Io.Wait_Group;

        Io.add(*group, settings.clients);
        for 1..settings.clients Io.spawn(client, Pg_Job.{ *group, it, b.iterations });

        set_items(b, settings.clients);
        reset_timer(b);
        Io.wait(*group);
    }, *state);
}

Pg_Bench :: struct {
    b: *Bench;
}

Pg_Job :: struct {
    group: *Io.Wait_Group;
    seed: s64;
    iterations: int;
}

client :: (job: Pg_Job) {
    defer Io.done(job.group);
    state := cast(u64) job.seed * 0x9E3779B97F4A7C15 + 1;

    for 0..job.iterations - 1 {
        error := run_transaction(black_box(random_in(*state, 1, 100000 * settings.scale)), random_in(*state, 1, settings.scale), random_in(*state, 1, 10 * settings.scale), random_in(*state, -5000, 5000));

        assert(error.kind == .NONE, "%", Pg.error_text(error));
        black_box(error);
        reset_temporary_storage();
    }
}

fatal :: (message: string) {
    assert(false, "%", message);
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

        if settings.prepared {
            result, error = Pg.query_cached(c, it.sql, ..it.args, binary = true);
        } else {
            result, error = Pg.query_binary(c, it.sql, ..it.args);
        }

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

tpcb_simple :: (b: *Bench) {
    run_tpcb(b, false);
} @bench
tpcb_prepared :: (b: *Bench) {
    run_tpcb(b, true);
} @bench
