// The black_box intrinsic passes exactly one value through.
black_box :: (a: int, b: int) -> int #intrinsic;

main :: () {
    x := black_box(1, 2);
}
