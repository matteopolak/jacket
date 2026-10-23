// A polymorphic struct cannot take context arguments (after `,,`) when baked.
Pair :: struct($T: Type) {
    a: T;
}

IntPair :: #bake_arguments Pair(T = int,, allocator = context.allocator);

main :: () {
    p: IntPair;
}
