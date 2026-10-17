// Original default-option reproducer; deliberately does not assign crash policy.
#import "Basic";
C :: #import "Compiler";

#run {
    workspace := C.compiler_create_workspace("default Windows startup");
    options := C.get_build_options(workspace);

    assert(options.backtrace_on_crash == .ON);
    assert(options.runtime_support_definitions == .AUTO);
    options.output_path = OUTPUT_PATH;
    options.output_executable_name = "startup-crash";
    #if OPTIMIZED C.set_optimization(*options, .OPTIMIZED);
    else C.set_optimization(*options, .DEBUG);
    C.set_build_options(options, workspace);
    C.compiler_begin_intercept(workspace);
    C.add_build_file(CHILD_SOURCE, workspace);

    while true {
        message := C.compiler_wait_for_message();
        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    C.compiler_end_intercept(workspace);
}

main :: () {}
