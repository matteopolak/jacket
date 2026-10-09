#import "Basic";

libc :: #system_library "libc";
labs :: (x: s64) -> s64 #foreign libc;

main :: () {
    total: s64;
    for 0..60_000_000 {
        total += labs(-it);
    }
    print("%\n", total);
}
