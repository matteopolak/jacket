// `#ifx` with a constant condition compiles only the arm it takes: the other one may have a
// different type, and may name something that does not exist.
#import "Basic";

Wrapper :: struct {
    data: [3] s64;
}

for_expansion :: (array: *Wrapper, body: Code, flags: For_Flags) #expand {
    for index: 0..array.data.count - 1 {
        `it_index := index;
        `it := #ifx flags & .POINTER then *array.data[index] else array.data[index];
        #insert(remove = #assert false "no remove") body;
    }
}

main :: () {
    w: Wrapper;
    for *w it.* = it_index + 1;

    sum := 0;
    for w sum += it;

    assert(sum == 6);

    x := #ifx true then 1 else "text";

    assert(x == 1);
    y := #ifx false then 1 else "text";

    assert(y == "text");
    z := #ifx true then 2 else NOT_DECLARED_ANYWHERE;

    assert(z == 2);
}
