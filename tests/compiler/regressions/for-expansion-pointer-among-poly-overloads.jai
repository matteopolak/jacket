// A `for_expansion` taking `*T` still iterates a `T` value through its address when other
// overloads in scope have polymorphic first parameters that do not match `T`.
#import "Basic";

Bag :: struct {
    count: int;
}

for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for i: 0..bag.count - 1 {
        `it := i;
        `it_index := i;
        #insert body;
    }
}

Wrapped :: struct($flag: bool) {
    count: int;
}

for_expansion :: (w: Wrapped($flag), body: Code, flags: For_Flags) #expand {
    #insert body;
}

Boxed :: struct($T: Type) {
    count: int;
}

for_expansion :: (b: *Boxed($T), body: Code, flags: For_Flags) #expand {
    #insert body;
}

sum :: (bag: Bag) -> int {
    total := 0;
    for value: bag total += value;
    return total;
}

main :: () {
    assert(sum(.{ 4 }) == 6);
}
