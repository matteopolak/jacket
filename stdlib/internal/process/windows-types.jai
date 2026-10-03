// Canonical public records; native ownership lives in the platform engine.
Pipe_Platform :: struct {
    handle: W.HANDLE = W.INVALID_HANDLE_VALUE;
    overlapped: *W.OVERLAPPED;
}

Process_Platform :: struct {
    job_handle: W.HANDLE = W.INVALID_HANDLE_VALUE;
    info: W.PROCESS_INFORMATION;
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

Process_Result_Type :: enum s32 {
    UNSTARTED :: 0;
    STILL_RUNNING :: 1;
    FAILED_TO_LAUNCH :: 2;
    EXITED :: 3;
    SIGNALED :: 4;
}

Process_Result :: struct {
    type: Process_Result_Type;
    exit_code: s32 = -1;
    signal: s32;
}

Process_Argument_Quoting :: enum {
    QUOTE_IF_NEEDED :: 0;
    NEVER_QUOTE :: 1;
}

#scope_file
W :: #import "Windows";
