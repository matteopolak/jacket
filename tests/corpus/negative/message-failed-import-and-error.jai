// rules: bo.13
// A workspace whose import finds no module sends FAILED_IMPORT, then ERROR, then COMPLETE
// with COMPILATION_FAILED. (The compiler also prints the error.)
#import "Basic";
#import "Compiler";

#run {
    w := compiler_create_workspace("t");
    opts := get_build_options(w);
    opts.output_type = .NO_OUTPUT;
    set_build_options(opts, w);
    compiler_begin_intercept(w);
    add_build_string("#import \"No_Such_Module_Here\"; main :: () {}", w);

    failed_imports := 0;
    errors := 0;
    complete_failed := false;
    while true {
        msg := compiler_wait_for_message();
        if msg.kind == {
            case .FAILED_IMPORT;
                failed := cast(*Message_Failed_Import) msg;
                assert(failed.status == .NOT_FOUND);
                assert(failed.target_module_name == "No_Such_Module_Here");
                failed_imports += 1;
            case .ERROR;
                errors += 1;
            case .COMPLETE;
                complete_failed = (cast(*Message_Complete) msg).error_code == .COMPILATION_FAILED;
        }
        if msg.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
    assert(failed_imports == 1 && errors == 1 && complete_failed);
    write_string("messages: failed import, error, complete\n", true);
}

main :: () {
    print("ok\n");
}
