// Valid control for the retired module-supplied Basic program setting.
#import "Basic";
C :: #import "Compiler";

#run {
    workspace := C.compiler_create_workspace("root-owned-basic-setting");
    options := C.get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    options.text_output_flags = 0;
    C.set_build_options(options, workspace);
    C.compiler_begin_intercept(workspace);
    C.add_build_string("using B :: #import \"Basic\"()(ENABLE_ASSERT=false); #assert !#exists(B.String_Builder.Buffer.ensured_count); main :: () {}", workspace);

    while true {
        message := C.compiler_wait_for_message();

        assert(message.kind != .ERROR);

        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == 0);
            break;
        }
    }

    C.compiler_end_intercept(workspace);
    // The child's setting must not change this parent program's Basic layout.
    assert(#exists(String_Builder.Buffer.ensured_count));
}

main :: () {}
