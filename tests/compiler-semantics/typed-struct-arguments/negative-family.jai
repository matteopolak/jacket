Parcel :: struct(T: Type, capacity: u64) {
    value: T;
}

Other :: struct(T: Type, capacity: u64) {
    value: T;
}

write :: (p: *Parcel($T, $Count), value: T) {
    p.value = value;
}

main :: () {
    other: Other(int, 20);
    write(*other, 1);
}
