using #import "Basic";
C :: #import "Compiler";

#run {
    workspace := C.compiler_create_workspace("Deferred child");

    defer C.compiler_destroy_workspace(workspace);
    options := C.get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    C.set_build_options(options, workspace);
    C.compiler_begin_intercept(workspace);
    defer C.compiler_end_intercept(workspace);
    filename := sprint("%deferred-invariants.jai", #filepath);

    defer free(filename);
    C.add_build_file(filename, workspace);

    while true {
        message := C.compiler_wait_for_message();

        assert(message != null && message.workspace == workspace);

        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    print("child dependency complete\n");
}

main :: () {}
