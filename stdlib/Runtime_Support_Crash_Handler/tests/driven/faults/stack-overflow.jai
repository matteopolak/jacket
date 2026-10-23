#import "Basic";
C :: #import "Runtime_Support_Crash_Handler";

depth :: (n: s64, pad: *s64) -> s64 {
    buffer: [64] s64;
    buffer[n % 64] = n;
    pad.* += buffer[(n + 1) % 64];
    return depth(n + 1, pad) + buffer[n % 64];
}

main :: () {
    C.init();
    pad: s64;
    print("%\n", depth(0, *pad));
}
