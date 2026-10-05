// What checkers and transpilers read from TYPECHECKED messages (MetaThreadSafe,
// Jai-Shader-Transpiler): callee notes through resolved calls, one header record per
// procedure (so a body reported later reaches headers seen earlier), parameter usings,
// bodies of noted procedures nothing calls, and initializer types.
#import "Basic";
#import "Compiler";

TARGET :: #string END
#import "Basic";
In :: struct { a: int; }
entry :: () { checked(); } @entry
checked :: () -> int { x := 2; return x; } @checked
shader :: (using input: In) -> int { y := a + 1; return y; } @shader
main :: () { entry(); }
END

#run {
    w := compiler_create_workspace("notes");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    headers: [..] *Code_Procedure_Header;
    saw_shader_body := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE  break;
        if message.kind != .TYPECHECKED  continue;
        typechecked := cast(*Message_Typechecked) message;
        for typechecked.procedure_headers  array_add(*headers, it.expression);
        for typechecked.procedure_bodies {
            body := it.expression;
            if body.header.name == "shader" {
                saw_shader_body = true;
                assert(body.header.parameter_usings.count == 1);
                decl := cast(*Code_Declaration) body.block.statements[0];
                assert(decl.expression.type == type_info(int));
            }
            if body.header.name == "entry" {
                call := cast(*Code_Procedure_Call) body.block.statements[0];
                callee := cast(*Code_Procedure_Header) call.resolved_procedure_expression;
                assert(callee.notes.count == 1 && callee.notes[0].text == "checked");
                // The same header the earlier message reported.
                found := false;
                for headers  if it == callee  found = true;
                assert(found);
            }
        }
    }
    compiler_end_intercept(w);
    assert(saw_shader_body);
    for headers  if it.name == "checked"  assert(it.body_or_null != null);
    set_build_options_dc(.{do_output = false});
}

main :: () {}
