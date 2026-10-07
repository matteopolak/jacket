// A cast does not type its operand: in `cast(float32) (0 - w)` the `0` is a u16, so with
// arithmetic_overflow_check = .FATAL the u16 subtraction underflows before the conversion.
#import "Compiler";

SOURCE :: #string JAI
work :: () {
    w: u16 = 15; f := cast(float32) (0 - w);
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
