#import "Basic";
#import "Compiler";
#run {
    w := compiler_create_workspace("Original called standard-library body probe");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w, 0);
    add_build_file(tprint("%profiler-stdlib-bodies-input.jai", #filepath), w);
    user_body, basic_body: bool;
    while true {
        event := compiler_wait_for_message();
        if !event break;
        if event.kind == .TYPECHECKED for tc: (cast(*Message_Typechecked) event).procedure_bodies {
            body := tc.expression;
            if !body || !body.header || !body.enclosing_load || !body.enclosing_load.enclosing_import continue;
            module := body.enclosing_load.enclosing_import;
            if body.header.name == "main" user_body = true;
            if (module.module_name == "Basic" || module.module_name == "internal/basic") && body.header.name == "alloc" basic_body = true;
        }
        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            break;
        }
    }
    compiler_end_intercept(w);
    assert(user_body);
    assert(basic_body, "compiler omitted the called ordinary Basic.alloc body from unfiltered interception");
}
main :: () {}
