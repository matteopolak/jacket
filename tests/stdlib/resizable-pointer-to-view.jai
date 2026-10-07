// `*[..] T` converts implicitly to `*[] T` (a resizable array starts with its view).
#import "Basic";

shrink :: (items: *[] int) {
    items.count -= 1;
    items.data[0] = 9;
}

main :: () {
    list: [..] int;
    array_add(*list, 1, 2, 3);
    shrink(*list);
    assert(list.count == 2 && list[0] == 9 && list[1] == 2);
}
