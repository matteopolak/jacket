// A program with no standard output keeps running when it prints: the text is dropped. A
// Windows program linked with /SUBSYSTEM:WINDOWS starts this way, with no console handles.
#import "Basic";

#if OS == .WINDOWS {
    kernel32 :: #system_library "kernel32";
    SetStdHandle :: (identifier: s32, handle: *void) -> s32 #foreign kernel32;

    drop_standard_output :: () {
        SetStdHandle(-11, null);
    }
} else {
    POSIX :: #import "POSIX";

    drop_standard_output :: () {
        POSIX.close(1);
    }
}

main :: () {
    drop_standard_output();
    print("dropped\n");
    print("dropped % too\n", 42);
    write_string("still running\n", to_standard_error = true);
}
