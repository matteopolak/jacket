#import "Basic";
#import "Compiler";
Print :: #import "Program_Print";

#run {
    w := compiler_create_workspace("Original growable local-array modification probe");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;

    #if #exists(PROBE_EXECUTABLE) {
        options.output_type = .EXECUTABLE;
        options.output_executable_name = PROBE_EXECUTABLE;
        // The crash archive prerequisite is independently owned; retain all assertions.
        options.backtrace_on_crash = .OFF;
        #if #exists(PROBE_OPTIMIZED) options.llvm_options.bitcode_optimization_setting = .O2;
        else options.llvm_options.bitcode_optimization_setting = .O0;
    }

    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_file(tprint("%profiler-array-local-input.jai", #filepath), w);
    user_body, basic_body: bool;

    while true {
        event := compiler_wait_for_message();
        if !event break;

        if event.kind == .TYPECHECKED for tc: (cast(*Message_Typechecked) event).procedure_bodies {
            body := tc.expression;
            if !body || !body.header || !body.enclosing_load || !body.enclosing_load.enclosing_import continue;

            module := body.enclosing_load.enclosing_import;

            if body.header.name == "main" user_body = true;

            if module.module_type == .MAIN_PROGRAM && body.header.name == "collect" {
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
                add_build_string("#run collect();", w);
            }
        }

        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
    assert(user_body);
    assert(basic_body, "compiler omitted the original local-array body");
}

main :: () {}
