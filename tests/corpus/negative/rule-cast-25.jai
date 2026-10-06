// A string literal converts to *u8, but a `string` variable does not.
// rules: cast.25
main :: () {
    s := "text";
    p: *u8 = s;
}
