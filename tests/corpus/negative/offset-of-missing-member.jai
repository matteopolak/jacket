// `offset_of` names a member that does not exist.
S :: struct { a: int; }

main :: () {
    x := offset_of(S, "nope");
}
