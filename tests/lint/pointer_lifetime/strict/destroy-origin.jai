#import "Basic";

Box :: struct { value: int; }
destroy :: (p: *Box) { free(p); }

probe :: () {
    p := New(Box);
    destroy(p);
    print("%", p.value);
}
