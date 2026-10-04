main :: () -> int {
    f: float = 3.0; a: s16 = 2; c: u8 = 2; b: s32 = 3; d: float64 = 1.5;
    g := f * a;
    h := d * b;
    k: float = c;
    return cast(int) (g + cast(float) h + k);
}
