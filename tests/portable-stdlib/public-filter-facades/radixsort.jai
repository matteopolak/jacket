#import "Basic";
#import "RadixSort";
R :: #import "RadixSort";
Again :: #import "RadixSort";

main :: () {
    input := u32.[9, 2, 7];
    sorter: RadixSort;

    defer free_resources(*sorter);
    sort(*sorter, input);
    assert(sorter[0] == 1 && sorter[1] == 2 && sorter[2] == 0);
    first: (RadixSort, int) -> u32 = R.operator [];
    second: (Again.RadixSort, int) -> u32 = Again.operator [];

    assert(first == second && first(sorter, 1) == rank(*sorter, 1));
    print("ok\n");
}
