// A plain load through a null pointer (`v := p.*;`) stops with a null pointer dereference on
// every backend. Native builds check the pointer before the load
// (`Build_Options.null_pointer_check`).
#import "Basic";

main :: () {
    k := 7;
    p := *k;
    v := p.*;
    print("%\n", v);
    p = null;
    v = p.*;
    print("unreachable %\n", v);
}
