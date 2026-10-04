// Build_Options.arithmetic_overflow_check = .FATAL: a multiplication stops the program that is run in the new workspace.
#import "Compiler";

SOURCE :: #string END
work :: () {
    x: s32 = 100000; z := x * x;
}
#run work();
END

#run {
    w := compiler_create_workspace();
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.arithmetic_overflow_check = .FATAL;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(SOURCE, w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
}
