A :: struct { x: int; y: int; }
B :: struct { x: float; z: int; }
C :: struct {
    using a: A;
    using b: B;
}
main :: () {}
