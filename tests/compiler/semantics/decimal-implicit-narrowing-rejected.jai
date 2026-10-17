// A required-float64 constant cannot narrow implicitly, including through its name.
PI_LONG :: 3.14159265358979;

main :: () {
    value: float32 = PI_LONG;
}
