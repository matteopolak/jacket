Parcel :: struct(T: Type, capacity: u64) {
    value: T;
}

write :: (p: *Parcel($T, $count), value: T) {
    p.value = value;
}

main :: () {
    p: Parcel(int, 20);
    write(*p, 1);
}
