// A procedure declared in a struct body counts as the program's own code too.
Thing :: struct {
    value: int;
    helper :: (t: Thing) -> int { return t.value + missing_in_struct_proc; }
}
main :: () {}
