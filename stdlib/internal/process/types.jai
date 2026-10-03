Process_Argument_Quoting :: enum {
    QUOTE_IF_NEEDED :: 0;
    NEVER_QUOTE :: 1;
}

Process_Result_Type :: enum s32 {
    UNSTARTED;
    STILL_RUNNING;
    FAILED_TO_LAUNCH;
    EXITED;
    SIGNALED;
}

Process_Result :: struct {
    type: Process_Result_Type;
    exit_code: s32 = -1;
    signal: s32;
}

Pipe_Platform :: struct {
    handle: s32 = -1;
}

Process_Platform :: struct {
    pid: P.pid_t;
    #if OS == .MACOS kevent_queue: s32 = -1;
}

Process :: struct {
    input: Pipe;
    output: Pipe;
    error: Pipe;
    using platform_specific: Process_Platform;
    Pipe :: struct {
        using platform_specific: Pipe_Platform;
        eof: bool;
    }
}

#scope_module
Result_Packet :: struct {
    status: s32;
    error: s32;
}

Process_State :: struct {
    next: *Process_State;
    process: *Process;
    allocator: Allocator;
    supervisor: P.pid_t;
    control: s32 = -1;
    result_fd: s32 = -1;
    packet: Result_Packet;
    received: s64;
    ready: bool;
    failed: bool;
    result: Process_Result;
}
