// rules: ws.13
// A workspace that fails to compile completes with error code COMPILATION_FAILED (1),
// and the failure fails the whole compile.
#import "Compiler";

#run {
    w := compiler_create_workspace("broken child");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string("main :: () { x: int = \"text\"; }", w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            if cast(s64) (cast(*Message_Complete) message).error_code == 1 {
                compiler_report("rule-ws-13: the child completed with code 1", mode = .WARNING);
            }
            break;
        }
    }
    compiler_end_intercept(w);
}

main :: () {}
