#import "Basic";

libc :: #system_library "libc";
c_exit :: (status: s32) #foreign libc "exit";

main :: () {
    print("before\n");
    c_exit(7);
    print("after\n");
}
