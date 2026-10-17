// Reviewed system.md: CPU selection precedes OS; Unix fallback is raw tv_nsec.
#import "Basic";
S :: #import "System";
#if CPU != .X64 P :: #import "POSIX";

main :: () {
    #if CPU == .X64 {
        #assert #exists(S.M64);
        cpu := S.M64.get_cpu_info();

        assert(S.M64.check_feature(cpu.feature_leaves, .TSC));

        for 0..63 {
            S.M64.lfence();
            before := S.M64.rdtsc();

            S.M64.lfence();
            actual := S.read_cpu_counter();

            S.M64.lfence();
            after := S.M64.rdtsc();

            S.M64.lfence();
            assert(before > 0 && before <= actual && actual <= after);
        }
    } else {
        #assert !#exists(S.M64);
        bracketed: s64;

        for 0..63 {
            before: P.timespec;
            after: P.timespec;
            status := P.clock_gettime(.MONOTONIC_RAW, *before);

            assert(status == 0);
            actual := S.read_cpu_counter();

            status = P.clock_gettime(.MONOTONIC_RAW, *after);
            assert(status == 0 && actual < 1000000000);

            if before.tv_sec == after.tv_sec {
                assert(cast(u64) before.tv_nsec <= actual && actual <= cast(u64) after.tv_nsec);
                bracketed += 1;
            }
        }

        assert(bracketed > 0);
    }

    print("ok\n");
}
