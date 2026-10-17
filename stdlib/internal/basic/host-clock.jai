// Compile-time code runs on the machine running the compiler, whichever OS the program targets.
// When the two differ, the target's C library constants are wrong for that machine (Linux and
// macOS number `clock_gettime`'s clocks differently), so `#run` code asks the compiler for the
// host's monotonic clock instead.
#if OS != BUILD_OS {
    host_monotonic_time :: () -> Apollo_Time {
        return nanoseconds_to_apollo(compile_time_host_monotonic_nanoseconds());
    }

    // The interpreter answers this call itself; the body only exists for programs that reach it
    // outside compile-time execution, which `current_time_monotonic` never does.
    compile_time_host_monotonic_nanoseconds :: () -> s64 #compiler #no_context {
        return 0;
    }
}
