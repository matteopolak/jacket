// Whether `create_process` and `run_command` put quotes around the arguments they are given.
//
// `QUOTE_IF_NEEDED` (the default) quotes an argument that has spaces, tabs, quotes or is empty, so each argument arrives as one
// piece. `NEVER_QUOTE` passes the text as it is, for a caller who has built the command line. Only Windows builds a command line
// and so only Windows reads this; on Unix targets the arguments are handed to the program one by one and nothing is quoted.
Process_Argument_Quoting :: enum {
    QUOTE_IF_NEEDED :: 0;
    NEVER_QUOTE :: 1;
}

// How far a process has got, as `Process_Result.type`.
//
// - `UNSTARTED`: no process was created for this record (or it was released with `deinit`).
// - `STILL_RUNNING`: it has not finished yet.
// - `FAILED_TO_LAUNCH`: the program could not be started. `run_command` reports it; `create_process` returns `false` instead.
// - `EXITED`: it ended by itself or by being killed on Windows; `exit_code` has its code.
// - `SIGNALED`: a signal ended it (Unix targets only); `signal` has the number, `9` for `kill_process`.
Process_Result_Type :: enum s32 {
    UNSTARTED;
    STILL_RUNNING;
    FAILED_TO_LAUNCH;
    EXITED;
    SIGNALED;
}

// How a process ended: the `type`, the `exit_code` when it `.EXITED` (`-1` otherwise) and the `signal` when it was `.SIGNALED`.
// A zero `Process_Result` says `UNSTARTED`.
//
// ```jai,skip_on(wasm)
// result: Process_Result;
// assert(result.type == .UNSTARTED && result.exit_code == -1 && result.signal == 0);
// ```
Process_Result :: struct {
    type: Process_Result_Type;
    exit_code: s32 = -1;
    signal: s32;
}

// The system part of a `Process.Pipe`: the file descriptor of a pipe to or from the child, `-1` when there is none. Read it,
// but let the procedures of this module do the closing.
Pipe_Platform :: struct {
    handle: s32 = -1;
}

// The system part of a `Process`: the child's process id in `pid` (`0` before `create_process`), and on macOS a descriptor used
// for waiting that the module keeps.
Process_Platform :: struct {
    pid: P.pid_t;
    #if OS == .MACOS kevent_queue: s32 = -1;
}

// A child process and its pipes. Make an empty one (`process: Process;`), start it with `create_process`, and release it with `deinit`.
//
// - `input` is a pipe to the child's standard input, written with `write_to_process`. It is always there.
// - `output` and `error` are pipes from its standard output and error, read with `read_from_process` or `read_pipe`.
//   They exist only when `capture_and_return_output` was set; otherwise the child shares this program's own output and the
//   pipes start at end of file.
// - `eof` on a pipe turns `true` once the child has closed its end and everything has been read.
//
// The `Process` must stay where it is from `create_process` to `deinit`: the module finds its bookkeeping by the record's
// address, so copying or moving a running process breaks it. A zero `Process` has not started, and `get_process_result` says
// `UNSTARTED` for it.
//
// ```jai,skip_on(wasm)
// process: Process;
// ok, result := get_process_result(*process, 0);
// assert(ok && result.type == .UNSTARTED);
// ```
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
