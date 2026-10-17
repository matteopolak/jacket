B :: #import "Basic";

Parcel :: struct {
    value: int;
}

compare :: (a: Parcel, b: Parcel) -> bool {
    return a.value % 10 == b.value % 10;
}

check :: () -> bool {
    items := Parcel.[.{ 12 }, .{ 13 }];
    found, index := B.array_find(items, Parcel.{ 2 }, compare = compare);
    return found && index == 0;
}

#assert check();

main :: () {
    B.assert(check());
}
