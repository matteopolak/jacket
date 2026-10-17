// Exact reviewed Compiler-API configuration, avoiding reference CLI assumptions.
B :: #import "Basic";
C :: #import "Compiler";

TARGET_SOURCE :: #string END
#run,host {
    B :: #import "Basic";
    first := B.current_time_monotonic();
    second := B.current_time_monotonic();
    B.assert(B.compare_apollo_times(second, first) >= 0);
    B.print("isolated-host clock ok\n");
}
main :: () {}
END

#run,stallable {
    parent := C.get_build_options();
    if parent.compile_time_command_line.count < 1 || parent.compile_time_command_line.count > 2 {
        C.compiler_report("Supply a private object-output directory and optional independent target source.");
        return;
    }

    C.set_build_options_dc(.{ do_output = false });
    child := C.compiler_create_workspace("isolated host clock reduction");
    options := C.get_build_options(child);
    options.os_target = .LINUX;
    options.cpu_target = .X64;
    options.output_type = .OBJECT_FILE;
    options.output_path = parent.compile_time_command_line[0];
    options.output_executable_name = "host-clock-target";
    C.set_build_options(options, child);
    C.compiler_begin_intercept(child);
    if parent.compile_time_command_line.count == 2 C.add_build_file(parent.compile_time_command_line[1], child);
    else C.add_build_string(TARGET_SOURCE, child);

    while true {
        message := C.compiler_wait_for_message();
        if !message {
            C.compiler_report("Child workspace ended without completion.");
            break;
        }

        if message.workspace == child && message.kind == .COMPLETE {
            complete := cast(*C.Message_Complete) message;
            if complete.error_code != .NONE C.compiler_report("Isolated host clock workspace failed.");
            break;
        }
    }

    C.compiler_end_intercept(child);
}

main :: () {}
