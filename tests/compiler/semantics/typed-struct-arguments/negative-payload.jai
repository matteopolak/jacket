Parcel :: struct(T: Type, capacity: u64) {
    value: T;
}

write :: (p: *Parcel($T, $Count), value: T) {
    p.value = value;
}

main :: () {
    p: Parcel(int, 20);
    value: u64 = 1;
    write(*p, value);
}
