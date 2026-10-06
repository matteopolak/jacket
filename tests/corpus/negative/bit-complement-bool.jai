// rules: ops.16
// `~` on a `bool` used to complement the byte, making a `bool` that holds 254.
main :: () {
    b := true;
    c := ~b;
}
