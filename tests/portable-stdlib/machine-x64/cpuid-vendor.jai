#import "Basic";

main :: () {
    #if CPU == .X64 {
        a: u32 = 0;
        b: u32;
        c: u32 = 0;
        d: u32;

        #asm { cpuid a, b, c, d; }
        assert((b | c | d) != 0, "CPUID leaf zero must return actual vendor bytes");
    } else {
        assert(false, "CPUID reproducer requires an x64 target");
    }
}
