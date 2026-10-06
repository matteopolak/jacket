#import "Basic";
main :: () { x := 5; a: Any; a.type = cast(*Type_Info) *x; a.value_pointer = *x; print("%\n", a); }
