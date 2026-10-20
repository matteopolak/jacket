// A parameter pattern with a fixed argument rejects other instantiations.
#import "Basic";
Box :: struct (T: Type, N: int) { items: [N] T; }
proc_pair :: (b: Box($T, 2)) {}
main :: () {
    b: Box(int, 5);
    proc_pair(b);
}
