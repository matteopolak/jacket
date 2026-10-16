M :: #import "Provider";

Parcel :: struct {
    value: int;
}

operator == :: (a: Parcel, b: Parcel) -> bool {
    return a.value % 10 == b.value % 10;
}

check :: () -> bool {
    return M.equal(Parcel.{ 12 }, Parcel.{ 2 });
}

#assert check();
main :: () {}
