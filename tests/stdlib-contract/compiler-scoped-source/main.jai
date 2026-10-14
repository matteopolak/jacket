#import "Basic";
C :: #import "Compiler";

#run {
    inside :: 31;
    C.add_build_string("#load \"scoped-piece.jai\"; #assert(inside == 31); #assert(scoped_piece_value == 23);", -1, #code 0);
    w := C.compiler_create_workspace("file scope contract");
    options := C.get_build_options(w); options.output_type = .NO_OUTPUT;
    C.set_build_options(options, w);
    C.add_build_string("#scope_file secret :: 47; #scope_export main :: () {}", w);
    C.compiler_begin_intercept(w);
    inserted := false;
    while true {
        message := C.compiler_wait_for_message();
        assert(message != null);
        if message.kind == .FILE && (cast(*C.Message_File) message).from_a_string && !inserted {
            C.add_build_string("#assert(secret == 47);", w, message);
            inserted = true;
        }
        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }
    assert(inserted);
    C.compiler_end_intercept(w);
}

main :: () { print("ok\n"); }
