// The system part of a `Process.Pipe`: the pipe handle (`INVALID_HANDLE_VALUE` when there is none) and the overlapped record its
// reads and writes use. Read it, but let the procedures of this module do the closing.
Pipe_Platform :: struct {
    handle: W.HANDLE = W.INVALID_HANDLE_VALUE;
    overlapped: *W.OVERLAPPED;
}

// The system part of a `Process`: the job object that holds the child (so it can be killed with its descendants), and the
// `PROCESS_INFORMATION` with its handles and ids.
Process_Platform :: struct {
    job_handle: W.HANDLE = W.INVALID_HANDLE_VALUE;
    info: W.PROCESS_INFORMATION;
}

// A child process and its pipes. Make an empty one (`process: Process;`), start it with `create_process`, and release it with `deinit`.
//
// - `input` is a pipe to the child's standard input, written with `write_to_process`. It is always there.
// - `output` and `error` are pipes from its standard output and error, read with `read_from_process` or `read_pipe`.
//   They exist only when `capture_and_return_output` was set; otherwise the child shares this program's own output.
// - `eof` on a pipe turns `true` once the child has closed its end and everything has been read.
//
// A zero `Process` has not started, and `get_process_result` says `UNSTARTED` for it.
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

// How far a process has got, as `Process_Result.type`: `UNSTARTED` (nothing was created), `STILL_RUNNING`, `FAILED_TO_LAUNCH`
// (`run_command` could not start the program) and `EXITED` with its `exit_code`. `SIGNALED` is never produced on Windows; a
// process that was killed is `EXITED`.
Process_Result_Type :: enum s32 {
    UNSTARTED :: 0;
    STILL_RUNNING :: 1;
    FAILED_TO_LAUNCH :: 2;
    EXITED :: 3;
    SIGNALED :: 4;
}

// How a process ended: the `type`, the `exit_code` when it `.EXITED` (`-1` otherwise) and the `signal`, which is always `0` on Windows.
Process_Result :: struct {
    type: Process_Result_Type;
    exit_code: s32 = -1;
    signal: s32;
}

// Whether `create_process` and `run_command` put quotes around the arguments they are given. Windows starts a program from one
// command-line string, so the arguments are joined into it.
//
// `QUOTE_IF_NEEDED` (the default) quotes an argument that has spaces, tabs, quotes or is empty, with the escaping that most programs
// (those using the C runtime's rules) undo. `NEVER_QUOTE` joins the arguments with spaces as they are, for a caller who has
// built the command line, or whose program has its own rules.
Process_Argument_Quoting :: enum {
    QUOTE_IF_NEEDED :: 0;
    NEVER_QUOTE :: 1;
}

#scope_file
W :: #import "internal/bindings/windows-api";
