// The reviewed contract preserves reflected canonical names and parameter
// identity, without promising a verbose parameterized pretty-name spelling.
// rules: struct.14
#import "Basic";
#import "String";

Entity :: struct(Payload: Type) {
    payload: Payload;
}

Grid :: struct(W: int, H: int) {
    cells: [W * H] u8;
}

main :: () -> s64 {
    e: Entity(s64);
    if !contains(tprint("%", type_of(e)), type_info(type_of(e)).name) return 1;
    if type_of(e) != Entity(s64) return 5;

    anon: Entity(struct {
        a: int;
    });
    if !contains(tprint("%", type_of(anon)), type_info(type_of(anon)).name) return 2;

    g: Grid(2, 3);
    if !contains(tprint("%", type_of(g)), type_info(type_of(g)).name) return 3;
    if g.W != 2 || g.H != 3 return 6;

    info := type_info(Grid(2, 3));
    if info.specified_parameters.count != 2 || info.polymorph_source_struct.name != "Grid" return 4;

    return 0;
}
