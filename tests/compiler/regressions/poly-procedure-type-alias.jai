// A `#type` alias whose parameters name a type variable (`*$T`) is a declaration like any other.
#import "Basic";

Iter :: #type (value: *$T, body: Code, flags: int);
Visitor :: #type (item: $T, count: int) -> $R;

main :: () {
    print("ok\n");
}
