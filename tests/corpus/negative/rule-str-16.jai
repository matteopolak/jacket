// rules: str.16
// A string variable does not convert to *u8; only a literal does.
main :: () {
    s := "text";
    p: *u8 = s;
}
