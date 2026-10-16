Basic :: #import "Basic"()(MEMORY_DEBUGGER = true);
Files :: #import "File_Utilities";

consume :: (value: Basic.Apollo_Time) {
    #assert type_of(value) == Basic.Apollo_Time;
}

main :: () {
    modification, _, _ := Files.file_modtime_and_size(".");
    consume(modification);
}
