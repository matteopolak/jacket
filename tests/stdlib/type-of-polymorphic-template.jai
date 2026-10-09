// A polymorphic struct template is a `Type` value that prints as its name, and
// `type_of` of a polymorphic procedure keeps its signature, with `$` for the parts that
// depend on a type variable.
#import "Basic";

Entity :: struct(Payload: Type) {
    payload: Payload;
}

other_poly :: (x: $T, y: int) -> T, string {
    return x, "";
}

with_pointer :: (x: *$T, y: [$N] int, s: string) {}

baked :: ($N: int, f: float) -> int {
    return N;
}

with_rest :: (x: int, rest: ..Any) -> $R {
    return 0;
}

main :: () -> s64 {
    kind := type_of(Entity);
    if kind != Type return 1;
    if tprint("%", Entity) != "Entity" return 2;
    template := Entity;
    if template != Entity return 3;
    if template == Entity(int) return 4;
    if tprint("%", Entity(int)) != "Entity(Payload=s64)" return 5;

    if tprint("%", type_of(other_poly)) != "procedure ($, s64) -> ($, string)" return 6;
    if tprint("%", type_of(with_pointer)) != "procedure (*$, [0] s64, string)" return 7;
    if tprint("%", type_of(baked)) != "procedure (s64, float32) -> s64" return 8;
    if tprint("%", type_of(with_rest)) != "procedure (s64, [] Any) -> $" return 9;

    info := cast(*Type_Info_Procedure) type_of(other_poly);
    if info.argument_types.count != 2 || info.return_types.count != 2 return 10;
    return 0;
}
