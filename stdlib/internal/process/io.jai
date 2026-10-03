write_to_process :: (process: *Process, data: [] u8) -> success: bool, bytes_written: int {
    if process.input.handle < 0 || data.count < 0 return false, 0;
    if data.count == 0 return true, 0;
    return protected_write(process.input.handle, data.data, data.count);
}

read_pipe :: (pipe: *Process.Pipe, buffer: [] u8) -> success: bool, bytes_read: int {
    if pipe.eof return true, 0;
    if pipe.handle < 0 || buffer.count <= 0 return false, 0;

    while true {
        count := P.read(pipe.handle, buffer.data, cast(u64) buffer.count);
        if count >= 0 {
            if count == 0 pipe.eof = true;
            return true, count;
        }

        code := P.errno();
        if code == P.EINTR continue;
        if code == P.EAGAIN || code == P.EWOULDBLOCK return true, 0;

        return false, 0;
    }

    return false, 0;
}

read_from_process :: (process: *Process, output_buffer: [] u8, error_buffer: [] u8, timeout_ms := -1) -> success: bool, output_bytes: int, error_bytes: int {
    fds: [2] P.pollfd;
    fds[0].fd = -1;
    fds[1].fd = -1;
    if !process.output.eof && output_buffer.count > 0 fds[0].fd = process.output.handle;
    if !process.error.eof && error_buffer.count > 0 fds[1].fd = process.error.handle;
    if fds[0].fd < 0 && fds[1].fd < 0 return true, 0, 0;
    for *fd: fds fd.events = cast(s16) (P.POLLIN | P.POLLHUP);

    ready := poll_timeout(fds, timeout_ms);
    if ready < 0 return false, 0, 0;
    if ready == 0 return true, 0, 0;

    output_bytes: s64;
    error_bytes: s64;
    success := true;

    if fds[0].revents != 0 {
        ok, count := read_pipe(*process.output, output_buffer);
        success = success && ok;
        output_bytes = count;
    }

    if fds[1].revents != 0 {
        ok, count := read_pipe(*process.error, error_buffer);
        success = success && ok;
        error_bytes = count;
    }

    return success, output_bytes, error_bytes;
}

get_process_result :: (process: *Process, timeout_ms := -1) -> success: bool, Process_Result {
    state := find_state(process);
    if !state return true, Process_Result.{ type = .UNSTARTED };
    if state.ready return !state.failed, state.result;

    fds: [1] P.pollfd = .[.{ fd = state.result_fd, events = cast(s16) (P.POLLIN | P.POLLHUP) }];
    ready := poll_timeout(fds, timeout_ms);

    if ready < 0 return false, Process_Result.{};
    if ready == 0 return true, Process_Result.{ type = .STILL_RUNNING };

    while state.received < size_of(Result_Packet) {
        count := P.read(state.result_fd, cast(*u8) *state.packet + state.received, cast(u64) (size_of(Result_Packet) - state.received));
        if count < 0 {
            if P.errno() == P.EINTR continue;
            state.failed = true;
            break;
        }

        if count == 0 {
            state.failed = true;
            break;
        }

        state.received += count;
    }

    status: s32;
    waited := wait_child(state.supervisor, *status, 0);

    if waited != state.supervisor || state.packet.error != 0 state.failed = true;

    state.ready = true;
    if state.failed return false, Process_Result.{};

    value := state.packet.status;
    if P.WIFEXITED(value) state.result = .{ type = .EXITED, exit_code = P.WEXITSTATUS(value) };
    else if P.WIFSIGNALED(value) state.result = .{ type = .SIGNALED, signal = P.WTERMSIG(value) };
    else {
        state.failed = true;
        return false, Process_Result.{};
    }

    return true, state.result;
}

#scope_module
clock_milliseconds :: () -> s64, bool #no_context {
    value: P.timespec;
    if P.clock_gettime(P.clockid_t.MONOTONIC, *value) != 0 return 0, false;
    return value.tv_sec * 1000 + value.tv_nsec / 1000000, true;
}

poll_timeout :: (fds: [] P.pollfd, timeout: s64) -> s32 {
    deadline: s64;

    if timeout >= 0 {
        now, ok := clock_milliseconds();
        if !ok return -1;
        deadline = now + min(timeout, 0x7fffffffffffffff - now);
    }

    while true {
        interval: s32 = -1;

        if timeout >= 0 {
            now, ok := clock_milliseconds();
            if !ok return -1;
            interval = cast(s32) min(max(cast(s64) 0, deadline - now), cast(s64) 0x7fffffff);
        }

        result := P.poll(fds.data, cast(P.nfds_t) fds.count, interval);
        if result == 0 && timeout > 0 {
            now, ok := clock_milliseconds();
            if !ok return -1;
            if now < deadline continue;
        }

        if result >= 0 || P.errno() != P.EINTR return result;
    }

    return -1;
}

protected_write :: (fd: s32, data: *void, count: s64) -> bool, s64 {
    mask: P.sigset_t;
    old: P.sigset_t;
    pending: P.sigset_t;

    if P.sigemptyset(*mask) != 0 || P.sigaddset(*mask, P.SIGPIPE) != 0 return false, 0;
    if c_pthread_sigmask(P.SIG_BLOCK, *mask, *old) != 0 return false, 0;

    if P.sigpending(*pending) != 0 {
        restored := c_pthread_sigmask(P.SIG_SETMASK, *old, null);
        if restored != 0 context.logger("signal mask restoration failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false, 0;
    }

    existed := P.sigismember(*pending, P.SIGPIPE) != 0;
    sent: s64;

    while true {
        sent = P.write(fd, data, cast(u64) count);
        if sent >= 0 || P.errno() != P.EINTR break;
    }

    error := P.errno();
    valid := true;

    if sent < 0 && error == P.EPIPE && !existed {
        if P.sigpending(*pending) != 0 valid = false;
        else if P.sigismember(*pending, P.SIGPIPE) != 0 {
            signal: s32;
            if P.sigwait(*mask, *signal) != 0 valid = false;
        }
    }

    if c_pthread_sigmask(P.SIG_SETMASK, *old, null) != 0 valid = false;

    P.set_errno(error);
    if sent < 0 return false, 0;

    return valid, sent;
}

#scope_file
libpthread :: #system_library "pthread";
c_pthread_sigmask :: (how: s32, set: *P.sigset_t, old: *P.sigset_t) -> s32 #foreign libpthread "pthread_sigmask";
