#import "Basic";
#import "Compiler";
build :: () {
    w := compiler_create_workspace();
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string("#import \"Basic\";\nwork :: () { x := 1; while true { x += 1; } }\n#run work();\n", w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
}
#run build();
main :: () {}
