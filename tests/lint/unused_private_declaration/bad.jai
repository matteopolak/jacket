#import "Basic";

run :: () -> int {
    return helper(2);
}

#scope_file

LIMIT :: 64;

Counter :: struct {
    value: int;
}

orphan_count: int;

helper :: (n: int) -> int {
    return n * 2;
}

clamp_index :: (i: int, count: int) -> int {
    return max(0, min(i, count - 1));
}

spin :: (n: int) -> int {
    if n <= 0 return 0;
    return spin(n - 1);
}

Mode :: enum {
    SLOW;
    FAST;
}

#scope_export

main :: () {
    print("%\n", run());
}
