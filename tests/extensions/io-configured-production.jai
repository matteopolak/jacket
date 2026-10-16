// Configure the child MAIN's canonical Basic before Io/Thread imports it.
#import "Basic";
C :: #import "Compiler";

#run {
    arguments := C.get_build_options().compile_time_command_line;
    source := tprint("%io-configured-workers.jai", #filepath);
    enabled := false;
    workspace := C.compiler_create_workspace("configured-production-io");
    options := C.get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    options.text_output_flags = 0;

    if arguments.count > 0 {
        assert(arguments.count == 5);
        assert(arguments[0] == "enabled" || arguments[0] == "disabled");
        assert(arguments[3] == "O0" || arguments[3] == "O2" || arguments[3] == "interp");
        assert(arguments[4] == "arm64" || arguments[4] == "x64");
        enabled = arguments[0] == "enabled";
        source = arguments[1];
        if arguments[3] != "interp" options.output_type = .EXECUTABLE;

        options.output_executable_name = arguments[2];
        options.cpu_target = ifx arguments[4] == "x64" then .X64 else .ARM64;
        C.set_optimization(*options, ifx arguments[3] == "O2" then .OPTIMIZED else .DEBUG);
    }

    C.set_build_options(options, workspace);

    C.compiler_begin_intercept(workspace);
    C.add_build_string(tprint("#import \"Basic\"()(ENABLE_ASSERT = %);", ifx enabled then "true" else "false"), workspace);
    C.add_build_file(source, workspace);
    if arguments.count > 0 && arguments[3] == "interp" C.add_build_string("#run main();", workspace);

    while true {
        message := C.compiler_wait_for_message();

        assert(message.kind != .ERROR);

        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == 0);
            break;
        }
    }

    C.compiler_end_intercept(workspace);
}

main :: () {}
