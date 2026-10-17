B :: #import "Basic";

Parcel :: struct {
    value: int;
}

operator == :: (a: Parcel, b: Parcel) -> bool {
    return a.value % 10 == b.value % 10;
}

main :: () {
    items := Parcel.[.{ 12 }];
    found, index := B.array_find(items, Parcel.{ 2 }); // jailint: allow(unused_variable) the call itself is what is tested
}
