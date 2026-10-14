#import "Basic";
C :: #import "Compiler";

#run {
    w := C.compiler_create_workspace("modify contract");
    options := C.get_build_options(w); options.output_type = .NO_OUTPUT;
    C.set_build_options(options, w);
    C.add_build_string("answer :: () -> int { return 13; } main :: () { value := answer(); }", w);
    C.compiler_begin_intercept(w);
    changed := false;
    while true {
        message := C.compiler_wait_for_message();
        assert(message != null);
        if message.kind == .TYPECHECKED {
            typed := cast(*C.Message_Typechecked) message;
            for item: typed.procedure_bodies {
                body := item.expression;
                if body.header.name == "answer" && !changed {
                    ret := cast(*C.Code_Return) body.block.statements[0];
                    assert(ret.kind == .RETURN);
                    literal := cast(*C.Code_Literal) ret.arguments_unsorted[0].expression;
                    assert(literal._s64 == 13);
                    literal._s64 = 29;
                    C.compiler_modify_procedure(w, body);
                    C.add_build_string("#assert(answer() == 29);", w);
                    changed = true;
                }
            }
        }
        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }
    assert(changed);
    C.compiler_end_intercept(w);
}

main :: () { print("ok\n"); }
