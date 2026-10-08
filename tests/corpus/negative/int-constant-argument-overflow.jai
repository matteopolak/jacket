// rules: num.20
// The fit rules apply to arguments as to declarations.
take :: (x: s8) -> s8 {
    return x;
}

main :: () {
    take(200);
}
