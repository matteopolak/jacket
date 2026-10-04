#import "Basic";
Ranks :: #import "RadixSort";

main :: () -> s64 {
    sorter: Ranks.RadixSort;
    defer Ranks.free_resources(*sorter);
    integers := u64.[0xffff_ffff_ffff_ffff, 4, 4, 0, 0x8000_0000_0000_0000];
    Ranks.sort(*sorter, integers);
    if !sorter.valid_ranks || sorter.ranks.count != 5 || sorter.ranks2.count != 5 return 1;
    if sorter[0] != 3 || sorter[1] != 1 || sorter[2] != 2 || sorter[4] != 0 return 2;
    integers[3] = 4;
    Ranks.sort(*sorter, integers);
    if sorter[0] != 3 || sorter[1] != 1 || sorter[2] != 2 return 3;
    Ranks.reset(*sorter);
    if sorter.valid_ranks return 4;
    Ranks.sort(*sorter, integers);
    if sorter[0] != 1 || sorter[1] != 2 || sorter[2] != 3 return 5;
    decimals := float.[3.25, -2, 0, -0.0, 3.25];
    // Valid ranks seed the stable merge; start the float case from identity.
    Ranks.reset(*sorter);
    Ranks.sort(*sorter, decimals);
    if sorter[0] != 1 || sorter[1] != 2 || sorter[2] != 3 || sorter[4] != 4 return 6;
    data: [..]float;
    array_copy(*data, decimals);
    defer array_free(data);
    Ranks.reorder(*data, sorter);
    for i: 1..data.count-1 if data[i-1] > data[i] return 7;
    Ranks.resize(*sorter, 2);
    if sorter.ranks.count != 2 || sorter.ranks2.count != 2 || sorter.valid_ranks return 8;
    Ranks.free_resources(*sorter);
    if sorter.ranks.data != null || sorter.ranks2.data != null || sorter.valid_ranks return 9;
    return 0;
}
