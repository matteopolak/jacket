using Basic :: #import "Basic";

Parcel :: struct {
    value: int;
}

operator == :: (a: Parcel, b: Parcel) -> bool {
    return a.value == b.value;
}

#poke_name Basic operator==;

main :: () {
    items := Parcel.[.{ value = 1 }, .{ value = 2 }];
    found, index := array_find(items, Parcel.{ value = 2 });
    assert(found && index == 1);
}
