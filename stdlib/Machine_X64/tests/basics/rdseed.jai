// Reviewed public hardware attempt; no success rate or entropy promise is invented.
#import "Basic";
M :: #import "Machine_X64";

main :: () {
    for 0..127 {
        success, value := M.rdseed();
        if !success assert(value == 0);
    }

    print("ok\n");
}
