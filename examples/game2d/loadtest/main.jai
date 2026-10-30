// A load test for the game2d backend. Every worker is a fiber with its own account that submits
// matches and reads the leaderboard in a loop; the workers are spread over several scheduler
// threads. It prints throughput and latency percentiles and exits 1 when any request failed.
//
//     jaic examples/game2d/loadtest/main.jai -exe game2d-loadtest -o
//     ./game2d-loadtest --url http://127.0.0.1:8080 --threads 2 --workers 16 --rounds 50

#import "Basic";
#import "Sort";
#import "Thread";
Io :: #import "Extensions/Io";
Args :: #import "Extensions/Args";
#import,dir "../shared";
#import "File";

Cli :: struct {
    url: string = "http://127.0.0.1:8080"; @short=u
    roots: string; @value=FILE
    threads: int = 2; @short=t @range(1,64)
    workers: int = 8; @short=w @range(1,10000)
    rounds: int = 25; @short=n @range(1,1000000)
}

HELP :: Args.Help(Cli).{
    about = "Hammers the game2d backend with concurrent players.",
    version = "0.1.0",
    url = "Base URL of the server",
    roots = "PEM certificate to trust for an https URL",
    threads = "Scheduler threads",
    workers = "Concurrent players per thread",
    rounds = "Matches each player submits",
};

Kind :: enum u8 {
    ACCOUNT;
    MATCH;
    LEADERBOARD;
    HEALTH;
}

Sample :: struct {
    kind: Kind;
    seconds: float64;
}

Shared_Results :: struct {
    lock: Mutex;
    samples: [..] Sample;
    failures: [..] string;
    requests: s64;
    failed: s64;
}

Load :: struct {
    cli: Cli;
    roots_pem: string;
    run_id: s64;
    results: Shared_Results;
}

Worker :: struct {
    load: *Load;
    thread: s64;
    index: s64;
    group: *Io.Wait_Group;
}

Recorder :: struct {
    samples: [..] Sample;
    failures: [..] string;
    requests: s64;
    failed: s64;
}

record :: (recorder: *Recorder, kind: Kind, started: float64, failure: string) {
    recorder.requests += 1;

    if failure {
        recorder.failed += 1;

        if recorder.failures.count < 5 array_add(*recorder.failures, copy_string(failure));

        return;
    }

    array_add(*recorder.samples, .{ kind, seconds_since_init() - started });
}

play :: (worker: *Worker) {
    defer Io.done(worker.group);

    load := worker.load;
    recorder: Recorder;
    api: Api;

    failure := api_init(*api, load.cli.url, load.roots_pem);

    if failure {
        record(*recorder, .ACCOUNT, seconds_since_init(), failure);
        merge(load, *recorder);

        return;
    }

    name := tprint("lt%_%_%", load.run_id, worker.thread, worker.index);
    started := seconds_since_init();

    _, failure = api_create_account(*api, name);

    record(*recorder, .ACCOUNT, started, failure);

    if failure {
        merge(load, *recorder);

        return;
    }

    seed: u64 = cast(u64) (load.run_id * 1000 + worker.thread * 100 + worker.index);

    for round: 1..load.cli.rounds {
        // A believable match: a minute of play and a score the kills can explain.
        kills := cast(s64) (next_random(*seed) % 40) + 5;
        match := New_Match.{ seed = cast(s64) next_random(*seed), kills = kills, score = kills * 10, duration_ms = 60_000 };

        started = seconds_since_init();
        _, failure = api_submit_match(*api, match);
        record(*recorder, .MATCH, started, failure);

        if round % 5 == 0 {
            started = seconds_since_init();
            entries, board_failure := api_leaderboard(*api, 10);

            if !board_failure && entries.count == 0 board_failure = "the leaderboard is empty after a match";

            record(*recorder, .LEADERBOARD, started, board_failure);
        }

        if round % 10 == 0 {
            started = seconds_since_init();
            _, failure = api_health(*api);
            record(*recorder, .HEALTH, started, failure);
        }

        reset_temporary_storage();
    }

    merge(load, *recorder);
}

merge :: (load: *Load, recorder: *Recorder) {
    results := *load.results;

    lock(*results.lock);
    defer unlock(*results.lock);

    for recorder.samples array_add(*results.samples, it);
    for recorder.failures if results.failures.count < 5 array_add(*results.failures, it);

    results.requests += recorder.requests;
    results.failed += recorder.failed;
}

next_random :: (state: *u64) -> u64 {
    state.* = state.* * 6364136223846793005 + 1442695040888963407;

    return state.* >> 33;
}

run_thread :: (load: *Load) {
    group: Io.Wait_Group;
    thread := cast(s64) context.thread_index;

    for 0..load.cli.workers - 1 {
        worker := New(Worker);

        worker.* = .{ load, thread, it, *group };

        Io.add(*group);
        Io.spawn(play, worker);
    }

    Io.wait(*group);
}

percentile :: (sorted: [] float64, fraction: float64) -> float64 {
    if sorted.count == 0 return 0;

    index := cast(s64) (cast(float64) (sorted.count - 1) * fraction);

    return sorted[index];
}

report :: (load: *Load, kind: Kind) {
    times: [..] float64;
    times.allocator = temp;

    for load.results.samples if it.kind == kind array_add(*times, it.seconds);

    if times.count == 0 return;

    quick_sort(times, (a, b) => ifx a < b then -1 else ifx a > b then 1 else 0);

    print("  %: % requests, p50 % ms, p99 % ms, max % ms\n", kind, times.count,
        formatFloat(percentile(times, 0.5) * 1000, trailing_width = 2, zero_removal = .NO),
        formatFloat(percentile(times, 0.99) * 1000, trailing_width = 2, zero_removal = .NO),
        formatFloat(times[times.count - 1] * 1000, trailing_width = 2, zero_removal = .NO));
}

main :: () {
    cli, _ := Args.parse(Cli, HELP);

    load: Load;

    load.cli = cli;
    load.run_id = to_milliseconds(current_time_monotonic()) % 1_000_000;
    init(*load.results.lock);

    if cli.roots {
        pem, ok := read_entire_file(cli.roots);

        if !ok {
            print("cannot read %\n", cli.roots);
            exit(1);
        }

        load.roots_pem = pem;
    }

    print("load test: % threads x % players x % matches against %\n", cli.threads, cli.workers, cli.rounds, cli.url);

    started := seconds_since_init();

    Io.io_run_threads(cli.threads, run_thread, *load);

    elapsed := seconds_since_init() - started;
    results := *load.results;

    print("% requests in % s (% per second), % failed\n", results.requests, formatFloat(elapsed, trailing_width = 2, zero_removal = .NO),
        cast(s64) (cast(float64) results.requests / elapsed), results.failed);

    for kind: enum_values_as_enum(Kind) report(*load, kind);

    for results.failures print("  failure: %\n", it);

    if results.failed != 0 || results.requests == 0 exit(1);
}
