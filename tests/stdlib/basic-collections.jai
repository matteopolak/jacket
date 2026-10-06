// Authored behavioral smoke input; execution belongs to the integration owner.
#import "Basic";
Bits :: #import "Bit_Array";
Tables :: #import "Hash_Table";
Heap_Sort :: #import "IntroSort";
Ranks :: #import "RadixSort";

main :: () -> s64 {
    array: [..] s64;
    defer array_free(array);
    array_add(*array, 10);
    array_add(*array, 30);
    array_insert_at(*array, 20, 1);
    if array.count != 3 || array[1] != 20 return 1;
    array_ordered_remove_by_index(*array, 0);
    if array.count != 2 || array[0] != 20 return 2;
    value := pop(*array);
    if value != 30 || array.count != 1 return 3;

    bits: Bits.Bit_Array;
    Bits.init_bit_array(*bits, 67);
    defer Bits.deinit(*bits);
    Bits.set_bit(*bits, 66);
    Bits.toggle_bit(*bits, 0);
    if !bits[66] || !bits[0] return 4;
    Bits.clear_bit(*bits, 66);
    if bits[66] return 5;

    table: Tables.Table(string, s64);
    defer Tables.deinit(*table);
    Tables.table_add(*table, "one", 1);
    Tables.table_set(*table, "one", 2);
    Tables.table_set(*table, "two", 3);
    ok, found := Tables.table_find(*table, "one");
    if !ok || found != 2 || table.count != 2 return 6;
    removed, saved := Tables.table_remove(*table, "one");
    if !removed || saved != 2 || Tables.table_contains(*table, "one") return 7;

    data := s64.[4, 1, 3, 2, 2];
    compare :: (a: s64, b: s64) -> s64 {
        return a - b;
    }
    Heap_Sort.intro_sort(data, compare);
    for i: 1..data.count - 1 if data[i - 1] > data[i] return 8;
    ranker: Ranks.RadixSort;
    unsigned := u32.[7, 2, 2, 9];
    Ranks.sort(*ranker, unsigned.data, cast(u32) unsigned.count);
    if Ranks.rank(*ranker, 0) != 1 || Ranks.rank(*ranker, 1) != 2 return 9;
    return 0;
}
