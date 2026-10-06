// rules: using.4
// Without #as, a struct with a using member does not convert to the embedded type.
Inner :: struct { n: int; }
Plain :: struct { using i: Inner; }
take :: (i: Inner) -> int { return i.n; }
main :: () {
    p: Plain;
    take(p);
}
