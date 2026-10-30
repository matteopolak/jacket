// Two procedures take a call equally well, one declared by the calling module and one imported.
// An import of the module's files loses to the module's own, but a `using` of the imported
// module in a body is nearer than the module's declarations and wins; a body's `#import` joins
// the file imports.
#import "Basic";
Own :: #import "Using_Tie_Own";

main :: () {
    assert(Own.from_file_import() == 1);
    assert(Own.from_body_import() == 1);
    assert(Own.from_using() == 2);
    assert(Own.from_block_using() == 2);
    assert(Own.from_nested_using() == 2);
    print("ok\n");
}
