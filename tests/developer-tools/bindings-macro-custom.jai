#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

macros: s64;

visit :: (decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .MACRO_DEFINITION {
        macros += 1;
        assert(decl.type != null && decl.expression != null && decl.expression.kind == .LITERAL);
        if decl.name == "ORIGINAL_INTEGER" decl.output_name = "RENAMED_INTEGER";
        if decl.name == "ORIGINAL_LONG_BYTES" decl.macro_value_result = .DISCARD;
    }

    return .RECURSE;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-macro-values.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_macros", identifier = "native" });
    defer array_free(options.libraries);
    options.visitor = visit;
    options.add_generator_command = false;
    path := tprint("%../../target/developer-tools-macros-custom-generated.jai", #filepath);

    assert(B.generate_bindings(options, path, log_results = false));
    assert(macros == 6);
    text, ok := read_entire_file(path);

    defer free(text);
    assert(ok && contains(text, "RENAMED_INTEGER ::") && !contains(text, "ORIGINAL_LONG_BYTES ::"));
    print("ok\n");
}
