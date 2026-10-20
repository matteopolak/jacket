// An untyped struct literal cannot bind a polymorphic parameter.
#import "Basic";
Ship :: struct { engines := 4; }
check :: (s: $T) {}
main :: () {
    check(.{engines = 8});
}
