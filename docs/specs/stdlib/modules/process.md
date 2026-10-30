# Process


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `break_command_into_strings :: (input: string) -> [..] string`
- `run_command :: (args: .. string, working_directory := "", capture_and_return_output := false, print_captured_output := false, timeout_ms := -1, arg_quoting := Process_Argument_Quoting.QUOTE_IF_NEEDED) -> (process_result: Process_Result, output_string := "", error_string := "", timeout_reached := false)`
- `Process :: struct`
- [OS_IS_UNIX] `Process_Platform :: struct`
- [OS == .WINDOWS] `Process_Platform :: struct`
- [OS_IS_UNIX] `Pipe_Platform :: struct`
- [OS == .WINDOWS] `Pipe_Platform :: struct`
- `Process_Argument_Quoting :: enum`
- `Process_Result_Type :: enum s32`
- `Process_Result :: struct`
- `get_quoted_command_string :: (args: [] string) -> string`
- `append_quoted :: (builder: *String_Builder, s: string)`
- [OS == .WINDOWS] `init_named_pipe_pair :: (pipe: *Process.Pipe, is_input: bool) -> success: bool, client_handle: HANDLE = INVALID_HANDLE_VALUE`
- [OS == .WINDOWS] `create_process :: (process: *Process, args: .. string, working_directory := "", capture_and_return_output := false, arg_quoting := Process_Argument_Quoting.QUOTE_IF_NEEDED, kill_process_if_parent_exits := true) -> success: bool`
- [OS == .WINDOWS] `deinit :: (process: *Process)`
- [OS == .WINDOWS] `deinit :: (pipe: *Process.Pipe)`
- [OS == .WINDOWS] `write_to_process :: (process: *Process, data: [] u8) -> success: bool, bytes_written: int`
- [OS == .WINDOWS] `read_from_process :: (process: *Process, output_buffer: [] u8, error_buffer: [] u8, timeout_ms := -1) -> success: bool, output_bytes: int, error_bytes: int`
- [OS == .WINDOWS] `kill_process :: (process: *Process) -> bool`
- [OS == .WINDOWS] `get_process_result :: (process: *Process, timeout_ms := -1) -> success: bool, Process_Result`
- [OS == .WINDOWS] `read_pipe :: (pipe: *Process.Pipe, buffer: [] u8) -> success: bool, bytes_read: int`
- `create_process :: (process: *Process, args: .. string, working_directory := "", capture_and_return_output := false, arg_quoting := Process_Argument_Quoting.QUOTE_IF_NEEDED, kill_process_if_parent_exits := true) -> success: bool`
- `deinit :: (process: *Process)`
- `deinit :: (pipe: *Process.Pipe)`
- `write_to_process :: (process: *Process, data: [] u8) -> success: bool, bytes_written: int`
- `read_from_process :: (process: *Process, output_buffer: [] u8, error_buffer: [] u8, timeout_ms := -1) -> success: bool, output_bytes: int, error_bytes: int`
- `kill_process :: (process: *Process) -> bool`
- `get_process_result :: (process: *Process, timeout_ms := -1) -> success: bool, Process_Result`
- `read_pipe :: (pipe: *Process.Pipe, buffer: [] u8) -> success: bool, bytes_read: int`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Pipe_Platform`

Windows:

```jai
Pipe_Platform :: struct {
    handle: HANDLE = INVALID_HANDLE_VALUE;
    overlapped: *OVERLAPPED;
}
```

The Windows variant stores a native pipe handle initialized to the invalid-handle sentinel and a pointer to native overlapped-I/O state. `HANDLE`, `OVERLAPPED`, and `INVALID_HANDLE_VALUE` are supplied by the Windows binding, not declared by Process. The pointer field is part of the public record shape; its pointed-to storage is managed by the pipe operations.

Unix:

```jai
Pipe_Platform :: struct {
        handle: s32 = -1;
    }
```
### `Process`

```jai
Process :: struct {
    input:  Pipe;
    output: Pipe;
    error:  Pipe;

    using platform_specific: Process_Platform;

    Pipe :: struct {
        using platform_specific: Pipe_Platform;

        eof: bool;
    }
}
```
### `Process_Argument_Quoting`

```jai
Process_Argument_Quoting :: enum {
    QUOTE_IF_NEEDED :: 0;
    NEVER_QUOTE     :: 1;
}
```
### `Process_Platform`

Windows:

```jai
Process_Platform :: struct {
    job_handle: HANDLE = INVALID_HANDLE_VALUE;
    info: PROCESS_INFORMATION;
}
```

`PROCESS_INFORMATION` is the Windows binding's native record. `job_handle` is initialized to `INVALID_HANDLE_VALUE`. These fields are publicly visible because they are embedded with `using` in `Process`.

Unix:

```jai
Process_Platform :: struct {
        pid: pid_t;

        #if OS == .MACOS {
            kevent_queue: s32 = -1;
        }
    }
```
### `Process_Result`

```jai
Process_Result :: struct {
    type: Process_Result_Type;
    exit_code: s32 = -1;
    signal: s32;
}
```
### `Process_Result_Type`

```jai
Process_Result_Type :: enum s32 {
    UNSTARTED;
    STILL_RUNNING;
    FAILED_TO_LAUNCH;
    EXITED;
    SIGNALED;
}
```
## Behavioral contract

Commands are launched from an argument array, avoiding shell concatenation. `run_command` can capture stdout/stderr, optionally echo captured bytes, set a working directory, and enforce a timeout; it returns process status, captured streams, and whether timeout occurred. `break_command_into_strings` is intentionally limited: quotes are recognized only at argument start, backslash-escaped quotes are not handled, and unmatched quotes are not reported cleanly. Relative executable resolution differs by OS: Windows resolves relative paths against the parent's working directory; Unix resolves against the child working directory, while bare executable names are searched only through `PATH`. Resolve executable paths explicitly when portability matters. OS implementations are selected for Windows and supported Unix-like systems; other targets have separate/conditional implementations. Depends on OS process, pipe, file, and timing APIs.

### Operation-level behavior

| Operation group | Observable behavior |
| --- | --- |
| `run_command` | Pass each argument separately; no shell expansion is implied. Without capture it waits for the child result. With capture it accumulates stdout and stderr independently until both pipes reach EOF, optionally echoing each chunk to the corresponding standard stream. A nonnegative timeout can kill the child and sets `timeout_reached`; `-1` means no timeout. |
| `create_process` / `deinit` | `create_process` fills the process handle and reports launch success. Call `deinit` to close/reap associated resources. On POSIX it first waits, without limit, for the child to finish (so it is reaped); on Windows it does not wait. `deinit(*process.input)` closes only the child's standard input on both targets, so a child reading it sees the end of the file. `kill_process` terminates the child; it does not replace result collection. |
| `get_process_result` | A not-yet-created process reports `UNSTARTED`. A live process may report `STILL_RUNNING` when a finite wait expires; completion reports `EXITED` with an exit code or `SIGNALED` with a signal on POSIX. Windows reports exit status through `EXITED`. |
| `write_to_process` / `read_from_process` | Writes return success plus bytes written. Reads fill separate stdout/stderr buffers and return per-stream byte counts; zero bytes can mean no data arrived before timeout or that a stream has reached EOF, which is tracked in each pipe's `eof` field. Check those flags and the success result to distinguish outcomes. |
| `break_command_into_strings` | The result array uses temporary storage. It splits on spaces and recognizes quotes only when they begin an argument; it has no escape syntax or robust malformed-quote error result. Use an explicit argument array when argument boundaries matter. |

Timeout precision and pipe scheduling are OS-dependent. A read error can cause `run_command` to log and kill the child; captured strings may then contain only bytes read before the failure.


Platform coverage note: Nintendo, PS5 branches exist, but their declarations are not specified here; exports for those targets are unspecified.
