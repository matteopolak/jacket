// rules: ptr.1
#import "Basic";

main :: () {
    a: [..] int;
    for 1..6 array_add(*a, it);
    for < *a {
        if it.* % 2 == 0 remove it;
    }
    assert(a.count == 3 && a[0] == 1 && a[1] == 5 && a[2] == 3);
    print("ok\n");
}
