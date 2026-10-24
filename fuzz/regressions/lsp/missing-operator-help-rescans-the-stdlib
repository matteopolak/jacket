#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

visit :: (_decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    return .RECURSE;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-macro-values.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_macros", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-macros-generated.jai", #filepath), log_results = false));
    path := tprint("%../../../../target/developer-tools-macros-unsupported.jai", #filepath);

    options.visitor = visit;
    // Wide strings are not representable byte-string constants: the macro is
    // dropped from the output rather than failing the whole run.
    options.source_files[0] = tprint("%bindings-macro-wide-string.h", #filepath);
    assert(B.generate_bindings(options, path, log_results = false));
    text, ok := read_entire_file(path);

    defer free(text);
    assert(ok && !contains(text, "ORIGINAL_WIDE_STRING"));
    print("ok\n");
}
