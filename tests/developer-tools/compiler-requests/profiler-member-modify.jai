#import "Basic";
#import "Compiler";

#run {
    w := compiler_create_workspace("Original member-body modification probe");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_file(tprint("%profiler-member-input.jai", #filepath), w);
    modified: bool;

    while true {
        event := compiler_wait_for_message();
        if !event break;

        if event.kind == .TYPECHECKED for tc: (cast(*Message_Typechecked) event).procedure_bodies {
            body := tc.expression;
            if !body || !body.header || !body.enclosing_load || !body.enclosing_load.enclosing_import continue;

            if body.enclosing_load.enclosing_import.module_type == .MAIN_PROGRAM && body.header.name != "main" {
                print("member name=[%], flags=%\n", body.header.name, cast(u64) body.header.procedure_flags);
                compiler_modify_procedure(w, body);
                modified = true;
            }
        }

        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(w);
    assert(modified);
}

main :: () {}
