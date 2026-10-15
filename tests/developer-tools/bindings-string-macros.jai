// Reviewed evaluated string literals and public real macro generation.
#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

strings: int;
errors: int;
expected_errors: int;

capture_error :: (message: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR {
        errors += 1;
        if contains(message, "not a supported byte string constant") expected_errors += 1;
    }
}

customize :: (decl: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .MACRO_DEFINITION {
        assert(decl.expression.kind == .LITERAL);
        assert((cast(*B.Literal) decl.expression).literal_kind == .STRING);
        assert(decl.type.hardcoded_jai_string == "string");
        strings += 1;
        if decl.name == "STRING_SIMPLE" decl.output_name = "RENAMED_STRING";
    }

    return .RECURSE;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-string-macros.h", #filepath));
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_string_macros", identifier = "native" });
    defer array_free(options.source_files);
    defer array_free(options.libraries);
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-string-macros-generated.jai", #filepath), log_results = false));
    options.visitor = customize;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-string-macros-custom.jai", #filepath), log_results = false));
    assert(strings == 6);

    options.visitor = null;
    options.source_files[0] = tprint("%bindings-macro-string.h", #filepath);
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-original-string-generated.jai", #filepath), log_results = false));

    rejected := tprint("%../../target/developer-tools-string-macros-preserved.jai", #filepath);

    assert(write_entire_file(rejected, "preserve"));

    for name: string.["bindings-macro-wide-string.h", "bindings-macro-nul-string.h"] {
        options.source_files[0] = tprint("%%", #filepath, name);

        for customized: 0..1 {
            options.visitor = ifx customized == 0 then null else customize;

            push_context {
                context.logger = capture_error;
                assert(!B.generate_bindings(options, rejected, log_results = false));
            }

            bytes, ok := read_entire_file(rejected);

            defer free(bytes);
            assert(ok && bytes == "preserve");
        }
    }

    assert(errors >= 4 && expected_errors == 4 && strings == 6);
    print("ok\n");
}
