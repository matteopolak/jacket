#import "Basic";
B :: struct(p: string) {
    #insert -> string { return "L :: enum u8 { x; y; }\nl: L;\n"; }
}
Plain :: struct {
    #insert "M :: enum u8 { a; b; }\nm: M;\n";
}
Sized :: struct(p: string) {
    N :: 4;
    a: [N] int;
}
main :: () {
    b: B("q");
    b.l = .y;
    print("%\n", b);
    p: Plain;
    p.m = .b;
    print("%\n", p);
    s: Sized("z");
    print("%\n", s.a.count);
}
