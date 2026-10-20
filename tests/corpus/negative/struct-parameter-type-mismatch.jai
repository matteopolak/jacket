// A concrete instantiation parameter rejects other instantiations.
#import "Basic";
Box :: struct (T: Type, N: int) { items: [N] T; }
proc_exact :: (b: Box(int, 2)) {}
main :: () {
    a: Box(float, 2);
    proc_exact(a);
}
