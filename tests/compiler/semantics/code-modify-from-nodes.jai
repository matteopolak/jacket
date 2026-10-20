// A procedure body can be edited with statements taken from `compiler_get_nodes` of a `#code`
// block, and a header that belongs to another workspace is not made live in this one.
#import "Basic";
#import "Compiler";

#run {
    w := compiler_create_workspace("Modify with nodes");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_string("counter: int; main :: () { counter = 1; }", w);
    changed := false;
    foreign_header: *Code_Procedure_Header;

    while true {
        message := compiler_wait_for_message();

        if message.kind == .TYPECHECKED {
            typechecked := cast(*Message_Typechecked) message;

            for typechecked.declarations {
                decl := it.expression;

                if decl.expression && decl.expression.kind == .PROCEDURE_HEADER && !foreign_header {
                    foreign_header = cast(*Code_Procedure_Header) decl.expression;
                }
            }

            for checked: typechecked.procedure_bodies {
                body := checked.expression;

                if !changed && body.header.name == "main" {
                    extra := compiler_get_nodes(#code {
                        counter += 41;
                    });

                    assert(extra != null);
                    fresh := cast(*Code_Block) extra;

                    body.block.statements[0] = fresh.statements[0];
                    compiler_modify_procedure(w, body);
                    changed = true;
                }
            }
        }

        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
    assert(changed);
    assert(foreign_header != null);
    other := compiler_create_workspace("Other");

    compiler_make_procedure_live(other, foreign_header);
}

main :: () {}
