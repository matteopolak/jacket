// Binding integration reproducer: -check -no_dce -os macos -cpu x64.
#import "Basic";

main :: () {
    assert(STRING_BUILDER_BUFFER_SIZE > 0);
}
