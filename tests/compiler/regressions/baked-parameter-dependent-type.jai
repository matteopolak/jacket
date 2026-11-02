// A baked parameter's type may name a type bound by an earlier parameter (`$e: C.E`), and the
// enum literal argument takes that type.
#import "Basic";

B :: struct {
    E :: enum {
        A;
        B;
    }
    x: int;
}

g1 :: (c: $C, $e: C.E) -> int {
    #if e == .B return 2;
    else return 1;
}

g2 :: (c: *$C, $e: C.E) -> int {
    return cast(int) e;
}

g3 :: ($C: Type, $e: C.E) -> int {
    return cast(int) e + 10;
}

main :: () {
    b: B;

    if g3(B, .A) != 10 exit(1);
    if g1(b, .A) != 1 exit(2);
    if g1(b, .B) != 2 exit(3);
    if g2(*b, .B) != 1 exit(4);

    print("ok\n");
}
