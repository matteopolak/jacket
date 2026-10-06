// rules: struct.5
// A struct that is never used is still laid out, so a member of an undefined type is an error.
Unused :: struct {
    a: int;
    b: Missing_Type;
}

main :: () {
}
