// rules: compiler.23 compiler.24
// remap_import redirects an import; provide_import answers a FAILED_IMPORT message with a
// module name, a file, a directory or the module's text, and the workspace then compiles.
#import "Basic";
#import "Compiler";

Result :: struct {
    failed_imports: [..] string;
    error_code: s64 = -1;
    errors: s64;
}

// Builds `source` in a new workspace; `answer` (if any) is called for each FAILED_IMPORT.
build :: (source: string, remap: bool, answer: (w: Workspace, m: *Message_Failed_Import)) -> Result {
    result: Result;
    w := compiler_create_workspace("probe");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    if remap remap_import(w, "", "Not_A_Module", "Math");
    compiler_begin_intercept(w);
    add_build_string(source, w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == {
            case .FAILED_IMPORT;
                failed := cast(*Message_Failed_Import) message;
                array_add(*result.failed_imports, copy_string(failed.target_module_name));
                if answer answer(w, failed);
            case .ERROR;
                result.errors += 1;
            case .COMPLETE;
                result.error_code = xx (cast(*Message_Complete) message).error_code;
        }
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
    return result;
}

USES_REMAPPED :: #string JAI
#import "Basic";
M :: #import "Not_A_Module";
#run assert(M.abs(-3) == 3);
main :: () {}
JAI

USES_PROVIDED :: #string JAI
#import "Basic";
P :: #import "Provided_Mod";
#run assert(P.answer() == 42);
main :: () {}
JAI

USES_SHORT :: #string JAI
#import "Basic";
P :: #import "Provided_Mod";
#run assert(P.abs(-4) == 4);
main :: () {}
JAI

provide_text :: (w: Workspace, m: *Message_Failed_Import) {
    provide_import(w, m, .FULL_TEXT, "answer :: () -> int { return 42; }");
}

provide_short_name :: (w: Workspace, m: *Message_Failed_Import) {
    provide_import(w, m, .SHORT_NAME, "Math");
}

provide_file :: (w: Workspace, m: *Message_Failed_Import) {
    provide_import(w, m, .PATH_TO_FILE, tprint("%/modules/Provided_File.jai", #filepath));
}

provide_directory :: (w: Workspace, m: *Message_Failed_Import) {
    provide_import(w, m, .PATH_TO_DIRECTORY, tprint("%/modules/Provided_Dir", #filepath));
}

checks :: () {
    // remap_import: `Not_A_Module` does not exist, `Math` does.
    r := build(USES_REMAPPED, true, null);
    assert(r.error_code == 0 && r.failed_imports.count == 0);

    // provide_import with each kind of value.
    r = build(USES_PROVIDED, false, provide_text);
    assert(r.failed_imports.count == 1 && r.failed_imports[0] == "Provided_Mod");
    assert(r.error_code == 0 && r.errors == 0);

    r = build(USES_SHORT, false, provide_short_name);
    assert(r.failed_imports.count == 1 && r.error_code == 0);

    r = build(USES_PROVIDED, false, provide_file);
    assert(r.failed_imports.count == 1 && r.error_code == 0);

    r = build(USES_PROVIDED, false, provide_directory);
    assert(r.failed_imports.count == 1 && r.error_code == 0);

    print("ok\n");
}

// Compile-time only: the Compiler module's procedures need the compiler.
#run checks();

main :: () {}
