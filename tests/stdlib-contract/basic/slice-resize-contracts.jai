// Portable storage coverage independent of the native elapsed-clock companion.
#import "Basic";

main :: () {
    values: [] u64;

    array_resize(*values, 3);
    assert(values.count == 3);
    assert(values[0] == 0 && values[1] == 0 && values[2] == 0);
    values[0] = 29;
    array_resize(*values, 1);
    assert(values.count == 1 && values[0] == 29);
    array_resize(*values, 4, initialize = false);
    assert(values.count == 4 && values[0] == 29);
    array_free(values);

    print("ok\n");
}
