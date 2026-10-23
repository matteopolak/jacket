// A declaration inside an `#if` of a file added after the first one is visible to the code of the
// files before it, not only to compile-time code.
#import "Basic";
#import "Compiler";

TARGET :: #string JAI
#import "Basic";
main :: () { print("%\n", LATE_VALUE + late_procedure()); }
JAI

#run {
    w := compiler_create_workspace("target");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    add_build_file(tprint("%late-file-conditional/second.jai", #filepath), w);

    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
}
