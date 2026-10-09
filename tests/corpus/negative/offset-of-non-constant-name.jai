// The member name of `offset_of` must be a constant string.
S :: struct { a: int; }

main :: () {
    name := "a";
    x := offset_of(S, name);
}
