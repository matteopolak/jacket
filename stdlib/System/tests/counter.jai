// test: only(.LINUX,.MACOS) skip_on(.WASM)
// Reviewed system.md: CPU selection precedes OS; Unix fallback is raw tv_nsec.
#import "Basic";
#import "Extensions/Testing";

cpu_counter_lies_between_two_reads_of_the_clock :: () {
    #if OS == .LINUX || OS == .MACOS {
        #if CPU == .X64 {
            #assert #exists(S.M64);
            cpu := S.M64.get_cpu_info();

            expect(S.M64.check_feature(cpu.feature_leaves, .TSC));

            for 0..63 {
                S.M64.lfence();
                before := S.M64.rdtsc();

                S.M64.lfence();
                actual := S.read_cpu_counter();

                S.M64.lfence();
                after := S.M64.rdtsc();

                S.M64.lfence();
                expect(before > 0 && before <= actual && actual <= after);
            }
        } else {
            #assert !#exists(S.M64);
            bracketed: s64;

            for 0..63 {
                before: P.timespec;
                after: P.timespec;
                status := P.clock_gettime(.MONOTONIC_RAW, *before);

                expect_equal(status, 0);
                actual := S.read_cpu_counter();

                status = P.clock_gettime(.MONOTONIC_RAW, *after);
                expect(status == 0 && actual < 1000000000);

                if before.tv_sec == after.tv_sec {
                    expect(cast(u64) before.tv_nsec <= actual && actual <= cast(u64) after.tv_nsec);
                    bracketed += 1;
                }
            }

            expect(bracketed > 0);
        }
    }
} @test

// The test runs only on Linux and macOS: elsewhere it is skipped and its body left out, since the
// directory still compiles as one program there.
#scope_file

#if OS == .LINUX || OS == .MACOS {
    S :: #import "System";
    #if CPU != .X64 P :: #import "POSIX";
}
