// x[i] uses a matching operator [], else operator *[] dereferenced (an assignable place).
// rules: opov.7 opov.8
#import "Basic";

Array :: struct(T: Type) {
    data: [4] T;
}

operator [] :: (a: Array($T), i: int) -> T {
    return a.data[i];
}

Bucket :: struct(T: Type) {
    items: [4] T;
}

operator *[] :: (b: *Bucket, i: int) -> *b.T {
    return *b.items[i];
}

Thing :: struct {
    deleted: bool;
    n: int;
}

main :: () {
    b: Bucket(Thing);
    b[2].deleted = true;
    b[2].n += 5;
    a: Array(int);
    a.data[1] = 7;
    assert(b.items[2].deleted && b[2].n == 5 && a[1] == 7);

}
