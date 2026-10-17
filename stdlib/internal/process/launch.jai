create_process :: (process: *Process, args: ..string, working_directory := "", capture_and_return_output := false, arg_quoting := Process_Argument_Quoting.QUOTE_IF_NEEDED, kill_process_if_parent_exits := true) -> success: bool {
    if args.count == 0 || args[0].count == 0 || process.pid != 0 return false;

    handles: Launch_Handles;

    defer cleanup_handles(*handles);
    if !new_pipe(*handles.input) return false;
    if capture_and_return_output && (!new_pipe(*handles.output) || !new_pipe(*handles.error)) return false;
    if !new_pipe(*handles.control) || !new_pipe(*handles.launch) || !new_pipe(*handles.result) return false;

    argv: [..] *u8;

    defer {
        for arg: argv if arg free(arg);
        array_free(argv);
    }

    for arg: args array_add(*argv, to_c_string(arg));

    array_add(*argv, cast(*u8) null);
    directory := to_c_string(working_directory);

    defer free(directory);
    registry_acquire();
    supervisor := P.fork();
    if supervisor != 0 registry_release();
    if supervisor < 0 return false;

    if supervisor == 0 {
        close_inherited_process_descriptors();
        child_close(handles.input[1]);
        child_close(handles.output[0]);
        child_close(handles.error[0]);
        child_close(handles.control[1]);
        child_close(handles.launch[0]);
        child_close(handles.result[0]);
        supervise(handles, argv.data, directory, working_directory.count > 0, kill_process_if_parent_exits);
        P._exit(125);
    }

    close_fd(*handles.input[0]);
    close_fd(*handles.output[1]);
    close_fd(*handles.error[1]);
    close_fd(*handles.control[0]);
    close_fd(*handles.launch[1]);
    close_fd(*handles.result[1]);
    launch: Launch_Packet;
    received := read_exact(handles.launch[0], *launch, size_of(Launch_Packet));

    close_fd(*handles.launch[0]);

    if !received || launch.error != 0 || launch.pid <= 0 {
        status: s32;
        waited := wait_child(supervisor, *status, 0);

        if waited != supervisor context.logger("process supervisor could not be reaped\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        if launch.error != 0 P.set_errno(cast(P.OS_Error_Code) launch.error);

        return false;
    }

    state := cast(*Process_State) context.allocator.proc(.ALLOCATE, size_of(Process_State), 0, null, context.allocator.data);
    if !state {
        command: u8 = 1;
        sent, bytes := protected_write(handles.control[1], *command, 1);
        status: s32;
        waited := wait_child(supervisor, *status, 0);

        if !sent || bytes != 1 || waited != supervisor context.logger("process cleanup after allocation failure failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

        return false;
    }

    state.* = .{};
    state.process = process;
    state.allocator = context.allocator;
    state.supervisor = supervisor;
    state.control = handles.control[1];
    state.result_fd = handles.result[0];
    handles.control[1] = -1;
    handles.result[0] = -1;
    process.pid = launch.pid;
    process.input.handle = handles.input[1];
    handles.input[1] = -1;
    process.input.eof = false;
    process.output.handle = handles.output[0];
    handles.output[0] = -1;
    process.error.handle = handles.error[0];
    handles.error[0] = -1;
    process.output.eof = !capture_and_return_output;
    process.error.eof = !capture_and_return_output;
    registry_acquire();
    state.next = registry;
    registry = state;
    registry_release();

    if capture_and_return_output && (!make_nonblocking(process.output.handle) || !make_nonblocking(process.error.handle)) {
        cancel_or_report(process);
        deinit(process);
        return false;
    }

    return true;
}

kill_process :: (process: *Process) -> bool {
    state := find_state(process);
    if !state || state.ready return false;

    command: u8 = 1;
    success, count := protected_write(state.control, *command, 1);

    return success && count == 1;
}

deinit :: (process: *Process) {
    close_pipe(*process.input);
    state := find_state(process);
    if state {
        if !state.ready {
            success := get_process_result(process, -1);
            if !success context.logger("process result collection failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        }

        close_fd(*state.control);
        close_fd(*state.result_fd);
        registry_acquire();
        previous: *Process_State;
        cursor := registry;

        while cursor && cursor != state {
            previous = cursor;
            cursor = cursor.next;
        }

        assert(cursor != null);
        if previous previous.next = state.next;
        else registry = state.next;

        registry_release();
        allocator := state.allocator;

        allocator.proc(.FREE, 0, size_of(Process_State), state, allocator.data);
    }

    close_pipe(*process.output);
    close_pipe(*process.error);
    process.pid = 0;
    #if OS == .MACOS close_fd(*process.kevent_queue);
} @Releases

#scope_module
cancel_or_report :: (process: *Process) {
    if kill_process(process) return;
    success, result := get_process_result(process, 0);
    if !success || result.type == .STILL_RUNNING
        context.logger("process cancellation request failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
}

close_pipe :: (pipe: *Process.Pipe) {
    close_fd(*pipe.handle);
    pipe.eof = true;
} @Releases

find_state :: (process: *Process) -> *Process_State {
    registry_acquire();
    defer registry_release();
    state := registry;
    while state {
        if state.process == process return state;
        state = state.next;
    }

    return null;
}

close_fd :: (fd: *s32) {
    registry_acquire();
    defer registry_release();

    if fd.* >= 0 {
        result := P.close(fd.*);
        fd.* = -1;
        if result != 0 context.logger("process descriptor close failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
    }
} @Releases

wait_child :: (pid: P.pid_t, status: *s32, flags: s32) -> P.pid_t #no_context {
    while true {
        result := P.waitpid(pid, status, flags);
        if result >= 0 || P.errno() != P.EINTR return result;
    }

    return -1;
}

#scope_file
Launch_Packet :: struct {
    pid: P.pid_t;
    error: s32;
}

Launch_Handles :: struct {
    input: [2] s32 = .[-1, -1];
    output: [2] s32 = .[-1, -1];
    error: [2] s32 = .[-1, -1];
    control: [2] s32 = .[-1, -1];
    launch: [2] s32 = .[-1, -1];
    result: [2] s32 = .[-1, -1];
}

registry: *Process_State;
registry_lock: s32;

registry_acquire :: () {
    acquired, old := compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1);
    while !acquired acquired, old = compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1);
}

registry_release :: () {
    success := compare_and_swap(*registry_lock, cast(s32) 1, cast(s32) 0);
    assert(success);
} @Releases

close_inherited_process_descriptors :: () #no_context {
    // Another supervisor must not retain this parent's older process controls
    // or stdin writers. The fork snapshot is taken under the registry lock.
    state := registry;
    while state {
        child_close(state.control);
        child_close(state.result_fd);
        child_close(state.process.input.handle);
        child_close(state.process.output.handle);
        child_close(state.process.error.handle);
        #if OS == .MACOS child_close(state.process.kevent_queue);
        state = state.next;
    }

    registry = null;
    registry_lock = 0;
} @Releases

new_pipe :: (pair: *[2] s32) -> bool {
    if P.pipe(pair) != 0 return false;
    return P.fcntl(pair.*[0], P.F_SETFD, P.FD_CLOEXEC) == 0 && P.fcntl(pair.*[1], P.F_SETFD, P.FD_CLOEXEC) == 0;
}

cleanup_handles :: (handles: *Launch_Handles) {
    for *fd: handles.input close_fd(fd);
    for *fd: handles.output close_fd(fd);
    for *fd: handles.error close_fd(fd);
    for *fd: handles.control close_fd(fd);
    for *fd: handles.launch close_fd(fd);
    for *fd: handles.result close_fd(fd);
}

make_nonblocking :: (fd: s32) -> bool {
    if fd < 0 return true;
    flags := P.fcntl(fd, P.F_GETFL);
    return flags >= 0 && P.fcntl(fd, P.F_SETFL, flags | P.O_NONBLOCK) == 0;
}

child_close :: (fd: s32) #no_context {
    if fd >= 0 && P.close(fd) != 0 P._exit(125);
} @Releases

read_exact :: (fd: s32, data: *void, count: s64) -> bool #no_context {
    cursor: s64;
    while cursor < count {
        got := P.read(fd, cast(*u8) data + cursor, cast(u64) (count - cursor));
        if got < 0 {
            if P.errno() == P.EINTR continue;
            return false;
        }

        if got == 0 return false;

        cursor += got;
    }

    return true;
}

write_exact :: (fd: s32, data: *void, count: s64) -> bool #no_context {
    cursor: s64;
    while cursor < count {
        sent := P.write(fd, cast(*u8) data + cursor, cast(u64) (count - cursor));
        if sent < 0 {
            if P.errno() == P.EINTR continue;
            return false;
        }

        if sent == 0 return false;

        cursor += sent;
    }

    return true;
}

send_and_exit :: (fd: s32, data: *void, bytes: s64, status: s32) #no_context {
    if !write_exact(fd, data, bytes) P._exit(125);
    P._exit(status);
}

exec_failure :: (fd: s32) #no_context {
    code := cast(s32) P.errno();
    send_and_exit(fd, *code, 4, 127);
}

stop_child :: (child: P.pid_t, packet: *Result_Packet) #no_context {
    if P.kill(child, P.SIGKILL) != 0 && P.errno() != P.ESRCH packet.error = cast(s32) P.errno();
    if wait_child(child, *packet.status, 0) != child packet.error = cast(s32) P.errno();
}

execute :: (handles: Launch_Handles, error_fd: s32, argv: **u8, directory: *u8, change_directory: bool) #no_context {
    child_close(handles.control[0]);
    child_close(handles.launch[1]);
    child_close(handles.result[1]);
    if P.dup2(handles.input[0], 0) < 0 exec_failure(error_fd);
    if handles.output[1] >= 0 && P.dup2(handles.output[1], 1) < 0 exec_failure(error_fd);
    if handles.error[1] >= 0 && P.dup2(handles.error[1], 2) < 0 exec_failure(error_fd);
    if P.fcntl(0, P.F_SETFD, 0) != 0 exec_failure(error_fd);
    if handles.output[1] >= 0 && P.fcntl(1, P.F_SETFD, 0) != 0 exec_failure(error_fd);
    if handles.error[1] >= 0 && P.fcntl(2, P.F_SETFD, 0) != 0 exec_failure(error_fd);
    if handles.input[0] > 2 child_close(handles.input[0]);
    if handles.output[1] > 2 child_close(handles.output[1]);
    if handles.error[1] > 2 child_close(handles.error[1]);
    if change_directory && P.chdir(directory) != 0 exec_failure(error_fd);
    if P.execvp(argv[0], argv) >= 0 P._exit(125);

    exec_failure(error_fd);
}

supervise :: (handles: Launch_Handles, argv: **u8, directory: *u8, change_directory: bool, kill_on_parent_exit: bool) #no_context {
    errors: [2] s32 = .[-1, -1];
    report: Launch_Packet;

    if P.pipe(*errors) != 0 {
        report.error = cast(s32) P.errno();
        send_and_exit(handles.launch[1], *report, size_of(Launch_Packet), 125);
    }

    if P.fcntl(errors[1], P.F_SETFD, P.FD_CLOEXEC) != 0 {
        report.error = cast(s32) P.errno();
        send_and_exit(handles.launch[1], *report, size_of(Launch_Packet), 125);
    }

    child := P.fork();
    if child < 0 {
        report.error = cast(s32) P.errno();
        send_and_exit(handles.launch[1], *report, size_of(Launch_Packet), 125);
    }

    if child == 0 {
        child_close(errors[0]);
        execute(handles, errors[1], argv, directory, change_directory);
        P._exit(127);
    }

    // Change only the supervisor's disposition, after forking the command.
    if c_signal(P.SIGPIPE, cast(*void) 1) == cast(*void) cast(s64) -1 {
        report.error = cast(s32) P.errno();
        packet: Result_Packet;

        stop_child(child, *packet);
        if packet.error != 0 report.error = packet.error;

        send_and_exit(handles.launch[1], *report, size_of(Launch_Packet), 125);
    }

    child_close(errors[1]);
    child_close(handles.input[0]);
    child_close(handles.output[1]);
    child_close(handles.error[1]);
    error_code: s32;
    count: s64;

    while count < 4 {
        got := P.read(errors[0], cast(*u8) *error_code + count, cast(u64) (4 - count));
        if got < 0 {
            if P.errno() == P.EINTR continue;
            error_code = cast(s32) P.errno();
            break;
        }

        if got == 0 break;

        count += got;
    }

    child_close(errors[0]);

    if error_code != 0 || count != 0 {
        report.error = error_code;
        if report.error == 0 report.error = P.EIO;

        status: s32;
        if wait_child(child, *status, 0) != child report.error = cast(s32) P.errno();

        send_and_exit(handles.launch[1], *report, size_of(Launch_Packet), 125);
    }

    report.pid = child;
    if !write_exact(handles.launch[1], *report, size_of(Launch_Packet)) {
        packet: Result_Packet;
        stop_child(child, *packet);
        P._exit(125);
    }

    child_close(handles.launch[1]);
    control := handles.control[0];

    while true {
        packet: Result_Packet;
        waited := wait_child(child, *packet.status, P.WNOHANG);

        if waited == child send_and_exit(handles.result[1], *packet, size_of(Result_Packet), 0);

        if waited < 0 {
            packet.error = cast(s32) P.errno();
            send_and_exit(handles.result[1], *packet, size_of(Result_Packet), 125);
        }

        pollfd: P.pollfd = .{ fd = control, events = cast(s16) (P.POLLIN | P.POLLHUP) };
        ready := P.poll(*pollfd, 1, 10);

        if ready < 0 {
            if P.errno() == P.EINTR continue;
            packet.error = cast(s32) P.errno();
            stop_child(child, *packet);
            send_and_exit(handles.result[1], *packet, size_of(Result_Packet), 125);
        }

        if ready > 0 {
            command: u8;
            got := P.read(control, *command, 1);

            if got < 0 {
                if P.errno() == P.EINTR continue;
                packet.error = cast(s32) P.errno();
            }

            if (got == 0 && kill_on_parent_exit) || (got == 1 && command == 1) || got < 0 {
                stop_child(child, *packet);
                send_and_exit(handles.result[1], *packet, size_of(Result_Packet), 0);
            }

            if got == 0 {
                child_close(control);
                control = -1;
            }
        }
    }
}

libc :: #system_library "libc";
c_signal :: (signal: s32, handler: *void) -> *void #foreign libc "signal";
