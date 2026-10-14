#import "Basic";
C :: #import "Compiler";

#run {
    w := C.compiler_create_workspace("message contract");
    options := C.get_build_options(w); options.output_type = .NO_OUTPUT;
    C.set_build_options(options, w);
    C.add_build_string("value :: 47; main :: () {}", w);
    C.compiler_begin_intercept(w);
    imports := 0; files := 0; declarations := 0; complete := false;
    while !complete {
        message := C.compiler_wait_for_message();
        assert(message != null && message.workspace == w);
        if message.kind == .IMPORT imports += 1;
        if message.kind == .FILE files += 1;
        if message.kind == .TYPECHECKED {
            typed := cast(*C.Message_Typechecked) message;
            for item: typed.declarations {
                if item.expression.name == "value" {
                    value := cast(*C.Code_Literal) item.expression.expression;
                    assert(value._s64 == 47);
                    declarations += 1;
                }
            }
        }
        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == .NONE);
            complete = true;
        }
    }
    assert(imports > 0 && files > 0 && declarations == 1);
    C.compiler_end_intercept(w);
}

main :: () { print("ok\n"); }
