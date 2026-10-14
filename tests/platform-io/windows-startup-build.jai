#import "Basic";
C :: #import "Compiler";

#run {
    workspace := C.compiler_create_workspace("Windows entry contract");
    options := C.get_build_options(workspace);
    options.output_path = OUTPUT_PATH;
    options.output_executable_name = "startup-crash";
    options.runtime_support_definitions = .ENTRY_POINT_AND_INIT;
    #if OPTIMIZED C.set_optimization(*options, .OPTIMIZED);
    else C.set_optimization(*options, .DEBUG);
    #if BACKTRACE_ENABLED options.backtrace_on_crash = .ON;
    else options.backtrace_on_crash = .OFF;
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
