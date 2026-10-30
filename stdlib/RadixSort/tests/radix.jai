#import "Basic";
#import "RadixSort";
R :: #import "RadixSort";
Again :: #import "RadixSort";

verify :: (sorter: *R.RadixSort, input: [] $T) {
    if !sorter.valid_ranks || sorter.ranks.count != input.count exit(2);

    seen: [8] bool;

    for index: 0..input.count - 1 {
        rank := R.rank(sorter, index);
        if rank != sorter.*[index] || rank >= input.count || seen[rank] exit(3);
        if index > 0 && input[R.rank(sorter, index - 1)] > input[rank] exit(4);
        seen[rank] = true;
    }
}

main :: () {
    #assert !#exists(R.sort_ranks);
    #assert !#exists(R.sortable_key);
    #assert !#exists(R.resize_rank_buffer);
    reader: (R.RadixSort, int) -> u32 = R.operator [];
    repeated: (Again.RadixSort, int) -> u32 = Again.operator [];

    if reader != repeated exit(17);
    if type_info(R.RadixSort).name != "RadixSort" exit(5);

    sorter: R.RadixSort;

    defer R.free_resources(*sorter);
    small: [8] u32 = .[0xffffffff, 0, 256, 1, 255, 1, 0x80000000, 65536];
    original := small;

    R.sort(*sorter, small);
    verify(*sorter, small);
    for value, index: small if value != original[index] exit(6);

    R.sort(*sorter, small.data, 8);
    verify(*sorter, small);
    ranks: [8] u32;
    for value, index: sorter.ranks ranks[index] = value;

    R.sort(*sorter, cast(*u32) null, 8);
    R.sort(*sorter, small.data, 0);
    for value, index: sorter.ranks if value != ranks[index] exit(7);
    if !sorter.valid_ranks exit(8);

    previous := sorter.ranks.data;
    previous2 := sorter.ranks2.data;

    R.reset(*sorter);
    if sorter.valid_ranks || sorter.ranks.data != previous || sorter.ranks.count != 8 exit(9);

    R.resize(*sorter, 3);
    if sorter.valid_ranks || sorter.ranks.count != 3 || sorter.ranks2.count != 3 exit(10);
    if sorter.ranks.data != previous || sorter.ranks2.data != previous2 exit(11);

    large: [8] u64 = .[0xffffffffffffffff, 0, 0x100000000, 1, 0x8000000000000000, 1, 255, 65536];

    R.sort(*sorter, large);
    verify(*sorter, large);
    R.sort(*sorter, large.data, 8);
    verify(*sorter, large);
    floating: [8] float = .[3.5, -100, 0, 1, -1, 3.5, 0.25, -0.25];
    original_float := floating;

    R.sort(*sorter, floating);
    verify(*sorter, floating);
    R.sort(*sorter, floating.data, 8);
    verify(*sorter, floating);
    for value, index: floating if value != original_float[index] exit(12);

    R.resize(*sorter, 8);
    if !sorter.valid_ranks exit(13);

    data: [..] float;
    for value: floating array_add(*data, value);

    defer array_free(data);
    R.reorder(*data, sorter);
    for value, index: data if value != original_float[sorter[index]] exit(14);
    for index: 1..data.count - 1 if data[index - 1] > data[index] exit(15);

    R.free_resources(*sorter);
    if sorter.ranks.data != null || sorter.ranks2.data != null || sorter.valid_ranks exit(16);

    R.radix_tests();
    write_string("ok\n");
}
