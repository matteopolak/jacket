// Own minimal input supplied for the copied producer driver; no peer input was available.
R :: struct {
    touch :: (value: int) -> int {
        return value + 1;
    }
}

main :: () -> int {
    return R.touch(8);
}
