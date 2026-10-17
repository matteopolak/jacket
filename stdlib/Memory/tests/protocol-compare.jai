// Reviewed protocol identifiers and coordinator 2026-10-15 equality clarification.
using #import "Basic";
P :: #import "Protocol_For_Memory_Visualization";

exercise :: () {
    pointers := (*void).[null, cast(*void) 1, cast(*void) 2, cast(*void) 0x100000000,
        cast(*void) 0x7fffffffffffffff, cast(*void) 0x8000000000000000, cast(*void) 0xffffffffffffffff];
    for allocation, i: pointers for release, j: pointers {
        a := P.Alloc_And_Free_Traces.{ allocation, release };

        for other_allocation, k: pointers for other_release, l: pointers {
            b := P.Alloc_And_Free_Traces.{ other_allocation, other_release };
            expected := i == k && j == l;
            assert(P.compare_alloc_and_free_traces(a, b) == expected);
            assert(P.compare_alloc_and_free_traces(b, a) == expected);
        }

        assert(a.alloc_site_trace == allocation && a.free_site_trace == release);
    }
}

#run exercise();

main :: () {
    exercise();
    print("ok\n");
}
