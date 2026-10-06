// A failing #assert is reported at compile time with its message.
// rules: decl.19 note.3
Rec :: struct {
    a: s64;
    b: s64;
}
#assert size_of(Rec) == 8 "Rec must be 8 bytes";
main :: () {}
