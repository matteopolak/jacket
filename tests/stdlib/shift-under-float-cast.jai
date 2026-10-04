#import "Basic";

V :: struct { x: s64; }

main :: () {
    d := V.{640};
    sx: float = 1;
    sx += cast(float) (d.x >> 6);
    assert(sx == 11);
    n := 3;
    f := cast(float) (1 << n);
    assert(f == 8);
    print("ok\n");
}
