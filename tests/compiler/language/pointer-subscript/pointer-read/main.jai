#import "Basic";

Holder :: struct(Elem: Type) {
    storage: *Elem;
}

operator *[] :: (using holder: Holder, index: int) -> *holder.Elem {
    return storage + index;
}

verify :: () -> bool {
    values: [3] s64;
    values[0] = 7;
    holder: Holder(s64);
    holder.storage = *values[0];
    p := *holder;
    before := p[0];
    address := *p[0];

    values[1] = 13;

    return before == 7 && values[1] == 13 && address == *values[0];
}

#assert verify();

main :: () {
    assert(verify());
}
