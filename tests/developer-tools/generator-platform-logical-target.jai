// Source-only desktop branch/ABI control, without pretending foreign execution.
#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    library := B.os_load_library(.[], "", "source_only", false);
    if library {
        assert(library.full_path.count > 0);
        B.os_symbol_exists_in_library(library, "original_resolved");
    }
}
