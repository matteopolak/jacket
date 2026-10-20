// A `using` field converts its outer struct implicitly only when marked `#as`.
#import "Basic";
Vehicle :: struct { make: string; }
Moto :: struct { using ve: Vehicle; top: float; }
main :: () {
    m: Moto;
    v: Vehicle;
    v = m;
    print("%\n", v);
}
