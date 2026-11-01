// The whole search: a work queue of files shared out over a `Thread_Group`.
//
// Each file is one job. A worker scans its jobs into a buffer of its own and notes where in that
// buffer each job's output starts and ends. Once every job is done the buffers are read back in
// the order of the sorted file list, so the output does not depend on which worker got which
// file, or on how many workers there were.

// Runs the search described by `options`.
//
// ```jai
// options := Options.{ pattern = "needle", paths = string.["notes.txt"], fixed = true };
// report := run_search(options);
//
// defer free_report(*report);
// // report.output: "notes.txt:3:5: a needle here\n"
// ```
run_search :: (options: Options) -> Report {
    report: Report;

    checked, problem := make_matcher(options);

    if problem.count > 0 {
        errors: [..] string;

        array_add(*errors, copy_string(problem));
        report.errors = errors;

        return report;
    }

    free_matcher(*checked);

    files, walk_errors := list_files(options);

    defer array_free(files);
    defer array_free(walk_errors);

    jobs := make_jobs(files);

    defer free_jobs(jobs);

    workers := scan_all(options, jobs);

    report.output = join_output(jobs, workers);
    report.errors = collect_errors(walk_errors, jobs);
    report.matched = any_match(jobs);

    for *workers {
        free_matcher(*it.matcher);
        array_free(it.output);
    }

    array_free(workers);

    return report;
}

#scope_file

// One file to search, and what came of it.
Job :: struct {
    path: string;
    worker: s64 = -1; // which worker's buffer holds the output
    start: s64; // where in that buffer
    length: s64;
    scan: Scan_Result;
}

// A thread's own state. Only that thread touches it while the search runs.
Worker :: struct {
    output: [..] u8;
    matcher: Matcher;
    ready: bool;
}

Shared :: struct {
    options: *Options;
    workers: [] Worker;
}

make_jobs :: (files: [] string) -> [] Job {
    jobs := NewArray(files.count, Job);

    for files jobs[it_index].path = it;

    return jobs;
}

free_jobs :: (jobs: [] Job) {
    for jobs {
        free(it.path);
        free(it.scan.error);
    }

    array_free(jobs);
}

// Scans every job on `options.jobs` threads (fewer when there are fewer files) and returns the
// workers, whose buffers the jobs refer to.
scan_all :: (options: Options, jobs: [] Job) -> [] Worker {
    if jobs.count == 0 return .[];

    count := cast(s32) min(max(options.jobs, 1), jobs.count);
    options_copy := options;
    shared := Shared.{ options = *options_copy, workers = NewArray(count, Worker) };

    group: Thread_Group;

    init(*group, count, scan_job, temporary_storage_size = 1 << 20);
    group.data = *shared;
    group.logging = false;
    start(*group);

    for *jobs add_work(*group, it);

    // Work comes back in whatever order it finishes; only the number matters here.
    done := 0;

    while done < jobs.count {
        done += get_completed_work(*group).count;

        if done < jobs.count sleep_milliseconds(1);
    }

    shutdown(*group);

    return shared.workers;
}

scan_job :: (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status {
    job := cast(*Job) work;
    shared := cast(*Shared) group.data;
    index := thread.worker_info.worker_index;
    worker := *shared.workers[index];

    if !worker.ready {
        worker.matcher, _ = make_matcher(shared.options.*);
        worker.ready = true;
    }

    job.worker = index;
    job.start = worker.output.count;
    job.scan = scan_file(*worker.matcher, shared.options.*, job.path, *worker.output);
    job.length = worker.output.count - job.start;

    reset_temporary_storage();

    return .CONTINUE;
}

join_output :: (jobs: [] Job, workers: [] Worker) -> string {
    builder: String_Builder;

    for jobs {
        if it.length == 0 continue;

        buffer := workers[it.worker].output;

        append(*builder, buffer.data + it.start, it.length);
    }

    return builder_to_string(*builder);
}

collect_errors :: (walk_errors: [] string, jobs: [] Job) -> [] string {
    errors: [..] string;

    // The walk's messages are handed over; the jobs keep theirs, so those are copied.
    for walk_errors array_add(*errors, it);

    for jobs {
        if it.scan.error.count > 0 array_add(*errors, copy_string(it.scan.error));
    }

    return errors;
}

any_match :: (jobs: [] Job) -> bool {
    for jobs if it.scan.lines > 0 return true;

    return false;
}
