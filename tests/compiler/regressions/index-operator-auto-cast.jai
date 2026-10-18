// `value[xx name]` on a struct with `operator []` casts to the parameter of the overload that
// takes the struct, here a `*u8`.
#import "Basic";

J :: struct { v: int; }

operator [] :: (j: J, name: *u8) -> int { return j.v + name[0]; }
operator [] :: (j: *J, name: string) -> int { return -1; }
operator [] :: (j: *J, name: *u8) -> int { return -2; }

main :: () {
    j := J.{10};
    assert(j[xx "A"] == 75);
}
