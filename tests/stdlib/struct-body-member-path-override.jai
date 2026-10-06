// Struct-body overrides of members reached through a dotted path (`base.f = show;`), as focus-editor's dialogs use.
#import "Basic";
Base :: struct(T: Type) {
    id: s64;
    f: (x: s64) -> ();
    g: () -> ();
}
Derived :: struct {
    using #as base: Base(int);
    base.id = 7;
    base.f = show;
}
Plain :: struct {
    h: () -> () = hello;
}
show :: (x: s64) {
    print("show %\n", x);
}
hello :: () {
    print("hello\n");
}

global_d: Derived;
global_p: Plain;

main :: () {
    local_d: Derived;
    print("% % %\n", global_d.id, global_d.f != null, local_d.f != null);
    if global_d.f global_d.f(1);
    if local_d.f local_d.f(2);
    if global_p.h global_p.h();
}
