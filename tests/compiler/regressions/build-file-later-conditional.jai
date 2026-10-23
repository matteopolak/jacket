// Code that runs at compile time, from program text added first, can use a declaration that an
// `#if` in a file added later makes, once the constant its condition reads is defined.
#import "Basic";
#import "Compiler";
#import "String";

PROGRAM :: #string JAI
#import "Basic";
#insert #run generate_limit();
#insert #run generate_wide();
#run assert(size_of(Wide) == 1);
main :: () {}
JAI

build :: () {
    w := compiler_create_workspace("later file");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(PROGRAM, w);
    add_build_file(tprint("%modules/Late_Conditional.jai", path_strip_filename(#file)), w);

    while true {
        message := compiler_wait_for_message();
        if message.kind != .COMPLETE continue;

        complete := cast(*Message_Complete) message;

        assert(complete.error_code == .NONE, "the child workspace compiles: %", complete.error_code);
        break;
    }

    compiler_end_intercept(w);
    compiler_destroy_workspace(w);
}

#run build();

main :: () {
    print("ok\n");
}
