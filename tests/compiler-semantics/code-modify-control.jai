#import "Basic";
#import "Compiler";

#run {
    w := compiler_create_workspace("Original control-flow body modification");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_string("main :: () { total:int; for i:0..3 {total+=i;} if total == {case 6; total=9; case; total=0;} }", w);
    changed: bool;

    while true {
        message := compiler_wait_for_message();

        assert(message != null);

        if message.kind == .TYPECHECKED for checked: (cast(*Message_Typechecked) message).procedure_bodies {
            body := checked.expression;
            if !changed && body.header.name == "main" {
                assert(body.block.statements[1].kind == .FOR);
                loop := cast(*Code_For) body.block.statements[1];
                bound := cast(*Code_Literal) loop.iteration_expression_right;

                assert(bound._s64 == 3);
                bound._s64 = 4;
                compiler_modify_procedure(w, body);
                changed = true;
            }
        }

        if message.kind == .ERROR assert(false, "modified control-flow body must check");

        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
    assert(changed);
}

main :: () {}
