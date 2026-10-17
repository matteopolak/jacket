// Reviewed public hardware attempt; runner must check the actual CPUID prerequisite.
#import "Basic";
M :: #import "Machine_X64";

main :: () {
    for 0..127 {
        success, value := M.rdrand();
        if !success assert(value == 0);
    }

    print("ok\n");
}
