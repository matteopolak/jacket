#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

errors: s64;
expected_diagnostic: bool;

capture_error :: (text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR {
        errors += 1;
        if contains(text, "not a supported byte string constant") expected_diagnostic = true;
    }
}

visit :: (_decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    return .RECURSE;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-macro-values.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_macros", identifier = "native" });
    defer array_free(options.libraries);
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-macros-generated.jai", #filepath), log_results = false));
    path := tprint("%../../target/developer-tools-macros-rejected.jai", #filepath);

    assert(write_entire_file(path, "existing bindings"));
    options.visitor = visit;
    // Ordinary byte strings are supported now; wide strings retain the real
    // unsupported-input diagnostic/preservation regression.
    options.source_files[0] = tprint("%bindings-macro-wide-string.h", #filepath);

    push_context {
        context.logger = capture_error;
        assert(!B.generate_bindings(options, path, log_results = false));
    }

    preserved, preserved_ok := read_entire_file(path);

    defer free(preserved);
    assert(errors > 0 && expected_diagnostic && preserved_ok && preserved == "existing bindings");
    print("ok\n");
}
