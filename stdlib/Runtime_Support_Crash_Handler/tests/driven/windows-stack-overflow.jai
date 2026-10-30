#import "Basic";

// Each frame hands its buffer to a foreign call. Without that, the optimizer
// is free to turn this recursion into a loop that never overflows anything.
depth :: (n: s64, pad: *s64) -> s64 {
    buffer: [64] s64;

    query_performance_counter(*buffer[n % 64]);
    buffer[n % 64] += n;
    pad.* += buffer[(n + 1) % 64];

    return depth(n + 1, pad) + buffer[n % 64];
}

main :: () {
    kernel32_set_error_mode(get_error_mode() | 2);
    pad: s64;
    print("recursion unexpectedly returned %\n", depth(0, *pad));
}

#scope_file
kernel32 :: #system_library "kernel32";
kernel32_set_error_mode :: (mode: u32) -> u32 #foreign kernel32 "SetErrorMode";
get_error_mode :: () -> u32 #foreign kernel32 "GetErrorMode";
query_performance_counter :: (count: *s64) -> s32 #foreign kernel32 "QueryPerformanceCounter";
