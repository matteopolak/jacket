// A baked `$xs: [$N] T` takes `$N` from the count of a constant array view
// (`xs :: #run make()`).
#import "Basic";

make :: () -> [] int {
    a := NewArray(3, int);
    a[0] = 1; a[1] = 2; a[2] = 3;
    return a;
}

count_of :: ($xs: [$N] int) -> int {
    return N;
}

main :: () {
    c :: #run make();
    assert(count_of(c) == 3);
}
