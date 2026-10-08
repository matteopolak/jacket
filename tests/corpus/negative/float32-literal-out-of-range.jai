// rules: num.18
// A literal beyond float32's range would become infinity: it needs an explicit cast.
main :: () {
    x: float32 = 1.0e39;
}
