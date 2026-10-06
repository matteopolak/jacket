// The Tagged_Union module: set stores a value and its type, isa hands back a pointer only for the
// type that was stored last.
#import "Basic";
#import "Tagged_Union";

main :: () {
    check_tagged_union();
    print("ok\n");
}

check_tagged_union :: () {
    Value :: Tagged_Union(s64, string, float32, [3] u8);
    #assert size_of(Value.bytes) == 16;
    v: Value;
    assert(isa(v, s64) == null && isa(v, string) == null, "nothing set yet");
    set(*v, cast(s64) -5);
    assert(v.tag == s64 && isa(v, s64).* == -5 && isa(v, string) == null);
    set(*v, "text");
    assert(v.tag == string && isa(v, string).* == "text" && isa(v, s64) == null);
    set(*v, cast(float32) 2.5);
    assert(isa(v, float32).* == 2.5);
    set(*v, u8.[1, 2, 3]);
    bytes := isa(v, [3] u8);
    assert(bytes != null && bytes.*[2] == 3);
    // isa hands out a pointer into the union: writes through it stick.
    bytes.*[0] = 9;
    assert(isa(v, [3] u8).*[0] == 9);
}
