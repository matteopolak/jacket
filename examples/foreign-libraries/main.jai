#import "Basic";

Crt :: #library,system "libc";
absolute :: (value: s32) -> s32 #foreign Crt "abs";

main :: () {
    if absolute(-42) == 42 exit(0);
    exit(1);
}
