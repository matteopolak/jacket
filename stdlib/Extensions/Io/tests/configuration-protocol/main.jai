// NO_OUTPUT child protocol: MAIN owns Basic PROGRAM settings, modules inherit.
#import "Basic";
C :: #import "Compiler";

#run {
    arguments := C.get_build_options().compile_time_command_line;

    assert(arguments.count == 0 || arguments.count == 1);
    enabled := true;

    if arguments.count == 1 {
        assert(arguments[0] == "enabled" || arguments[0] == "disabled");
        enabled = arguments[0] == "enabled";
    }

    workspace := C.compiler_create_workspace("io-configuration-protocol");
    options := C.get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    options.text_output_flags = 0;
    options.import_path = .[tprint("%modules", #filepath)];
    C.set_build_options(options, workspace);
    C.compiler_begin_intercept(workspace);
    C.add_build_string(tprint("#import \"Basic\"()(ENABLE_ASSERT = %);", ifx enabled then "true" else "false"), workspace);
    C.add_build_string(tprint("EXPECTED_ASSERTIONS :: %;", ifx enabled then "true" else "false"), workspace);
    C.add_build_file(tprint("%child.jai", #filepath), workspace);

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

main :: () {
    print("parent protocol complete\n");
}
