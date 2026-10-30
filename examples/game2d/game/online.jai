// Talking to the backend without stalling the frame: a worker thread takes jobs from a queue, runs
// the blocking API calls (each one starts its own Io scheduler) and leaves the outcome in `status`
// and `board` for the game to read under the lock.

Job_Kind :: enum u8 {
    REGISTER;
    SUBMIT;
    LEADERBOARD;
    QUIT;
}

Job :: struct {
    kind: Job_Kind;
    name: string;
    match: New_Match;
}

Online :: struct {
    enabled: bool;
    thread: Thread;
    lock: Mutex;
    wake: Semaphore;
    jobs: [..] Job;
    api: Api;

    // What the worker found out; the game reads these under `lock` with `online_snapshot`.
    status: string;
    board: [..] Leaderboard_Entry;
    account_id: s64;
    token: string;
    last_rank: s64;
    busy: bool;
}

Online_Snapshot :: struct {
    status: string;
    board: [] Leaderboard_Entry;
    account_id: s64;
    last_rank: s64;
    busy: bool;
}

online_start :: (online: *Online, base_url: string, roots_pem: string, account_id: s64, token: string) -> bool {
    if !base_url return false;

    failure := api_init(*online.api, base_url, roots_pem);

    if failure {
        online.status = copy_string(failure);

        return false;
    }

    online.api.account_id = account_id;
    online.api.token = copy_string(token);
    online.account_id = account_id;
    online.token = copy_string(token);
    online.enabled = true;
    online.status = copy_string("online");

    init(*online.lock);
    init(*online.wake);

    thread_init(*online.thread, online_worker);

    online.thread.data = online;
    thread_start(*online.thread);

    return true;
}

online_stop :: (online: *Online) {
    if !online.enabled return;

    online_submit_job(online, .{ kind = .QUIT });

    // A request may be in flight (its timeout is 5 s): wait for the worker to see the QUIT.
    for 1..100 if thread_is_done(*online.thread, 100) break;

    thread_deinit(*online.thread);
    online.enabled = false;
}

online_submit_job :: (online: *Online, job: Job) {
    lock(*online.lock);

    array_add(*online.jobs, job);
    online.busy = true;

    unlock(*online.lock);
    signal(*online.wake);
}

online_register :: (online: *Online, name: string) {
    if online.enabled online_submit_job(online, .{ kind = .REGISTER, name = copy_string(name) });
}

online_submit :: (online: *Online, match: New_Match) {
    if online.enabled online_submit_job(online, .{ kind = .SUBMIT, match = match });
}

online_refresh_board :: (online: *Online) {
    if online.enabled online_submit_job(online, .{ kind = .LEADERBOARD });
}

// A copy of what the worker has found, valid until the next call (it lives in temporary storage).
online_snapshot :: (online: *Online) -> Online_Snapshot {
    snapshot: Online_Snapshot;

    if !online.enabled {
        snapshot.status = "offline";

        return snapshot;
    }

    lock(*online.lock);
    defer unlock(*online.lock);

    snapshot.status = copy_string(online.status,, temp);
    snapshot.account_id = online.account_id;
    snapshot.last_rank = online.last_rank;
    snapshot.busy = online.busy;
    snapshot.board.count = online.board.count;
    snapshot.board.data = alloc(online.board.count * size_of(Leaderboard_Entry),, temp);

    for online.board {
        snapshot.board[it_index] = it;
        snapshot.board[it_index].player = copy_string(it.player,, temp);
    }

    return snapshot;
}

online_set_status :: (online: *Online, status: string) {
    lock(*online.lock);
    defer unlock(*online.lock);

    free(online.status);
    online.status = copy_string(status);
}

online_worker :: (thread: *Thread) -> s64 {
    online := cast(*Online) thread.data;

    while true {
        wait_for(*online.wake);

        lock(*online.lock);

        if online.jobs.count == 0 {
            unlock(*online.lock);
            continue;
        }

        job := online.jobs[0];

        array_ordered_remove_by_index(*online.jobs, 0);
        unlock(*online.lock);

        if job.kind == .QUIT break;

        run_job(online, job);

        lock(*online.lock);
        online.busy = online.jobs.count > 0;
        unlock(*online.lock);

        reset_temporary_storage();
    }

    return 0;
}

run_job :: (online: *Online, job: Job) {
    if #complete job.kind == {
        case .QUIT;
        case .REGISTER;
            account, failure := api_create_account(*online.api, job.name);

            if failure {
                online_set_status(online, tprint("sign-up failed: %", failure));

                return;
            }

            lock(*online.lock);
            online.account_id = account.id;
            free(online.token);
            online.token = copy_string(account.token);
            unlock(*online.lock);

            online_set_status(online, tprint("signed up as %", account.name));
        case .SUBMIT;
            if !online.api.token {
                online_set_status(online, "not signed up: score not sent");

                return;
            }

            match, failure := api_submit_match(*online.api, job.match);

            if failure {
                online_set_status(online, tprint("score not sent: %", failure));

                return;
            }

            lock(*online.lock);
            online.last_rank = match.rank;
            unlock(*online.lock);

            online_set_status(online, tprint("score sent, rank %", match.rank));
        case .LEADERBOARD;
            entries, failure := api_leaderboard(*online.api, 8);

            if failure {
                online_set_status(online, tprint("leaderboard: %", failure));

                return;
            }

            lock(*online.lock);

            for online.board free(it.player);

            array_reset(*online.board);

            for entries {
                copy := it;

                copy.player = copy_string(it.player);
                array_add(*online.board, copy);
            }

            unlock(*online.lock);
    }
}
