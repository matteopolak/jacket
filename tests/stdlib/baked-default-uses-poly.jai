// A baked parameter default whose declared type names an earlier type variable.
#import "Basic";
find_first :: (arr: [] $T, item: T, $compare: (T, T) -> bool = (a, b) => a == b) -> int {
    for arr if compare(it, item) return it_index;
    return -1;
}
push_unique :: (arr: *[..] $T, item: T, $compare: (T, T) -> bool = (a, b) => a == b) -> bool {
    if find_first(arr.*, item, compare) == -1 {
        array_add(arr, item);
        return true;
    }
    return false;
}
main :: () -> s64 {
    a: [..] *int;
    x := 3;
    push_unique(*a, *x);
    push_unique(*a, *x);
    return ifx a.count == 1 then 0 else 1;
}
