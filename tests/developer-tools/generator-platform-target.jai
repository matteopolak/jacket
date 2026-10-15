#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    // Source-check all implemented bodies on each reviewed desktop branch.
    B.ensure_local_libclang_exists();
    B.ensure_local_libclang_exists("installed-clang");
    ok, names := B.os_get_symbol_names_from_file("original.o");
    content_ok, content_names := B.os_get_symbol_names_from_file_content("object bytes", "original.o");
    library := B.os_load_library(.[], "Original", "original", false);
    found := B.os_symbol_exists_in_library(library, "original");
    function: B.Function;
    missing := B.function_is_missing_foreign_library(*function);
    name: string = "_original";

    B.fix_macos_foreign_name(*name);
    print("% % % % % % %\n", ok, names.count, content_ok, content_names.count, found, missing, name);
}
