// A loop's value and index cannot share a name.
main :: () {
    for a, a: 0..3 {}
}
