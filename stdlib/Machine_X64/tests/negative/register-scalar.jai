// Original scalar probe reclassified: the reviewed __reg contract requires a SIMD operand.
#import "Basic";

increment :: (r: __reg) #expand {
    #asm { add r, 1; }
}

main :: () {
    value: u64 = 11;
    increment(value);
    assert(value == 12);
}
