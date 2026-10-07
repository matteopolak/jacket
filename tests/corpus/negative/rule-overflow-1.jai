// Build_Options.arithmetic_overflow_check = .FATAL also checks the compound operators: `*=` that
// does not fit its u8 operand stops the program run in the new workspace.
// rules: overflow.1 overflow.3 overflow.4
#import "Compiler";

SOURCE :: #string JAI
work :: () {
    x: u8 = 200;
    x *= 2;
}
#run work();
JAI

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
