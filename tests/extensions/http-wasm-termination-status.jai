// Distinguish an explicit C exit status from main's return and from a host trap.
#if OS == .WASM exit :: (status: s32) #foreign;

#import "Basic";

main :: () {
    print("before exit 37\n");
    exit(37);
    print("ERROR: execution continued after exit 37\n");
}
