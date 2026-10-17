// Dynamic-array append/find/reset contracts in Basic.
#import "Basic";

main :: () {
    values: [..] s64;

    array_add(*values, 12);
    array_add(*values, -4);
    array_add(*values, 12);
    found, index := array_find(values, 12);

    assert(found && index == 0);
    found, index = array_find(values, 99);
    assert(!found && index == -1);
    array_ordered_remove_by_index(*values, 0);
    assert(values.count == 2 && values[0] == -4 && values[1] == 12);
    array_reset_keeping_memory(*values);
    assert(values.count == 0);
    print("ok\n");
}
