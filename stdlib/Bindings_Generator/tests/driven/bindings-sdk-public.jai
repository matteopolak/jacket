#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-sdk.h", #filepath));
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_sdk", identifier = "sdk" });
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-sdk-generated.jai", #filepath), log_results = false));
    print("ok\n");
}
