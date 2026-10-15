// Files on an IoRing (Windows 11 22H2 and newer), for the `.IORING` backend of engine_iocp.jai.
//
// The IoRing API is loaded from KernelBase.dll when the first engine starts, so a Windows without
// it still runs (the files then use the completion port). Before the ring is trusted, a self test
// reads two bytes of the program's own executable through a throwaway ring, waits for the
// completion event and checks the bytes. A system where any step fails uses completion ports for
// files, and `backend_reason()` says which step.
//
// How it fits the engine:
//   - A read or write fills a submission entry (`BuildIoRingReadFile`/`BuildIoRingWriteFile`, with
//     the `Op` as user data); entries reach the kernel in `ring_flush`, which the scheduler's wait
//     calls before it blocks.
//   - Completions are popped from the ring in user space (`ring_drain`). The ring's completion
//     event wakes the scheduler: a helper thread, started at the first file operation, waits on
//     that event and posts a `KEY_RING` packet to the port. While operations are in the ring the
//     scheduler also never sleeps longer than RING_POLL_MILLISECONDS, so a lost event delays
//     rather than hangs.
//   - At most RING_SUBMISSION operations are in the ring; more wait in a list and enter as room
//     frees up.
//   - Operations in a ring cannot be interrupted (`Op.uncancelable`): a cancel waits for the read or
//     write to finish, which takes disk time. Closing a file waits for the ring to go idle.
//   - Files are not tied to the completion port in this mode.
//
// The structs the API takes by value are declared as they are in ioringapi.h; jaic's Win64 and
// Windows arm64 call lowering passes them the way the compiler for C does.

#if OS == .WINDOWS {
    Io_Ring_Flags :: struct {
        required: u32;
        advisory: u32;
    }

    // IORING_HANDLE_REF and IORING_BUFFER_REF: a kind (0: a raw handle or address) and the value.
    Io_Ring_Ref :: struct {
        kind: u32;
        value: u64;
    }

    Io_Ring_Cqe :: struct {
        user_data: u64;
        result: s32;
        information: u64;
    }

    Io_Ring_Capabilities :: struct {
        max_version: u32;
        max_submission: u32;
        max_completion: u32;
        features: u32;
    }

    Ring_Create :: #type (version: u32, flags: Io_Ring_Flags, submission: u32, completion: u32, ring: *s64) -> s32 #c_call;
    Ring_Close :: #type (ring: s64) -> s32 #c_call;
    Ring_Query :: #type (capabilities: *Io_Ring_Capabilities) -> s32 #c_call;
    Ring_Submit :: #type (ring: s64, wait_count: u32, milliseconds: u32, submitted: *u32) -> s32 #c_call;
    Ring_Pop :: #type (ring: s64, cqe: *Io_Ring_Cqe) -> s32 #c_call;
    Ring_Set_Event :: #type (ring: s64, event: s64) -> s32 #c_call;
    Ring_Read :: #type (ring: s64, file: Io_Ring_Ref, buffer: Io_Ring_Ref, count: u32, offset: u64, user_data: u64, flags: u32) -> s32 #c_call;
    Ring_Write :: #type (ring: s64, file: Io_Ring_Ref, buffer: Io_Ring_Ref, count: u32, offset: u64, write_flags: u32, user_data: u64, flags: u32) -> s32 #c_call;

    // The addresses of the functions, found once per process.
    Ring_Api :: struct {
        create: *void;
        close: *void;
        query: *void;
        submit: *void;
        pop: *void;
        set_event: *void;
        read: *void;
        write: *void;
    }

    ring_api: Ring_Api;
    // 0: not probed yet, 1: usable, 2: not.
    ring_probe_state: s32;
    ring_probe_reason: string;

    // The ring versions to try, newest first: IORING_VERSION_3 is 300 in ioringapi.h. File writes
    // need one of the later versions; the self test builds a write entry to find out which a
    // system accepts.
    RING_VERSION_NEWEST :: 300;
    RING_VERSION_OLDER :: 2;

    // The version the self test accepted.
    ring_version: u32;
    RING_SUBMISSION :: 128;
    RING_COMPLETION :: 256;
    RING_POLL_MILLISECONDS :: 50;

    // Whether this system has a usable IoRing: "" if so, else why not. The answer is kept.
    ring_probe :: () -> string {
        known := atomic_load(*ring_probe_state);

        if known == 1 return "";
        if known == 2 return ring_probe_reason;

        reason := ring_probe_run();

        ring_probe_reason = reason;
        atomic_store(*ring_probe_state, ifx reason == "" then cast(s32) 1 else cast(s32) 2);

        return reason;
    }

    ring_probe_run :: () -> string {
        kernelbase := win_load_library("kernelbase.dll");
        if kernelbase == 0 return "KernelBase.dll could not be loaded";

        api: Ring_Api;

        api.create = win_proc_address(kernelbase, "CreateIoRing");
        api.close = win_proc_address(kernelbase, "CloseIoRing");
        api.query = win_proc_address(kernelbase, "QueryIoRingCapabilities");
        api.submit = win_proc_address(kernelbase, "SubmitIoRing");
        api.pop = win_proc_address(kernelbase, "PopIoRingCompletion");
        api.set_event = win_proc_address(kernelbase, "SetIoRingCompletionEvent");
        api.read = win_proc_address(kernelbase, "BuildIoRingReadFile");
        api.write = win_proc_address(kernelbase, "BuildIoRingWriteFile");

        if !api.create || !api.close || !api.query || !api.submit || !api.pop || !api.set_event || !api.read || !api.write {
            return "this Windows has no IoRing for files (it needs Windows 11 22H2 or newer)";
        }

        capabilities: Io_Ring_Capabilities;
        query := cast(Ring_Query) api.query;

        if query(*capabilities) < 0 return "QueryIoRingCapabilities failed";
        if capabilities.max_version < RING_VERSION_OLDER return "this IoRing cannot write files (it needs Windows 11 22H2 or newer)";

        create := cast(Ring_Create) api.create;
        close := cast(Ring_Close) api.close;
        set_event := cast(Ring_Set_Event) api.set_event;

        reason := "the IoRing self test failed";

        for version: u32.[RING_VERSION_NEWEST, RING_VERSION_OLDER] {
            if version > capabilities.max_version continue;

            ring: s64;

            if create(version, Io_Ring_Flags.{}, 8, 16, *ring) < 0 || ring == 0 {
                reason = "CreateIoRing failed";

                continue;
            }

            event := win_create_event(null, 0, 0, null);

            if event == 0 {
                close(ring);

                return "could not create an event for the IoRing";
            }

            ok := false;

            if set_event(ring, event) >= 0 ok = ring_self_test(api, ring, event);

            close(ring);
            win_close_handle(event);

            if ok {
                ring_version = version;
                ring_api = api;

                return "";
            }
        }

        return reason;
    }

    // Reads the first two bytes of the running executable through `ring` and waits for the event,
    // then checks that the ring accepts a write entry.
    ring_self_test :: (api: Ring_Api, ring: s64, event: s64) -> bool {
        name: [520] u16;

        length := win_module_file_name(0, name.data, 520);

        if length == 0 || length >= 520 return false;

        file := win_create_file(name.data, GENERIC_READ, FILE_SHARE_ALL, null, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED, 0);
        if file == -1 return false;

        defer win_close_handle(file);

        read := cast(Ring_Read) api.read;
        submit := cast(Ring_Submit) api.submit;
        pop := cast(Ring_Pop) api.pop;

        buffer: [2] u8;

        if read(ring, Io_Ring_Ref.{ 0, cast(u64) file }, Io_Ring_Ref.{ 0, cast(u64) buffer.data }, 2, 0, 77, 0) < 0 return false;

        submitted: u32;

        if submit(ring, 0, 0, *submitted) < 0 return false;
        if win_wait_for_one(event, 3000) != WAIT_OBJECT_0 return false;

        cqe: Io_Ring_Cqe;

        if pop(ring, *cqe) != 0 return false;

        // "MZ", the first bytes of every executable.
        if !(cqe.user_data == 77 && cqe.result >= 0 && cqe.information == 2 && buffer[0] == 77 && buffer[1] == 90) return false;

        // Only built, never submitted: an old ring version refuses the entry.
        write := cast(Ring_Write) api.write;

        return write(ring, Io_Ring_Ref.{ 0, cast(u64) file }, Io_Ring_Ref.{ 0, cast(u64) buffer.data }, 1, 0, 0, 78, 0) >= 0;
    }

    // Makes this engine's ring. Returns why it cannot, or "" if files now use it.
    ring_open :: (state: *Iocp_State) -> string {
        reason := ring_probe();
        if reason != "" return reason;

        create := cast(Ring_Create) ring_api.create;
        set_event := cast(Ring_Set_Event) ring_api.set_event;
        close := cast(Ring_Close) ring_api.close;

        ring: s64;

        if create(ring_version, Io_Ring_Flags.{}, RING_SUBMISSION, RING_COMPLETION, *ring) < 0 || ring == 0 return "CreateIoRing failed";

        event := win_create_event(null, 0, 0, null);
        stop := win_create_event(null, 1, 0, null);

        if event == 0 || stop == 0 || set_event(ring, event) < 0 {
            close(ring);

            if event != 0 win_close_handle(event);
            if stop != 0 win_close_handle(stop);

            return "could not set up the IoRing's completion event";
        }

        state.ring = ring;
        state.ring_event = event;
        state.stop_event = stop;

        return "";
    }

    ring_close :: (state: *Iocp_State) {
        if state.ring_thread_started {
            win_set_event(state.stop_event);
            finish_thread(*state.ring_thread);

            state.ring_thread_started = false;
        }

        if state.ring == 0 return;

        close := cast(Ring_Close) ring_api.close;

        close(state.ring);

        win_close_handle(state.ring_event);
        win_close_handle(state.stop_event);

        state.ring = 0;
    }

    // Waits for the ring's completion event and tells the scheduler's port.
    ring_waiter :: (thread: *Thread.Thread) -> s64 {
        state := cast(*Iocp_State) thread.data;

        handles: [2] s64;
        handles[0] = state.ring_event;
        handles[1] = state.stop_event;

        while true {
            if win_wait_for_many(2, handles.data, 0, INFINITE) != WAIT_OBJECT_0 break;

            iocp_post(state.port, KEY_RING);
        }

        return 0;
    }

    ring_start_thread :: (state: *Iocp_State) {
        if state.ring_thread_started return;

        thread := *state.ring_thread;

        assert(Thread.thread_init(thread, ring_waiter), "Io: could not prepare the IoRing thread");
        thread.data = state;
        Thread.thread_start(thread);

        state.ring_thread_started = true;
    }

    // A file read or write in the ring. True: it failed at once. False: its completion will come.
    ring_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) -> bool {
        state := engine.iocp;

        ring_start_thread(state);

        op.uncancelable = true;

        if state.ring_inflight >= RING_SUBMISSION {
            op.next = null;

            if state.backlog_tail state.backlog_tail.next = op;
            else state.backlog_head = op;

            state.backlog_tail = op;

            return false;
        }

        if !ring_enqueue(state, op) return iocp_fail(op, E_INVAL);

        return false;
    }

    // Puts `op` in the submission queue. False if the ring refused it.
    ring_enqueue :: (state: *Iocp_State, op: *Op) -> bool {
        file := Io_Ring_Ref.{ 0, cast(u64) op.handle.fd };
        buffer := Io_Ring_Ref.{ 0, cast(u64) op.buffer };
        count := cast(u32) min(op.length, MAX_TRANSFER);
        offset := cast(u64) max(op.offset, 0);

        read := cast(Ring_Read) ring_api.read;
        write := cast(Ring_Write) ring_api.write;

        for 1..2 {
            result: s32;

            if op.kind == .READ result = read(state.ring, file, buffer, count, offset, cast(u64) op, 0);
            else result = write(state.ring, file, buffer, count, offset, 0, cast(u64) op, 0);

            if result >= 0 {
                state.ring_inflight += 1;
                state.ring_dirty = true;

                return true;
            }

            // The queue may be full of entries the kernel has not seen yet.
            ring_flush(state);
        }

        return false;
    }

    // Hands the queued entries to the kernel.
    ring_flush :: (state: *Iocp_State) {
        if !state.ring_dirty return;

        submit := cast(Ring_Submit) ring_api.submit;
        submitted: u32;

        result := submit(state.ring, 0, 0, *submitted);

        assert(result >= 0, "Io: SubmitIoRing failed (%)", result);

        state.ring_dirty = false;
    }

    // The Win32 error an HRESULT from the ring stands for (0 for success).
    ring_error :: (result: s32) -> s64 {
        if result >= 0 return 0;

        bits := cast(u32) result;

        // HRESULT_FROM_WIN32.
        if ((bits >> 16) & 0x1fff) == 7 return bits & 0xffff;

        // HRESULT_FROM_NT: the NTSTATUS with the "NT" facility bit set.
        if bits & 0x1000_0000 return win_status_error(cast,trunc(s32) (bits & 0xefff_ffff));

        return E_INVAL;
    }

    // Takes the completions that are in the ring, finishes their operations, and lets waiting ones in.
    ring_drain :: (scheduler: *Scheduler, engine: *Engine) {
        state := engine.iocp;
        pop := cast(Ring_Pop) ring_api.pop;

        while state.ring_inflight > 0 {
            cqe: Io_Ring_Cqe;

            if pop(state.ring, *cqe) != 0 break;
            if cqe.user_data == 0 continue;

            state.ring_inflight -= 1;

            op := cast(*Op) cqe.user_data;
            error := ring_error(cqe.result);
            result: s64;

            if error == 0 result = cast(s64) cqe.information;
            else if error == ERROR_HANDLE_EOF result = 0;
            else result = -error;

            finish_op(scheduler, op, result);
        }

        while state.backlog_head && state.ring_inflight < RING_SUBMISSION {
            op := state.backlog_head;

            state.backlog_head = op.next;
            if !state.backlog_head state.backlog_tail = null;

            op.next = null;

            if !ring_enqueue(state, op) finish_op(scheduler, op, -E_INVAL);
        }
    }

    // Blocks the thread until every operation in the ring has finished, for `close` of a file.
    ring_wait_idle :: (scheduler: *Scheduler, engine: *Engine) {
        state := engine.iocp;

        while state.ring_inflight > 0 || state.backlog_head {
            ring_flush(state);
            ring_drain(scheduler, engine);

            if state.ring_inflight > 0 win_sleep(1);
        }
    }
}
