Crt :: #library,system "libc";
absolute :: (value: s32) -> s32 #foreign Crt "abs";

main :: () -> int {
    if absolute(-42) == 42 return 0;
    return 1;
}
