// A small job system. Worker threads sleep on a semaphore; `run_job` splits `0..total` into chunks,
// wakes every worker and lets them (and the calling thread) claim chunks from one atomic counter
// until none are left. It returns when every chunk is done and every worker has left the claim
// loop, so the next job can safely reuse the same fields.

Job_Proc :: #type (data: *void, first: s64, count: s64);

Job_System :: struct {
    workers: [] T.Thread;
    wake: T.Semaphore;

    // The job being run. Written by `run_job` before the workers are woken.
    proc: Job_Proc;
    data: *void;
    total: s64;
    chunk: s64;

    // Atomic counters, reset for every job.
    next: s64;
    finished: s64;
    exited: s64;

    quit: bool;
}

// Starts `count` workers. With 0 workers every job runs on the calling thread.
jobs_start :: (jobs: *Job_System, count: s64) -> bool {
    T.init(*jobs.wake, 0);
    jobs.workers = NewArray(count, T.Thread);

    for *worker: jobs.workers {
        worker.data = jobs;

        if !T.thread_init(worker, worker_main, name = "game3d worker") {
            jobs.workers.count = it_index;
            jobs_stop(jobs);

            return false;
        }

        T.thread_start(worker);
    }

    return true;
}

jobs_stop :: (jobs: *Job_System) {
    jobs.quit = true;
    for jobs.workers T.signal(*jobs.wake);

    for *worker: jobs.workers {
        T.thread_is_done(worker, -1);
        T.thread_deinit(worker);
    }

    array_free(jobs.workers);
    jobs.workers = .[];
    T.destroy(*jobs.wake);
}

// Runs `proc` over `0..total` in pieces of at most `chunk` items. `proc` is called with the first
// item of a piece and how many items it covers, possibly from several threads at once, so a piece
// may only touch data that no other piece touches.
run_job :: (jobs: *Job_System, proc: Job_Proc, data: *void, total: s64, chunk: s64) {
    if total <= 0 return;

    if jobs.workers.count == 0 {
        proc(data, 0, total);

        return;
    }

    jobs.proc = proc;
    jobs.data = data;
    jobs.total = total;
    jobs.chunk = chunk;
    atomic_write(*jobs.next, 0);
    atomic_write(*jobs.finished, 0);
    atomic_write(*jobs.exited, 0);

    for jobs.workers T.signal(*jobs.wake);

    claim_chunks(jobs, proc, data, total, chunk);

    // Wait for the stragglers: all items done and every worker out of the claim loop.
    while atomic_read(*jobs.finished) < total || atomic_read(*jobs.exited) < jobs.workers.count {
        sleep_milliseconds(0);
    }
}

#scope_file

claim_chunks :: (jobs: *Job_System, proc: Job_Proc, data: *void, total: s64, chunk: s64) {
    while true {
        first := atomic_add(*jobs.next, chunk);

        if first >= total break;

        count := min(chunk, total - first);

        proc(data, first, count);
        atomic_add(*jobs.finished, count);
    }
}

worker_main :: (thread: *T.Thread) -> s64 {
    jobs := cast(*Job_System) thread.data;

    while true {
        T.wait_for(*jobs.wake, -1);

        if jobs.quit break;

        claim_chunks(jobs, jobs.proc, jobs.data, jobs.total, jobs.chunk);
        atomic_add(*jobs.exited, 1);
    }

    return 0;
}
