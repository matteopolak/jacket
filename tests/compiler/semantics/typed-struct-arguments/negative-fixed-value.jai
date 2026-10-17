Parcel :: struct(T: Type, capacity: u64) {
    value: T;
}

fixed :: (p: *Parcel($T, 21)) {}

main :: () {
    p: Parcel(int, 20);
    fixed(*p);
}
