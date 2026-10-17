// Arithmetic context must not silently round a required-float64 literal.
main :: () {
    factor: float32 = 2;
    value := factor * 3.14159265358979;
}
