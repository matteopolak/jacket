// `#assert(cond, "format", args)` fills the message's `%` slots with the arguments, as
// n00bmind/reflector does.
Unsupported :: struct { x: int; }

check :: ($T: Type) {
    #assert(T == int, "type % (size %) is not supported", T, size_of(T));
}

main :: () {
    check(Unsupported);
}
