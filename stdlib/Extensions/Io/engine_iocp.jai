// The Windows backends: I/O completion ports for sockets, and for files either the same port
// (`.IOCP`) or, on Windows 11 22H2 and newer, an IoRing (`.IORING`, in engine_ioring.jai).
//
// One completion port per scheduler thread. Every operation is an overlapped call that carries the
// address of its `Op` (the OVERLAPPED is the first member of `Op`, so a completion packet's
// pointer is the operation). The calls are always made with the plain "queue a packet when done"
// behaviour, never "skip the packet on success", so an operation has exactly one way to finish:
// its packet arrives in `iocp_wait`. A call that fails at once (no packet will come) finishes in
// `iocp_submit` instead.
//
//   accept   AcceptEx, on a socket made ahead of time
//   connect  ConnectEx, on a socket bound to the wildcard address first
//   recv     WSARecv
//   send     WSASend, at most SEND_CHUNK bytes at a time
//   read     ReadFile with the offset in the OVERLAPPED (or the IoRing)
//   write    WriteFile likewise
//   cancel   CancelIoEx; the operation still completes, with the abort status
//   close    CancelIoEx for the whole handle, then closesocket/CloseHandle; operations waiting on it
//            complete with the abort status
//
// A packet's status is the NTSTATUS in OVERLAPPED.Internal, read from the dequeued entry rather
// than asked of the (possibly already closed) handle, and turned into a Win32 error with
// RtlNtStatusToDosError. Helper threads (name lookups, the IoRing's event) wake a scheduler by
// posting a packet with no OVERLAPPED and a key (`KEY_WAKE`, `KEY_RING`).

// What the Windows engine keeps. The fields exist everywhere so that `Engine` has one shape.
Iocp_State :: struct {
    port: s64;
    sockets_started: bool;
    accept_ex: *void;
    connect_ex: *void;

    // The IoRing, when files use one (0 otherwise), its completion event, and the thread that turns
    // the event into a packet on the port. The thread starts at the first file operation.
    ring: s64;
    ring_event: s64;
    stop_event: s64;
    ring_thread: Thread.Thread;
    ring_thread_started: bool;
    ring_inflight: s64;
    ring_dirty: bool;
    // File operations waiting for room in the ring's queue.
    backlog_head: *Op;
    backlog_tail: *Op;
}

#if OS == .WINDOWS {
    // Completion packets dequeued per call.
    ENTRIES :: 64;

    iocp_start :: (engine: *Engine, request: Backend) {
        state := New(Iocp_State);

        engine.iocp = state;
        engine.backend = .IOCP;

        assert(win_sockets_start(), "Io: Winsock did not start");
        state.sockets_started = true;

        state.port = win_create_port(-1, 0, 0, 1);
        assert(state.port != 0, "Io: could not create a completion port");

        // AcceptEx and ConnectEx are not exported by name; the system hands out their addresses.
        socket := win_socket(AF_INET, SOCK_STREAM, IPPROTO_TCP, null, 0, WSA_FLAG_OVERLAPPED);

        assert(socket != -1, "Io: could not create a socket");

        state.accept_ex = win_extension(socket, guid_accept_ex());
        state.connect_ex = win_extension(socket, guid_connect_ex());
        win_close_socket(socket);

        assert(state.accept_ex != null && state.connect_ex != null, "Io: Winsock has no AcceptEx/ConnectEx");

        if request == .IO_URING || request == .EPOLL || request == .KQUEUE {
            engine.fallback_reason = "JAI_IO_BACKEND names a backend Windows does not have";
        }

        // Files go through the IoRing unless completion ports were asked for or the ring is not there.
        if request != .IOCP {
            reason := ring_open(state);

            if reason == "" engine.backend = .IORING;
            else if engine.fallback_reason == "" engine.fallback_reason = reason;
        }
    }

    iocp_stop :: (engine: *Engine) {
        state := engine.iocp;
        if !state return;

        ring_close(state);

        win_close_handle(state.port);

        if state.sockets_started win_sockets_stop();

        free(state);
        engine.iocp = null;
    }

    // Wakes the scheduler that owns `port` from any thread.
    iocp_post :: (port: s64, key: u64) {
        win_post_completion(port, 0, key, null);
    }

    // Finishes `op` at once with the Win32 error `error` (negated into the result).
    iocp_fail :: (op: *Op, error: s64) -> bool {
        op.result = -error;
        op.state = .DONE;

        return true;
    }

    // After an overlapped call: the packet will come if it succeeded or is pending.
    iocp_started :: (op: *Op, succeeded: bool, error: s64) -> bool {
        if succeeded || error == ERROR_IO_PENDING return false;

        return iocp_fail(op, error);
    }

    // Starts `op`. True: it finished now (the result is set). False: its packet will arrive.
    iocp_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) -> bool {
        state := engine.iocp;
        handle := op.handle.fd;

        op.overlapped = .{};

        if op.kind == {
            case .CLOSE;
                // Reads and writes the ring still owns must not lose their file.
                if op.handle.kind == .FILE && state.ring_inflight > 0 ring_wait_idle(scheduler, engine);

                op.result = sys_close_handle(op.handle);
                op.state = .DONE;

                return true;

            case .ACCEPT;
                return iocp_accept(state, op);

            case .CONNECT;
                return iocp_connect(state, op);

            case .RECV;
                buffer := Wsa_Buffer.{ cast(u32) min(op.length, MAX_TRANSFER), op.buffer };
                flags: u32;

                result := win_receive(handle, *buffer, 1, null, *flags, *op.overlapped, null);

                return iocp_started(op, result == 0, win_wsa_error());

            case .SEND;
                buffer := Wsa_Buffer.{ cast(u32) min(op.length, SEND_CHUNK), op.buffer };

                result := win_send(handle, *buffer, 1, null, 0, *op.overlapped, null);

                return iocp_started(op, result == 0, win_wsa_error());

            case .READ; #through;
            case .WRITE;
                if state.ring != 0 && op.handle.kind == .FILE return ring_submit(scheduler, engine, op);

                return iocp_file(op);
        }

        return iocp_fail(op, E_INVAL);
    }

    iocp_accept :: (state: *Iocp_State, op: *Op) -> bool {
        // The socket that will be the connection must exist before AcceptEx, in the listener's family.
        name: Sock_Storage;

        result := sys_socket_name(op.handle.fd, *name, false);

        if result < 0 return iocp_fail(op, -result);

        family := cast(s32) (cast(*u8) *name)[0];

        socket := sys_socket(family, true);

        if socket < 0 return iocp_fail(op, -socket);

        op.aux = socket;

        call := cast(Accept_Ex) state.accept_ex;
        received: u32;

        succeeded := call(op.handle.fd, socket, *op.address, 0, ACCEPT_ADDRESS_BYTES, ACCEPT_ADDRESS_BYTES, *received, *op.overlapped) != 0;
        error := win_wsa_error();

        if iocp_started(op, succeeded, error) {
            win_close_socket(socket);

            return true;
        }

        return false;
    }

    iocp_connect :: (state: *Iocp_State, op: *Op) -> bool {
        // ConnectEx wants the socket bound first.
        wildcard: Sock_Storage;

        family := (cast(*u8) *op.address)[0];

        (cast(*u8) *wildcard)[0] = family;

        length: u32 = 16;
        if family == AF_INET6 length = 28;

        result := sys_bind(op.handle.fd, *wildcard, length);
        if result < 0 return iocp_fail(op, -result);

        call := cast(Connect_Ex) state.connect_ex;

        succeeded := call(op.handle.fd, *op.address, cast(s32) op.address_length, null, 0, null, *op.overlapped) != 0;
        error := win_wsa_error();

        return iocp_started(op, succeeded, error);
    }

    // The file offset goes into the OVERLAPPED. A negative one means the end of the file (for
    // appending handles, which only have FILE_APPEND_DATA access) or the start.
    iocp_file :: (op: *Op) -> bool {
        offset := op.offset;

        if offset < 0 {
            if op.kind == .WRITE offset = -1;
            else offset = 0;
        }

        op.overlapped.offset = cast(u32) (offset & 0xffff_ffff);
        op.overlapped.offset_high = cast(u32) ((offset >> 32) & 0xffff_ffff);

        count := cast(u32) min(op.length, MAX_TRANSFER);
        succeeded: bool;

        if op.kind == .READ succeeded = win_read_file(op.handle.fd, op.buffer, count, null, *op.overlapped) != 0;
        else succeeded = win_write_file(op.handle.fd, op.buffer, count, null, *op.overlapped) != 0;

        error := win_error();

        // A read that starts at or beyond the end is a normal end of file.
        if !succeeded && error == ERROR_HANDLE_EOF {
            op.result = 0;
            op.state = .DONE;

            return true;
        }

        return iocp_started(op, succeeded, error);
    }

    iocp_cancel :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {
        // File operations in an IoRing run to the end.
        if op.uncancelable return;

        win_cancel_io(op.handle.fd, *op.overlapped);
    }

    // Takes packets for up to `timeout` nanoseconds (forever if negative), finishing operations.
    iocp_wait :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {
        state := engine.iocp;

        if state.ring != 0 {
            ring_flush(state);
            ring_drain(scheduler, engine);
        }

        milliseconds: u32 = INFINITE;
        if timeout >= 0 milliseconds = cast(u32) min((timeout + 999_999) / 1_000_000, 0x7fff_fff0);

        // A lost event must never hang a fiber: while the ring owns operations, look at it now and then.
        if state.ring_inflight > 0 && milliseconds > RING_POLL_MILLISECONDS milliseconds = RING_POLL_MILLISECONDS;

        entries: [ENTRIES] Overlapped_Entry;
        removed: u32;

        if win_get_completions(state.port, entries.data, ENTRIES, *removed, milliseconds, 0) == 0 {
            error := win_error();

            assert(error == WAIT_TIMEOUT, "Io: GetQueuedCompletionStatusEx failed (%)", error);

            return;
        }

        for 0..cast(s64) removed - 1 {
            entry := *entries[it];

            if !entry.overlapped {
                if entry.key == KEY_WAKE {
                    if scheduler.waker wake_collect(scheduler);
                } else if entry.key == KEY_RING {
                    ring_drain(scheduler, engine);
                }

                continue;
            }

            iocp_complete(scheduler, engine, cast(*Op) entry.overlapped, cast,trunc(s32) entry.internal, entry.bytes);
        }
    }

    // The packet of `op` arrived with this NTSTATUS and byte count.
    iocp_complete :: (scheduler: *Scheduler, engine: *Engine, op: *Op, status: s32, bytes: u32) {
        error := win_status_error(status);
        result: s64;

        if op.kind == {
            case .ACCEPT;
                if error == 0 {
                    // Makes getsockname/getpeername/shutdown work on the new socket.
                    listener := op.handle.fd;
                    win_set_socket_option(op.aux, SOL_SOCKET, SO_UPDATE_ACCEPT_CONTEXT, *listener, 8);

                    result = op.aux;
                } else {
                    win_close_socket(op.aux);

                    result = -error;
                }

            case .CONNECT;
                if error == 0 {
                    win_set_socket_option(op.handle.fd, SOL_SOCKET, SO_UPDATE_CONNECT_CONTEXT, null, 0);

                    result = 0;
                } else {
                    result = -error;
                }

            case .RECV; #through;
            case .SEND; #through;
            case .READ; #through;
            case .WRITE;
                if error == 0 result = bytes;
                else if error == ERROR_HANDLE_EOF result = 0;
                else if bytes > 0 && error == E_CANCELED result = bytes;
                else result = -error;
        }

        finish_op(scheduler, op, result);
    }
} else {
    iocp_start :: (engine: *Engine, request: Backend) {}
    iocp_stop :: (engine: *Engine) {}

    iocp_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) -> bool {
        op.result = -E_UNSUPPORTED;
        op.state = .DONE;

        return true;
    }

    iocp_cancel :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {}
    iocp_wait :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {}
    iocp_post :: (port: s64, key: u64) {}
}
