M :: #import "Facade";

Parcel :: struct {
    value: int;
}

operator == :: (a: Parcel, b: Parcel) -> bool {
    return a.value % 10 == b.value % 10;
}

#poke_name M operator==;

check :: () -> bool {
    return M.equal(Parcel.{ 12 }, Parcel.{ 2 });
}

#assert check();
main :: () {}
