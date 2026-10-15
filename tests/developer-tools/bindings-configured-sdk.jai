#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    version := B.clang_getClangVersion();

    defer B.clang_disposeString(version);
    assert(to_string(B.clang_getCString(version)).count > 0);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-configured-sdk.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_configured", identifier = "configured_native" });
    defer array_free(options.libraries);
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-configured-generated.jai", #filepath), log_results = false));
    print("ok\n");
}
