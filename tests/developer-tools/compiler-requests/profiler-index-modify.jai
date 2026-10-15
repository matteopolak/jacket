#import "Basic";
#import "Compiler";
Print :: #import "Program_Print";

#run {
    w := compiler_create_workspace("Original indexed-body modification probe");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_file(tprint("%profiler-index-input.jai", #filepath), w);
    user_body, basic_body: bool;

    while true {
        event := compiler_wait_for_message();
        if !event break;

        if event.kind == .TYPECHECKED for tc: (cast(*Message_Typechecked) event).procedure_bodies {
            body := tc.expression;
            if !body || !body.header || !body.enclosing_load || !body.enclosing_load.enclosing_import continue;

            module := body.enclosing_load.enclosing_import;

            if body.header.name == "main" user_body = true;

            if module.module_type == .MAIN_PROGRAM && body.header.name == "indexed" {
                basic_body = true;
                print("modifying %.%\n", module.module_name, body.header.name);

                for node: body.block.statements {
                    builder: String_Builder;
                    success := Print.print_expression(*builder, node, skip_parens = true);
                    text := builder_to_string(*builder);

                    print("statement kind=%, printable=%, text=[%]\n", node.kind, success, text);
                    free(text);
                    free_buffers(*builder);
                }

                compiler_modify_procedure(w, body);
            }
        }

        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
    assert(user_body);
    assert(basic_body, "compiler omitted the original indexed body");
}

main :: () {}
