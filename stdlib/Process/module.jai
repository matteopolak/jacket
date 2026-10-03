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
