// Formats many files at once. `format_batch` does the reading, formatting and writing on worker
// threads and leaves one `Outcome` per file, in the order given, so the caller reports them in that
// order whatever the schedule was. See docs/tools/jaifmt.md.

Thread :: #import "Thread";
System :: #import "System";

Status :: enum {
    UNCHANGED;
    CHANGED;
    READ_FAILED;
    FORMAT_FAILED;
    WRITE_FAILED;
}

// What happened to one file. The caller fills `path`, `config` and `write`; `format_batch` the rest.
Outcome :: struct {
    path: string;
    config: *Format_Config;
    write: bool; // rewrite the file when it would change
    status: Status;
    error: string; // for FORMAT_FAILED: the formatter's message
    long_lines: int;
    diff_line: int; // for CHANGED: the first line that differs
}

// The worker count when none is asked for: one per logical core.
default_workers :: () -> int {
    return max(1, cast(int) System.get_number_of_processors(.ALL_LOGICAL));
}

// Formats `outcomes[i].path` with `outcomes[i].config` and records the result in the same entry.
// `workers` of 1 runs on the calling thread. Each file is independent, so the outcomes do not
// depend on `workers`.
format_batch :: (outcomes: [] Outcome, workers: int) {
    count := min(workers, outcomes.count);

    if count <= 1 {
        mark := get_temporary_storage_mark();

        for *outcomes {
            format_one(it);
            set_temporary_storage_mark(mark);
        }

        return;
    }

    group: Thread.Thread_Group;
    group.logging = false;
    Thread.init(*group, cast(s32) count, batch_worker, temporary_storage_size = 4 * 1024 * 1024);
    Thread.start(*group);

    for *outcomes Thread.add_work(*group, it);

    Thread.wait_for_all(*group);
    Thread.shutdown(*group);
}

#scope_file

batch_worker :: (group: *Thread.Thread_Group, thread: *Thread.Thread, work: *void) -> Thread.Thread_Continue_Status {
    format_one(cast(*Outcome) work);
    reset_temporary_storage();

    return .CONTINUE;
}

format_one :: (outcome: *Outcome) {
    source, ok := read_entire_file(outcome.path, log_errors = false);
    if !ok {
        outcome.status = .READ_FAILED;
        return;
    }

    defer free(source);

    output, formatted, error := format_source(source, outcome.config.*);
    if !formatted {
        outcome.status = .FORMAT_FAILED;
        outcome.error = copy_string(error);
        return;
    }

    defer free(output);

    outcome.long_lines = format_count_long_lines(output, outcome.config.max_width);

    if output == source {
        outcome.status = .UNCHANGED;
        return;
    }

    outcome.status = .CHANGED;
    outcome.diff_line = first_difference_line(source, output);

    if outcome.write && !write_entire_file(outcome.path, output.data, output.count) {
        outcome.status = .WRITE_FAILED;
    }
}

first_difference_line :: (a: string, b: string) -> int {
    line := 1;

    for i: 0..min(a.count, b.count) - 1 {
        if a[i] != b[i] return line;
        if a[i] == #char "\n" line += 1;
    }

    return line;
}
