// rules: compiler.23 compiler.24
// A FAILED_IMPORT answered with a file that does not exist cannot be satisfied: the workspace
// must end with an error instead of asking again for ever.
#import "Basic";
#import "Compiler";

SOURCE :: #string JAI
P :: #import "Missing_Mod";
main :: () {}
JAI

checks :: () {
    asked := 0;
    error_code := -1;
    w := compiler_create_workspace("probe");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(SOURCE, w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == {
            case .FAILED_IMPORT;
                asked += 1;
                provide_import(w, cast(*Message_Failed_Import) message, .PATH_TO_FILE, "/no/such/dir/Missing_Mod.jai");
            case .COMPLETE;
                error_code = xx (cast(*Message_Complete) message).error_code;
        }
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
    assert(error_code != 0);
    // Asked again once, for the offer; the same offer a second time changes nothing.
    assert(asked >= 1 && asked <= 2);
    print("ok\n");
}

#run checks();

main :: () {}
