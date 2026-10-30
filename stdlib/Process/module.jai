// Starting programs, feeding them input and collecting their output and exit status.
//
// `run_command` is the short way: it runs a program to its end, optionally with a time limit and with its output
// captured, and returns how it ended. `create_process` starts one that you then drive yourself with
// `write_to_process`, `read_from_process`, `get_process_result` and `kill_process`, and release with `deinit`.
// No shell is involved: the arguments go to the program as they are, so use `/bin/sh -c` when you want shell syntax.
// `break_command_into_strings`, `get_quoted_command_string` and `append_quoted` handle command-line text.
//
// Unix targets (Linux, macOS) start the program with `fork` and `execvp`, and report a signal-killed child as
// `SIGNALED`; Windows builds one command line, and reports everything as `EXITED`. A child started with
// `kill_process_if_parent_exits` (the default) does not outlive this program. There is no wasm backend.
//
// ```jai,only(linux, macos)
// result, out, err, timed_out := run_command("/bin/sh", "-c", "printf hello", capture_and_return_output = true, timeout_ms = 5000);
// assert(!timed_out && result.type == .EXITED && result.exit_code == 0);
// assert(out == "hello");
// free(out);
// free(err);
// ```

#assert OS == .WINDOWS || OS == .MACOS || OS == .LINUX || OS == .ANDROID;
break_command_into_strings :: Impl.break_command_into_strings;
run_command :: Impl.run_command;
Process :: Impl.Process;
Process_Platform :: Impl.Process_Platform;
Pipe_Platform :: Impl.Pipe_Platform;
Process_Argument_Quoting :: Impl.Process_Argument_Quoting;
Process_Result_Type :: Impl.Process_Result_Type;
Process_Result :: Impl.Process_Result;
get_quoted_command_string :: Impl.get_quoted_command_string;
append_quoted :: Impl.append_quoted;
create_process :: Impl.create_process;
deinit :: Impl.deinit;
write_to_process :: Impl.write_to_process;
read_from_process :: Impl.read_from_process;
kill_process :: Impl.kill_process;
get_process_result :: Impl.get_process_result;
read_pipe :: Impl.read_pipe;
#if OS == .WINDOWS init_named_pipe_pair :: Impl.init_named_pipe_pair;

#scope_file
#if OS == .WINDOWS Impl :: #import "internal/process/windows";
else Impl :: #import "internal/process";
