#import "Basic";

Holder :: struct(Elem: Type) {
    storage: *Elem;
}

operator *[] :: (using holder: Holder, index: int) -> *holder.Elem {
    return storage + index;
}

make :: (p: *s64) -> Holder(s64) {
    return .{ p };
}

verify :: () -> bool {
    values: [3] s64;
    values[0] = 7;
    holder: Holder(s64);
    holder.storage = *values[0];
    before := make(*values[0])[0];
    address := *holder[0];

    holder[1] = 13;

    return before == 7 && values[1] == 13 && address == *values[0];
}

#assert verify();

main :: () {
    assert(verify());
}
