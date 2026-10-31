using Basic :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
#import "File_Utilities";

main :: () {
    modification, size, exists := file_modtime_and_size(".");

    assert(exists);
    assert(size >= 0);
    milliseconds, converted := to_milliseconds(modification);

    assert(converted);
    assert(milliseconds > 0);
}
