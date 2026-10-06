// A procedure's header reported to a metaprogram (TYPECHECKED) and the same procedure
// resolved from the workspace's own compile-time code (`#modify`) are separate records:
// only the latter's types are real descriptors for `get_type` (or_return's shape).
// rules: records.11
#import "Basic";
#import "Compiler";

TARGET :: #string END
#import "Basic";
Compiler :: #import "Compiler";
Callback :: #type (s: string) -> string;
echo :: (s: string) -> string { return s; }
pair :: (a: int) -> Callback, bool { return echo, true; }
returns_two :: ($code: Code) -> bool #expand
#modify {
    root := Compiler.compiler_get_nodes(code);
    header := cast(*Compiler.Code_Procedure_Header) (cast(*Compiler.Code_Procedure_Call) root).resolved_procedure_expression;
    if Compiler.get_type(header.returns[0].type_inst.result) != Callback  return false, "not Callback";
    if Compiler.get_type(header.returns[1].type_inst.result) != bool  return false, "not bool";
    return true;
} { return true; }
main :: () { assert(returns_two(pair(1))); }
END

#run {
    w := compiler_create_workspace("own types");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }
    compiler_end_intercept(w);
    set_build_options_dc(.{ do_output = false });
}

main :: () {}
