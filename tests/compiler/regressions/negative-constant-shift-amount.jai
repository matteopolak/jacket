// A negative constant shift amount shifts every bit out; it is not a range error. `64 - x` is -63
// for a baked `x` of 127, in code that the `x >= 64` check before it never lets run.
#import "Basic";

rotate :: (a: u64, $$x: u8) -> u64 {
    if x == 0 return a;
    if x >= 64 return 0;
    return (a >> x) | (a << (64 - x));
}

K :: 3;

main :: () {
    assert(rotate(5, 127) == 0);
    assert(rotate(5, 63) == 10);
    assert(rotate(5, 0) == 5);

    value: u64 = 5;

    assert(value << -1 == 0);
    assert(value >> (K - 5) == 0);
}

#run main();
