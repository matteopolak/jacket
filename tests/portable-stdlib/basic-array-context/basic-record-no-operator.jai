B :: #import "Basic";

Parcel :: struct {
    value: int;
}

main :: () {
    items := Parcel.[.{ 12 }];
    found, index := B.array_find(items, Parcel.{ 12 });
}
