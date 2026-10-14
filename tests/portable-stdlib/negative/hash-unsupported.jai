H :: #import "Hash";

Thing :: struct {
    value: int;
}

main :: () {
    H.get_hash(Thing.{ 1 });
}
