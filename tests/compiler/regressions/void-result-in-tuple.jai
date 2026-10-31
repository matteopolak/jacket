// A `void` result next to other results is a value of its own: both names of `a, b := call()` bind, and `b` is a void variable.
#import "Basic";

with_void :: (ok: bool) -> (bool, void) {
    nothing: void;

    return ok, nothing;
}

named :: () -> (found: bool, nothing: void, count: s64) {
    nothing: void;

    return true, nothing, 7;
}

Holder :: struct($Value: Type) {
    present: bool;
}

lookup :: (holder: Holder($Value)) -> (bool, Value) {
    value: Value;

    return holder.present, value;
}

twice :: (ok: bool) -> bool {
    defer {}

    first, _ := with_void(ok);
    second, nothing := with_void(!ok);

    return first != second && type_of(nothing) == void;
}

main :: () {
    one := with_void(true);

    assert(one);

    found, nothing := with_void(true);

    assert(found);
    assert(type_of(nothing) == void);

    missing, _ := with_void(false);

    assert(!missing);

    if with_void(true) {} else assert(false);

    procedure := with_void;
    indirect, also_nothing := procedure(true);

    assert(indirect);
    assert(type_of(also_nothing) == void);

    set: Holder(void);
    set.present = true;
    present, empty := lookup(set);

    assert(present);
    assert(type_of(empty) == void);

    assert(twice(true));

    flag, void_value, count := named();

    assert(flag && count == 7 && type_of(void_value) == void);

    print("ok\n");
}
