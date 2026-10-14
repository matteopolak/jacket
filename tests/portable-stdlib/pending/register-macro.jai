#import "Basic";

increment :: (r: __reg) #expand {
    #asm { add r, 1; }
}

main :: () {
    value: u64 = 11;
    increment(value);
    assert(value == 12);
}
