Handle :: #type,distinct u32;
Alias :: Handle;
Child :: #type,isa Handle;

consume :: (value: Handle) -> int {
    return cast(int) value;
}

main :: () -> int {
    a: Handle = 5;
    b: Alias = 3 * a + 2;
    child: Child = 13;
    return cast(int) b + consume(child);
}
