// A variadic parameter's Code_Type_Instantiation has `.VARARGS` in its inst_flags, and other
// parameters do not; Check's @PrintLike test reads it. Compile-time only.
#import "Basic";
#import "Compiler";

SOURCE :: #string JAI
say :: (format: string, args: ..Any) {}
gather :: (first: $T, rest: ..T) {}
main :: () { say("x"); gather(1, 2); }
JAI

#run {
    w := compiler_create_workspace("varargs flag");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(SOURCE, w);
    seen := 0;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
        if message.kind != .TYPECHECKED continue;
        tc := cast(*Message_Typechecked) message;
        for tc.procedure_headers {
            h := it.expression;
            if h.name != "say" && h.name != "gather" continue;
            assert(h.arguments.count == 2, "% has % arguments", h.name, h.arguments.count);
            first := h.arguments[0];
            last := h.arguments[1];
            assert(first.type_inst == null || (first.type_inst.inst_flags & .VARARGS) == 0, "%'s first parameter is marked VARARGS", h.name);
            assert(last.type_inst != null && (last.type_inst.inst_flags & .VARARGS) != 0, "%'s variadic parameter lacks .VARARGS: %", h.name, last.type_inst.inst_flags);
            seen += 1;
        }
    }
    compiler_end_intercept(w);
    assert(seen >= 2, "saw % headers", seen);
    print("ok\n");
}
