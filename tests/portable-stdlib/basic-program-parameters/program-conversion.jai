using Basic :: #import "Basic"()(MEMORY_DEBUGGER = true);
#import "File_Utilities";

main :: () {
    modification, _, _ := file_modtime_and_size(".");
    milliseconds, converted := to_milliseconds(modification);
    if converted assert(milliseconds >= 0);
}
