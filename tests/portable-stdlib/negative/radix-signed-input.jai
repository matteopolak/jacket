R :: #import "RadixSort";

main :: () {
    sorter: R.RadixSort;
    input: [2] s32 = .[-1, 1];
    R.sort(*sorter, input);
}
