B :: #import "Basic";

Parcel :: struct(T: Type, capacity: u64) {
    value: T;
}

Reversed :: struct(capacity: u64, T: Type) {
    value: T;
}

Mixed :: struct(T: Type, unsigned: u64, signed: s32) {
    value: T;
}

Seeded :: struct(seed: $Seed, capacity: u64) {
    value: Seed;
}

IntParcel :: #bake_arguments Parcel(T = int);

Domain :: struct {
    Parcel :: struct(T: Type, capacity: u64) {
        value: T;
    }
    IntParcel :: #bake_arguments Parcel(T = int);
}

write_qualified :: (p: *Domain.IntParcel($Count), value: int) -> u64 {
    p.value = value;
    return Count;
}

write :: (p: *Parcel($Element, $Count), value: Element) -> u64 {
    p.value = value;
    return Count;
}

write_reversed :: (p: *Reversed($Count, $Element), value: Element) -> u64 {
    p.value = value;
    return Count;
}

unsigned_only :: (value: u64) -> bool {
    return value == 20;
}

signed_only :: (value: s32) -> bool {
    return value == 20;
}

write_mixed :: (p: *Mixed($T, $U, $S), value: T) -> bool {
    p.value = value;
    return unsigned_only(U) && signed_only(S);
}

seeded :: (p: *Seeded($Value, $Count)) -> bool {
    return Value == 3 && unsigned_only(Count);
}

write_baked :: (p: *IntParcel($Count), value: int) -> u64 {
    p.value = value;
    return Count;
}

check :: () -> bool {
    a: Parcel(int, 20);
    b: Parcel(u64, 30);
    c: Parcel(int, 30);
    reversed: Reversed(20, int);
    mixed: Mixed(int, 20, 20);
    seed: Seeded(3, 20);
    baked: IntParcel(20);
    qualified: Domain.IntParcel(20);

    return write(*a, 1) == 20 && write(*b, cast(u64) 2) == 30
        && write(*c, 3) == 30 && write_reversed(*reversed, 4) == 20
        && write_mixed(*mixed, 6)
        && seeded(*seed) && write_baked(*baked, 7) == 20
        && a.value == 1 && b.value == 2 && c.value == 3
        && reversed.value == 4 && mixed.value == 6 && baked.value == 7 && write_qualified(*qualified, 8) == 20 && qualified.value == 8;
}

#assert check();

main :: () {
    B.assert(check());
}
