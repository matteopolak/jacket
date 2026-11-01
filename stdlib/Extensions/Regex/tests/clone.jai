// `Regex.clone` shares the compiled pattern: it gives everything back whatever the order of the frees, and clones searched from
// several threads at once agree with the original.
// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)")
#import "Basic";
T :: #import "Thread";
Regex :: #import "Extensions/Regex";

Tracking :: struct {
    base: Allocator;
    outstanding: s64;
}

tracking_allocator :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;
    result := state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);

    if mode == .ALLOCATE && result state.outstanding += 1;
    if mode == .RESIZE && !old_memory && result state.outstanding += 1;
    if mode == .FREE && old_memory state.outstanding -= 1;

    return result;
}

PATTERN :: "(?i)\\b(\\w+)ing\\b|(?<number>[0-9]{3})-[0-9]+";
LINES :: string.["Running late", "call 555-1234 now", "naïve jumping über 12-3", "no match here", "ÉCOLE singing 007-99", "x"];

// How many matches the pattern finds in all the lines, `rounds` times over.
count_matches :: (re: *Regex.Regex, rounds: int) -> int {
    total: int;

    for 1..rounds {
        for line: LINES {
            matches := Regex.find_all(re, line);

            total += matches.count;
            array_free(matches);

            caps, found := Regex.captures(re, line);

            if found {
                total += 1;
                array_free(caps.slots);
            }
        }
    }

    return total;
}

Job :: struct {
    re: Regex.Regex;
    rounds: int;
    total: int;
}

search_thread :: (thread: *T.Thread) -> s64 {
    job := cast(*Job) thread.data;

    job.total = count_matches(*job.re, job.rounds);

    return 0;
}

free_in_order :: (order: [] int) {
    original, ok := Regex.compile(PATTERN);

    assert(ok);

    regexes: [4] Regex.Regex;

    regexes[0] = original;
    regexes[1] = Regex.clone(*original);
    regexes[2] = Regex.clone(*original);
    regexes[3] = Regex.clone(*regexes[1]); // a clone of a clone

    expected := count_matches(*original, 1);

    for index: order {
        assert(count_matches(*regexes[index], 1) == expected);
        Regex.free_regex(*regexes[index]);
        assert(regexes[index].engine == null);
    }
}

main :: () {
    tracking := Tracking.{ base = context.allocator };
    scoped := context;

    scoped.allocator = .{ tracking_allocator, *tracking };

    push_context scoped {
        // The original first, last and in the middle; every regex is searched before it is freed.
        free_in_order(.[0, 1, 2, 3]);
        free_in_order(.[3, 2, 1, 0]);
        free_in_order(.[1, 0, 3, 2]);
    }

    assert(tracking.outstanding == 0, "% allocations left", tracking.outstanding);

    // Threads, each with its own clone, searching at the same time.
    THREADS :: 8;
    original, ok := Regex.compile(PATTERN);

    assert(ok);

    expected := count_matches(*original, 200);
    jobs: [THREADS] Job;
    threads: [THREADS] T.Thread;

    for *job, i: jobs {
        job.re = Regex.clone(*original);
        job.rounds = 200;
        assert(T.thread_init(*threads[i], search_thread));
        threads[i].data = job;
    }

    for *threads T.thread_start(it);

    for *job, i: jobs {
        assert(T.thread_is_done(*threads[i], -1));
        T.thread_deinit(*threads[i]);
        assert(job.total == expected, "thread % found % matches, expected %", i, job.total, expected);
        Regex.free_regex(*job.re);
    }

    Regex.free_regex(*original);
    print("ok\n");
}
