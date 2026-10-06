// A result type may name parameters: `-> type_of(asset)` is the parameter's type.
Asset :: struct(T: Type) {
    ref_count: int;
    value: T;
}
add_ref :: (using asset: *Asset, location := #caller_location) -> type_of(asset) {
    ref_count += 1;
    return asset;
}
twice :: (x: $T) -> type_of(x) {
    return x + x;
}
main :: () -> s64 {
    a: Asset(float);
    p := add_ref(*a);
    if p != *a || p.ref_count != 1 return 1;
    if twice(21) != 42 return 2;
    return 0;
}
