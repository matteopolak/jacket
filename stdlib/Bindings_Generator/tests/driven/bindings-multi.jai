// Reviewed source_files array: shared native declarations across multiple headers.
#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

record_visits: int;
sum_visits: int;
errors: int;

customize :: (decl: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .STRUCT && decl.name == "MultiRecord" record_visits += 1;

    if decl.kind == .FUNCTION && decl.name == "multi_sum" {
        sum_visits += 1;
        decl.output_name = "custom_sum";
    }

    return .RECURSE;
}

capture_error :: (message: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-multi-base.h", #filepath));
    array_add(*options.source_files, tprint("%bindings-multi-extra.h", #filepath));
    // Repeated guarded input and its include must preserve one native declaration.
    array_add(*options.source_files, options.source_files[0]);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_multi", identifier = "native" });
    defer array_free(options.source_files);
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    normalized, ok := B.normalize_options(options);

    assert(ok && normalized.source_files.count == 3);
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-multi-generated.jai", #filepath), log_results = false));
    options.visitor = customize;
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-multi-custom.jai", #filepath), log_results = false));
    assert(record_visits == 1 && sum_visits == 1);

    options.visitor = null;
    options.libraries[0].filename = "developer_tools_multi_cpp";
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.extra_clang_arguments);
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-multi-cpp-generated.jai", #filepath), log_results = false));
    options.header = tprint("#include \"%bindings-multi-base.h\"\n#include \"%bindings-multi-extra.h\"\n", #filepath, #filepath);
    assert(B.generate_bridge(options, tprint("%../../../../target/developer-tools-multi-cpp-bridge.cpp", #filepath)));
    options.header = "";
    options.extra_clang_arguments.count = 0;

    path := tprint("%../../../../target/developer-tools-multi-preserved.jai", #filepath);

    assert(write_entire_file(path, "preserve"));
    options.source_files[1] = tprint("%bindings-malformed.h", #filepath);

    push_context {
        context.logger = capture_error;
        assert(!B.generate_bindings(options, path, log_results = false));
    }

    assert(errors > 0);
    assert(record_visits == 1 && sum_visits == 1);
    preserved, read_ok := read_entire_file(path);

    defer free(preserved);
    assert(read_ok && preserved == "preserve");
    print("ok\n");
}
