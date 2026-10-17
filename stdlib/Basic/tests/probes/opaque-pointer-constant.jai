// Original compiler dependency, independent of registry/modules/native calls.
// Positive integer-to-pointer constants carry address bits, not owned memory.
handle :: cast(*void) 0x80000001;
#assert cast(u64) handle == 0x80000001;

main :: () {
    assert(cast(u64) handle == 0x80000001);
}

#import "Basic";
