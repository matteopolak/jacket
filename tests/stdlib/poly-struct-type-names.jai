// Instances of polymorphic structs print their parameters, like `Entity(Payload=s64)`.
#import "Basic";
Entity :: struct(Payload: Type) {
    payload: Payload;
}
Grid :: struct(W: int, H: int) {
    cells: [W * H] u8;
}
main :: () -> s64 {
    e: Entity(s64);
    if tprint("%", type_of(e)) != "Entity(Payload=s64)" return 1;
    anon: Entity(struct {
        a: int;
    });
    if tprint("%", type_of(anon)) != "Entity(Payload=(anonymous struct))" return 2;
    g: Grid(2, 3);
    if tprint("%", type_of(g)) != "Grid(W=2, H=3)" return 3;
    info := type_info(Grid(2, 3));
    if info.specified_parameters.count != 2 || info.polymorph_source_struct.name != "Grid" return 4;
    return 0;
}
