#import "Basic";
S :: #import "Sort";
I :: #import "IntroSort";

Item :: struct {
    key: s64;
    identity: s64;
}

compare :: (a: Item, b: Item) -> s64 {
    if a.key < b.key return -1;
    if a.key > b.key return 1;
    return 0;
}

key_of :: (item: Item) -> s64 {
    return item.key;
}

main :: () {
    #assert !#exists(S.sift);
    #assert !#exists(I.quick_sort);

    for length: s64.[0, 1, 2, 3, 16, 127, 128] {
        for mode: 0..3 {
            storage: [128] Item;
            for *item, index: storage item.* = .{ (index * 37) % 17 - 8, index };

            input := array_view(storage, 0, length);

            if mode == 0 {
                result := S.quick_sort(input, compare);
                if result.data != input.data || result.count != input.count exit(2);
            } else if mode == 1 {
                result := S.quick_sort(input, key_of);
                if result.data != input.data || result.count != input.count exit(3);
            } else if mode == 2 I.intro_sort(input, compare);
            else I.insertion_sort(input, compare);

            seen: [128] bool;

            for value, index: input {
                if index > 0 && input[index - 1].key > value.key exit(4);
                if value.identity < 0 || value.identity >= length || seen[value.identity] exit(5);
                if value.key != (value.identity * 37) % 17 - 8 exit(6);
                seen[value.identity] = true;
            }

            for index: length..127 if storage[index].identity != index || storage[index].key != (index * 37) % 17 - 8 exit(7);
        }
    }

    if S.compare_floats(-1, 2) >= 0 || S.compare_floats(2, -1) <= 0 || S.compare_floats(0, -0.0) != 0 exit(8);
    if S.compare_strings("a", "aa") >= 0 || S.compare_strings("b", "aa") <= 0 || S.compare_strings("same", "same") != 0 exit(9);

    floats: [5] float = .[3.5, -1.25, 0, 3.5, 2];
    result := S.quick_sort(floats, S.compare_floats);

    if result.data != floats.data || floats[0] != -1.25 || floats[4] != 3.5 exit(10);

    strings: [4] string = .["b", "", "aa", "a"];

    S.quick_sort(strings, S.compare_strings);
    if strings[0] != "" || strings[1] != "a" || strings[2] != "aa" || strings[3] != "b" exit(11);

    write_string("ok\n");
}
