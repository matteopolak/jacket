// A program that starts a second compilation in a workspace of its own, then loads files itself.
#import "Basic";
#import "Compiler";
#load "helper.jai";

REPORT :: #run inspect();

inspect :: () -> string {
    workspace := compiler_create_workspace("inspection");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, workspace);

    compiler_begin_intercept(workspace);
    add_build_string("answer :: () -> int { return 42; }", workspace);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(workspace);
    return helper_name();
}

main :: () {
    print("%\n", REPORT);
}
