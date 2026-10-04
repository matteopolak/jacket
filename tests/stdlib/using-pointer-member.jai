#import "Basic";
P :: struct { idx: int; ids: [..] int; }
E :: struct { active: *P; }
editors: E;
f :: () {
    using editors.active;
    if idx < 0 || !ids return;
    idx = 5;
}
main :: () { p: P; editors.active = *p; array_add(*p.ids, 1); f(); assert(p.idx == 5); print("ok\n"); }
