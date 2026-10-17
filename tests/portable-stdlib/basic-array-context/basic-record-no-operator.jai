B :: #import "Basic";

Parcel :: struct {
    value: int;
}

main :: () {
    items := Parcel.[.{ 12 }];
    found, index := B.array_find(items, Parcel.{ 12 }); // jailint: allow(unused_variable) the call itself is what is tested
}
