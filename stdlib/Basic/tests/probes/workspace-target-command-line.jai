// Original general Compiler API reproducer, independent of Default_Metaprogram.
C :: #import "Compiler";
using #import "Basic";

#run,stallable {
    current := C.get_build_options();
    current.output_type = .NO_OUTPUT;
    C.set_build_options(current);
    w := C.compiler_create_workspace("command line forwarding");
    options := C.get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.compile_time_command_line = .["first", "-literal"];
    C.set_build_options(options, w);
    C.compiler_begin_intercept(w);
    C.add_build_string("using #import \"Basic\"; C :: #import \"Compiler\"; #run { args := C.get_toplevel_command_line(); assert(args.count == 2); assert(args[0] == \"first\"); assert(args[1] == \"-literal\"); }", w);

    while true {
        message := C.compiler_wait_for_message();

        assert(message != null);

        if message.kind == .COMPLETE {
            complete := cast(*C.Message_Complete) message;
            assert(complete.error_code == .NONE);
            break;
        }
    }

    C.compiler_end_intercept(w);
}
