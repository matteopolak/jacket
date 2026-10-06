// rules: must.11
// An argument to a #discard parameter is not evaluated, but it is still typechecked.
check :: (#discard x: int) {}
main :: () { check("not an int"); }
